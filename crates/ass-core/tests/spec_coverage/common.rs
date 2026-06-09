//! Shared fixtures for the ASS specification coverage integration tests.
//!
//! Hosts the comprehensive script sample exercised by several submodules so the
//! large literal lives in one place.

/// Complex ASS script with comprehensive spec coverage
pub const COMPREHENSIVE_SCRIPT: &str = r"[Script Info]
Title: Comprehensive Spec Coverage Test
ScriptType: v4.00+
WrapStyle: 0
ScaledBorderAndShadow: yes
YCbCr Matrix: TV.709
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,50,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1
Style: Title,Impact,72,&H00FFD700,&H000000FF,&H00000000,&H80000000,1,0,0,0,120,120,2,0,1,3,3,2,0,0,0,1
Style: Subtitle,Calibri,45,&H00E6E6FA,&H000000FF,&H00404040,&H80000000,0,1,0,0,95,95,1,0,1,1,1,8,20,20,20,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text

; Basic dialogue event
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello world! Basic text without formatting.

; Complex style override tags
Dialogue: 0,0:00:05.00,0:00:10.00,Default,,0,0,0,,{\b1}Bold {\i1}and italic{\i0}{\b0} with {\u1}underline{\u0} and {\s1}strikeout{\s0}.

; Font and color changes
Dialogue: 0,0:00:10.00,0:00:15.00,Default,,0,0,0,,{\fn Impact}{\fs72}Large Impact font {\c&H0000FF&}in red color{\c}.

; Position and movement animations
Dialogue: 0,0:00:15.00,0:00:20.00,Default,,0,0,0,,{\pos(960,540)}Centered text that {\move(960,540,100,100,0,1000)}moves to corner.

; Rotation and scaling
Dialogue: 0,0:00:20.00,0:00:25.00,Default,,0,0,0,,{\frz45}{\fscx150}{\fscy75}Rotated and scaled text with distortion.

; Advanced animation with timing
Dialogue: 0,0:00:25.00,0:00:30.00,Default,,0,0,0,,{\t(0,2000,\fscx200\fscy50)}{\t(2000,4000,\frz360)}Complex multi-stage animation.

; Fade effects
Dialogue: 0,0:00:30.00,0:00:35.00,Default,,0,0,0,,{\fade(255,0,0,0,500,4000,4500)}Text with complex fade in and out.

; Drawing commands and vector graphics
Dialogue: 0,0:00:35.00,0:00:40.00,Default,,0,0,0,,{\p1}{\pos(960,540)}m 0 0 l 100 0 100 100 0 100{\p0} Vector square drawn.

; Karaoke effects
Dialogue: 0,0:00:40.00,0:00:45.00,Default,,0,0,0,,{\k50}Ka{\k30}ra{\k70}o{\k40}ke {\kf100}effect {\ko50}demo.

; Clipping and masking
Dialogue: 0,0:00:45.00,0:00:50.00,Default,,0,0,0,,{\clip(100,100,500,300)}Clipped text region for special effects.

; 3D perspective and shearing
Dialogue: 0,0:00:50.00,0:00:55.00,Default,,0,0,0,,{\frx45}{\fry30}{\fax0.5}{\fay0.2}3D rotated and sheared text.

; Border and shadow effects
Dialogue: 0,0:00:55.00,0:01:00.00,Default,,0,0,0,,{\bord5}{\shad3}{\3c&H00FF00&}{\4c&H0000FF&}Thick border with colored shadow.

; Unicode and bidirectional text
Dialogue: 0,0:01:00.00,0:01:05.00,Default,,0,0,0,,Mixed {\b1}English{\b0} and {\i1}العربية{\i0} text with {\u1}עברית{\u0} support.

; Line breaks and spacing
Dialogue: 0,0:01:05.00,0:01:10.00,Default,,0,0,0,,First line\NSecond line\nThird line with\hhard spaces.

; Blur and glow effects
Dialogue: 0,0:01:10.00,0:01:15.00,Default,,0,0,0,,{\blur2}{\be1}Blurred text with edge blur effect applied.

; Alignment variations
Dialogue: 0,0:01:15.00,0:01:20.00,Default,,0,0,0,,{\an1}Bottom left aligned text for positioning test.
Dialogue: 0,0:01:15.00,0:01:20.00,Default,,0,0,0,,{\an5}Middle center aligned text overlay.
Dialogue: 0,0:01:15.00,0:01:20.00,Default,,0,0,0,,{\an9}Top right aligned text corner.

; Color alpha and transparency
Dialogue: 0,0:01:20.00,0:01:25.00,Default,,0,0,0,,{\alpha&H80&}Semi-transparent text with {\1a&HFF&}invisible primary color.

; Complex nested animations
Dialogue: 0,0:01:25.00,0:01:35.00,Default,,0,0,0,,{\t(0,5000,\move(100,100,1820,980))}{\t(2000,8000,\frz720)}{\t(4000,10000,\fscx300\fscy300)}Ultimate complex animation sequence.

; Picture event
Picture: 0,0:01:35.00,0:01:40.00,Default,,0,0,0,,logo.png

; Sound event
Sound: 0,0:01:40.00,0:01:45.00,Default,,0,0,0,,audio.wav

; Movie event
Movie: 0,0:01:45.00,0:01:50.00,Default,,0,0,0,,video.avi

; Command event
Command: 0,0:01:50.00,0:01:55.00,Default,,0,0,0,,{\c&H00FF00&}Special command event type.

; Comment events for documentation
Comment: 0,0:00:00.00,0:00:00.00,Default,,0,0,0,,This is a comment that should be ignored in rendering.
Comment: 0,0:00:00.00,0:00:00.00,Default,,0,0,0,,{\b1}Comments can also contain formatting codes.

[Fonts]
fontname: CustomFont.ttf
M3%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J
M<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J
M<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J
M<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J<C%J
`
end

fontname: AnotherFont.otf
M9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F
M9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F9F=F
`
end

[Graphics]
filename: background.png
M5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O
M5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O
M5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O5#-O
`
end

filename: overlay.jpg
M2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A
M2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A2&5A
`
end
";
