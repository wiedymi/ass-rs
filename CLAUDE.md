# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build/Test/Lint Commands
- Build: `cargo build [--release] [--features="simd,arena"]`
- Test all: `cargo test --all-features`
- Test single: `cargo test test_name`
- Test file: `cargo test --test file_name`
- Format: `cargo fmt --all`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Benchmarks: `cargo bench --features="benches"`
- WASM tests: `wasm-pack test --chrome`
- Fuzzing: `cargo +nightly fuzz run tokenizer`
- Coverage: `cargo tarpaulin --all-features`

## Code Style Guidelines
- **Safety**: No unsafe code allowed - absolutely forbidden
- **Imports**: Group by scope (std → external → internal); specific imports for frequent items
- **Formatting**: 4-space indentation; standard Rust formatting; rustfmt defaults
- **Types**: Use zero-copy spans (`&str`) for performance; custom Result types; feature-gated optimizations
- **Naming**: CamelCase for types; snake_case for functions/files; SCREAMING_SNAKE_CASE for constants
- **Error Handling**: Use `thiserror` for enums (no `anyhow`); prefer Result over panics
- **Documentation**: Use only Rustdoc (`///` for public, `//!` for modules); no inline comments; example-heavy
- **Testing**: >90% test coverage required; fuzz hot paths; WASM tests mandatory
- **Dependencies**: Minimal deps (<50KB); pin versions in workspace; feature-gate heavy ones
- **Performance**: <5ms/operation, <1.1x input memory; use zero-copy, arenas, SIMD where beneficial
- **Modularity**: Submodules per concern; traits for extensibility; file size <200 LOC
- **Features**: Consistent across crates; default minimal; gate extras; maintain no_std compatibility
- **Workarounds**: Never use workarounds, bypass, or skip logic; never use `allow(clippy)` to fix something

## Architecture Overview

### Project Structure
ASS-RS is designed as a modular ASS (Advanced SubStation Alpha) subtitle parser surpassing libass in performance and safety. The current workspace contains:
- **`crates/ass-core`**: Zero-copy parser with trait-based plugin system
- **Future crates**: renderer, editor, CLI, WASM, benchmarks (planned)

### Core Design Principles
- **Zero-Copy**: Uses `&'a str` spans throughout AST (target: ~1.1x input memory)
- **Plugin System**: `TagHandler` and `SectionProcessor` traits with runtime `ExtensionRegistry`
- **Thread Safety**: Immutable `Script` design with `Send + Sync`
- **Performance Targets**: <5ms parsing, <2ms incremental updates
- **Safety First**: Zero unsafe code, comprehensive error handling with `thiserror`

### Key Features
- **Format Support**: ASS v4+, SSA v4, libass 0.17.4+ compatibility
- **SIMD Acceleration**: Feature-gated with `simd` and `simd-full` features
- **Arena Allocation**: Optional `bumpalo` integration for reduced allocations
- **Analysis Engine**: Linting, style resolution, performance analysis
- **Incremental Parsing**: Editor-friendly partial re-parsing

### Feature Flags
- **Default**: `std`, `analysis`, `plugins`
- **Performance**: `simd`, `simd-full`, `arena`
- **Platform**: `nostd` for embedded/WASM
- **Development**: `serde`, `benches`, `stream`

## Common Clippy Issues to Avoid

Based on experience, here are common clippy warnings and how to avoid them:

### 1. **Large Enum Variants** (`large_enum_variant`)
- **Issue**: Enum variants with significantly different sizes
- **Fix**: Box large variants to reduce size difference
- **Example**: `LineContent::Style(Box<Style<'a>>)` instead of `LineContent::Style(Style<'a>)`

### 2. **Documentation Formatting** (`doc_markdown`)
- **Issue**: Missing backticks around code in documentation
- **Fix**: Always use backticks for code elements, types, and variable names
- **Example**: `` `SectionType` `` not `SectionType`

### 3. **Option/Result Handling**
- **`is_some_and`**: Use `option.is_some_and(|x| predicate(x))` instead of `option.map_or(false, |x| predicate(x))`
- **`map_or`**: Use `option.map_or(default, |x| expression)` for simple transformations
- **`?` operator**: Use `?` instead of `if let Err(e) { return Err(e); }`

### 4. **Pattern Matching**
- **`single_match_else`**: Use `if let` for single-pattern matches with else
- **`match` vs `if let`**: Use `if let Some(x) = option` instead of `match option { Some(x) => ..., None => {} }`

### 5. **Import Organization** (`items_after_statements`)
- **Issue**: `use` statements after other code in functions
- **Fix**: Move all `use` statements to the beginning of the function/module

### 6. **Literal Formatting** (`unreadable_literal`)
- **Issue**: Large numbers without separators
- **Fix**: Use underscores: `999_999` instead of `999999`

### 7. **Unused Variables**
- **Issue**: Using descriptive names for unused variables
- **Fix**: Use just `_` for unused variables, not `_variable_name`

### 8. **Derive Traits**
- **Issue**: Missing required derives when composing types
- **Fix**: When a struct contains a type, it needs the same derives (e.g., if containing `Vec<Change>` where `Change` needs `Eq`, the struct also needs `Eq`)

### 9. **Method Attributes**
- **`must_use`**: Add to methods returning values that shouldn't be ignored
- **`const fn`**: Be careful with methods that allocate (`Vec::new()` isn't const)

### 10. **Type References** (`use_self`)
- **Issue**: Using full type name instead of `Self` in impl blocks
- **Fix**: Use `Self` instead of repeating the type name

### 11. **Boolean Logic** (`overly_complex_bool_expr`)
- **Issue**: Complex boolean expressions that always evaluate to true/false
- **Fix**: Simplify logic and remove redundant conditions

### 12. **Never Use `allow(clippy::...)`**
- **Principle**: Fix the underlying issue instead of suppressing the warning
- **Exception**: Only for false positives with clear documentation

### 13. **Unnecessary Cloning**
- **Issue**: Unnecessary `.clone()` calls that can be avoided
- **Fix**: Use references, `Cow`, or reference-based types to minimize cloning
- **Examples**: 
  - Prefer `&str` over `String` when possible
  - Use `Cow<str>` for mixed owned/borrowed scenarios
  - Use `.to_owned()` only when absolutely necessary

### 14. **Redundant Field Names**
- **Issue**: Repeating field names in struct initialization
- **Fix**: Use shorthand when variable name matches struct field name
- **Example**: `Style { name, size }` instead of `Style { name: name, size: size }`

### 15. **Wildcard Imports** (`enum_glob_use`)
- **Issue**: Using wildcard imports for enum variants (`use Section::*`)
- **Fix**: Use explicit imports to make code clearer
- **Example**: `use Section::{Events, Fonts, Graphics, ScriptInfo, Styles}` instead of `use Section::*`

### 16. **Unused Variables** (`collection_is_never_read`, `unused_self`)
- **Issue**: Variables declared but never used, or unused `self` parameters
- **Fix**: Remove unused variables entirely, or convert methods to associated functions
- **Example**: Remove `let processed_sections = HashSet::new();` if never used
- **Example**: Change `fn method(&self, ...)` to `fn method(...)` if `self` isn't used

### 17. **Raw String Literals** (`needless_raw_string_hashes`)
- **Issue**: Using unnecessary `#` symbols around raw strings when not needed
- **Fix**: Use `r"..."` instead of `r#"..."#` when the string doesn't contain quotes
- **Example**: `r"[Script Info]\nTitle: Test"` instead of `r#"[Script Info]\nTitle: Test"#`

### 18. **String Concatenation** (`string_add`)
- **Issue**: Using `+` operator to concatenate strings (inefficient)
- **Fix**: Use `String::push_str()` method for better performance
- **Example**: 
  ```rust
  // Bad
  let result = base.to_string() + "\nVersion: 1.0" + &suffix;
  
  // Good
  let mut result = base.to_string();
  result.push_str("\nVersion: 1.0");
  result.push_str(&suffix);
  ```

### 19. **Format String Inlining** (`uninlined_format_args`)
- **Issue**: Not using direct variable interpolation in format strings
- **Fix**: Use `{variable}` instead of `{}, variable` in format macros
- **Example**: `println!("Value: {value}")` instead of `println!("Value: {}", value)`

### 20. **Manual String Creation** (`manual_string_new`)
- **Issue**: Using `"".to_string()` instead of `String::new()`
- **Fix**: Use `String::new()` for empty strings (more explicit and potentially faster)
- **Example**: `String::new()` instead of `"".to_string()`

### 21. **Safe Arithmetic Over Casting** (`cast_possible_wrap`, `cast_sign_loss`)
- **Issue**: Using `as` casts that can wrap around or lose sign information
- **Fix**: Use safe arithmetic operations instead of casting between signed/unsigned types
- **Example**:
  ```rust
  // Bad - potential overflow/underflow
  let new_start = (span.start as isize + offset).max(0) as usize;
  
  // Good - safe arithmetic
  let new_start = if new_len >= old_len {
      span.start + (new_len - old_len)
  } else {
      span.start.saturating_sub(old_len - new_len)
  };
  ```

### 22. **Method Chaining Optimization** (`map_unwrap_or`)
- **Issue**: Using `.map(f).unwrap_or(default)` pattern
- **Fix**: Use `.map_or(default, f)` for better performance and readability
- **Example**: `option.map_or("[Unknown]", |(h, _)| *h)` instead of `option.map(|(h, _)| *h).unwrap_or("[Unknown]")`

## Post-Iteration Checklist
Always run: fmt → clippy → test → bench → coverage → build release