#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// Borrow the underlying byte slice for this span.
    #[inline]
    pub fn as_bytes(self, src: &[u8]) -> &[u8] {
        // SAFETY: Spans are always created with valid bounds
        unsafe { src.get_unchecked(self.start..self.end) }
    }

    /// Borrow the underlying UTF-8 string slice for this span. Returns `None` if the bytes are not valid UTF-8.
    #[inline]
    pub fn as_str(self, src: &[u8]) -> Option<&str> {
        std::str::from_utf8(self.as_bytes(src)).ok()
    }

    /// Get the length of this span
    #[inline]
    pub fn len(self) -> usize {
        self.end - self.start
    }

    /// Check if span is empty
    #[inline]
    pub fn is_empty(self) -> bool {
        self.start >= self.end
    }
}

/// Zero-copy line tokenizer.
///
/// Walks the input byte slice once and yields spans that delimit each line, *without* allocating
/// or copying. New-line characters ("\n" or "\r\n") are *not* included in the returned span.
/// Successive calls to `next()` are O(length). Overall complexity is O(n).
///
/// The tokenizer is agnostic to UTF-8 validity; it treats the input as raw bytes.
/// Optimized for performance with SIMD-like byte scanning where possible.
pub struct Tokenizer<'a> {
    src: &'a [u8],
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    /// Create a new tokenizer over the provided byte slice.
    #[inline]
    pub fn new(src: &'a [u8]) -> Self {
        Self { src, pos: 0 }
    }

    /// Current position in the source.
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// The source byte slice.
    #[inline]
    pub fn source(&self) -> &'a [u8] {
        self.src
    }

    /// Fast scan for newline characters using optimized byte search
    #[inline]
    fn find_line_end(&self, start: usize) -> usize {
        let slice = &self.src[start..];

        // Use optimized byte search for common patterns
        if let Some(pos) = memchr::memchr2(b'\n', b'\r', slice) {
            start + pos
        } else {
            self.src.len()
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Span;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.src.len() {
            return None;
        }

        let start = self.pos;
        let end = self.find_line_end(start);

        // At this point, [start, end) delimits the line sans newline chars.
        let span = Span { start, end };

        // Advance pos past the newline sequence.
        self.pos = end;
        if self.pos < self.src.len() {
            match self.src[self.pos] {
                b'\r' => {
                    self.pos += 1;
                    if self.pos < self.src.len() && self.src[self.pos] == b'\n' {
                        self.pos += 1;
                    }
                }
                b'\n' => {
                    self.pos += 1;
                }
                _ => {}
            }
        }

        Some(span)
    }

    /// Provide size hint for better allocation patterns
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.src.len() - self.pos;
        // Estimate based on average line length (conservative estimate: 40 chars/line)
        let estimate = remaining / 40;
        (estimate / 2, Some(estimate * 2))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssToken {
    /// A section header like "[Script Info]". Contains the span of the name inside brackets.
    SectionHeader { name: Span },
    /// A key/value pair such as `Key: Value`.
    KeyValue { key: Span, value: Span },
    /// Empty line (whitespace or nothing).
    Empty,
    /// A comment line starting with ';' or '#'. Stores whole span sans newline.
    Comment { span: Span },
    /// Any other raw line we don't parse yet.
    Raw { span: Span },
}

/// Iterator that yields `AssToken`s over the input buffer.
/// It builds on the generic `Tokenizer`, keeping zero-copy semantics by returning `Span`s
/// that reference the original slice.
///
/// Optimized for performance with reduced allocations and better cache locality.
pub struct AssTokenizer<'a> {
    inner: Tokenizer<'a>,
    src: &'a [u8],
}

impl<'a> AssTokenizer<'a> {
    #[inline]
    pub fn new(src: &'a [u8]) -> Self {
        Self {
            inner: Tokenizer::new(src),
            src,
        }
    }

    /// Trim leading and trailing ASCII whitespace (space and tab only) from a span, returning the trimmed span.
    /// Optimized with branchless operations where possible.
    #[inline]
    fn trim_span(&self, span: Span) -> Span {
        if span.is_empty() {
            return span;
        }

        let bytes = unsafe { self.src.get_unchecked(span.start..span.end) };
        let mut start_offset = 0;
        let mut end_offset = bytes.len();

        // Trim start
        while start_offset < end_offset {
            match bytes[start_offset] {
                b' ' | b'\t' => start_offset += 1,
                _ => break,
            }
        }

        // Trim end
        while end_offset > start_offset {
            match bytes[end_offset - 1] {
                b' ' | b'\t' => end_offset -= 1,
                _ => break,
            }
        }

        Span {
            start: span.start + start_offset,
            end: span.start + end_offset,
        }
    }

    #[inline]
    fn classify_line(&self, span: Span) -> AssToken {
        // Trim whitespace to check if line is empty.
        let trimmed = self.trim_span(span);
        if trimmed.is_empty() {
            return AssToken::Empty;
        }

        let bytes = unsafe { self.src.get_unchecked(trimmed.start..trimmed.end) };
        let first_byte = bytes[0];

        // Comment? leading ';' or '#'
        if matches!(first_byte, b';' | b'#') {
            return AssToken::Comment { span: trimmed };
        }

        // Section header? starts with '[' and ends with ']'
        if first_byte == b'[' && trimmed.len() >= 2 {
            let last = bytes[bytes.len() - 1];
            if last == b']' {
                // name inside brackets.
                let name_span = Span {
                    start: trimmed.start + 1,
                    end: trimmed.end - 1,
                };
                let name_span = self.trim_span(name_span);
                return AssToken::SectionHeader { name: name_span };
            }
        }

        // Key-value pair? Look for colon ':' in trimmed line.
        // Use optimized byte search
        if let Some(rel_colon) = memchr::memchr(b':', bytes) {
            let colon_idx = trimmed.start + rel_colon;
            let key_span = Span {
                start: trimmed.start,
                end: colon_idx,
            };
            let val_span = Span {
                start: colon_idx + 1,
                end: trimmed.end,
            };
            let key_span = self.trim_span(key_span);
            let val_span = self.trim_span(val_span);
            return AssToken::KeyValue {
                key: key_span,
                value: val_span,
            };
        }

        // Otherwise raw.
        AssToken::Raw { span: trimmed }
    }
}

impl<'a> Iterator for AssTokenizer<'a> {
    type Item = AssToken;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|span| self.classify_line(span))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

/// Efficient batch processing for multiple tokens
impl<'a> AssTokenizer<'a> {
    /// Collect all tokens into a vector with pre-allocated capacity
    pub fn collect_all(mut self) -> Vec<AssToken> {
        let (lower, upper) = self.size_hint();
        let mut tokens = Vec::with_capacity(upper.unwrap_or(lower));

        while let Some(token) = self.next() {
            tokens.push(token);
        }

        tokens
    }

    /// Process tokens in batches for better cache performance  
    pub fn process_batch<F>(&mut self, batch_size: usize, mut process_fn: F)
    where
        F: FnMut(&[AssToken]),
    {
        let mut batch = Vec::with_capacity(batch_size);

        while let Some(token) = self.next() {
            batch.push(token);

            if batch.len() >= batch_size {
                process_fn(&batch);
                batch.clear();
            }
        }

        if !batch.is_empty() {
            process_fn(&batch);
        }
    }
}

// Performance optimizations: add memchr dependency for fast byte searching
#[cfg(not(feature = "std"))]
mod memchr_fallback {
    #[inline]
    pub fn memchr(needle: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().position(|&b| b == needle)
    }

    #[inline]
    pub fn memchr2(needle1: u8, needle2: u8, haystack: &[u8]) -> Option<usize> {
        haystack.iter().position(|&b| b == needle1 || b == needle2)
    }
}

#[cfg(not(feature = "std"))]
use memchr_fallback as memchr;

#[cfg(feature = "std")]
use memchr;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_tokenization() {
        let data = b"Line1\nLine2\r\nLine3";
        let toks: Vec<_> = Tokenizer::new(data).collect();
        let lines: Vec<_> = toks.iter().map(|sp| sp.as_str(data).unwrap()).collect();
        assert_eq!(lines, vec!["Line1", "Line2", "Line3"]);
    }

    #[test]
    fn classify_section_and_keyvalue() {
        let data = b"[Script Info]\nTitle: Example\n;comment\n  \n[Events]\n";
        let mut toks = AssTokenizer::new(data);

        match toks.next().unwrap() {
            AssToken::SectionHeader { name } => {
                assert_eq!(name.as_str(data).unwrap(), "Script Info");
            }
            other => panic!("Unexpected token: {other:?}"),
        }
        match toks.next().unwrap() {
            AssToken::KeyValue { key, value } => {
                assert_eq!(key.as_str(data).unwrap(), "Title");
                assert_eq!(value.as_str(data).unwrap(), "Example");
            }
            other => panic!("Unexpected token: {other:?}"),
        }
        match toks.next().unwrap() {
            AssToken::Comment { .. } => {}
            other => panic!("Unexpected token: {other:?}"),
        }
        match toks.next().unwrap() {
            AssToken::Empty => {}
            other => panic!("Unexpected token: {other:?}"),
        }
        match toks.next().unwrap() {
            AssToken::SectionHeader { name } => {
                assert_eq!(name.as_str(data).unwrap(), "Events");
            }
            _ => unreachable!(),
        }
        assert!(toks.next().is_none());
    }

    #[test]
    fn performance_batch_processing() {
        let data = b"[Section1]\nKey1: Value1\nKey2: Value2\n[Section2]\nKey3: Value3\n";
        let mut tokenizer = AssTokenizer::new(data);
        let mut processed_batches = 0;

        tokenizer.process_batch(2, |batch| {
            processed_batches += 1;
            assert!(batch.len() <= 2);
        });

        assert!(processed_batches > 0);
    }

    #[test]
    fn span_operations() {
        let span = Span { start: 5, end: 10 };
        assert_eq!(span.len(), 5);
        assert!(!span.is_empty());

        let empty_span = Span { start: 5, end: 5 };
        assert_eq!(empty_span.len(), 0);
        assert!(empty_span.is_empty());
    }
}
