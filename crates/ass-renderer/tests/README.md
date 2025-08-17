# Compatibility Testing Guide

This document explains how to run comprehensive compatibility tests between our ass-renderer and libass.

## Prerequisites

### Install libass

**macOS (Homebrew):**
```bash
brew install libass
```

**Ubuntu/Debian:**
```bash
sudo apt-get install libass-dev
```

**Fedora/RHEL:**
```bash
sudo dnf install libass-devel
```

**Windows:**
```bash
# Using vcpkg
vcpkg install libass
```

### Install Dependencies

```bash
# Add image support for visual diffs
cargo add image --features png

# Optional: serde for JSON output
cargo add serde --features derive
cargo add serde_json
```

## Running Tests

### Basic Compatibility Tests

Run all compatibility tests:
```bash
cargo test --features libass-compare
```

Run specific test categories:
```bash
# Basic text rendering
cargo test test_basic_text_compatibility --features libass-compare

# Animation tests  
cargo test test_animation_compatibility --features libass-compare

# Transform tests
cargo test test_transform_compatibility --features libass-compare
```

### With Visual Diff Generation

Enable visual diff output:
```bash
GENERATE_VISUAL_DIFFS=1 cargo test --features libass-compare,image
```

This will create PNG files in `test_output/compatibility/` showing:
- `{test_name}_ours.png` - Our renderer output
- `{test_name}_libass.png` - libass reference output  
- `{test_name}_diff.png` - Difference heatmap

### Performance Benchmarking

Run with performance measurement:
```bash
cargo test --features libass-compare --release -- --nocapture
```

### Custom Test Configuration

Create a custom test with specific tolerance:
```rust
let config = TestConfig {
    pixel_tolerance: 3,           // Allow 3 pixel value difference
    significance_threshold: 0.01, // 1% difference threshold
    generate_visual_diffs: true,
    test_animations: true,
    animation_step_cs: 5,         // 50ms animation steps
    ..Default::default()
};
```

## Test Categories

### 1. Basic Text Rendering
- Simple dialogue lines
- Font styling (bold, italic, underline)
- Color changes
- Font size variations

### 2. Text Effects
- Outlines and shadows
- Blur effects
- Color overrides
- Spacing adjustments

### 3. Animations
- Movement (`\move`)
- Fading (`\fad`, `\fade`)
- Transforms (`\t`)
- Karaoke effects (`\k`, `\kf`)

### 4. Transformations
- Rotation (`\frx`, `\fry`, `\frz`)
- Scaling (`\fscx`, `\fscy`)
- Shearing (`\fax`, `\fay`)
- Origin points (`\org`)

### 5. Drawing Commands
- Vector paths (`\p`)
- Bezier curves
- Clipping regions (`\clip`)

### 6. Edge Cases
- Empty events
- Invalid parameters
- Off-screen content
- Malformed tags

## Interpreting Results

### Compatibility Metrics

**Pixel Difference Percentage:**
- `0.000%` - Perfect match (pixel identical)
- `< 0.001%` - Excellent (sub-pixel differences)
- `< 0.01%` - Good (minor rasterization differences)
- `< 0.1%` - Acceptable (visible but minor differences)
- `> 0.1%` - Poor (significant visual differences)

**Maximum Pixel Difference:**
- `0-2` - Excellent (imperceptible)
- `3-5` - Good (very minor)
- `6-15` - Acceptable (minor)
- `16+` - Poor (noticeable)

### Common Difference Sources

1. **Font Rasterization:** Different font rendering engines
2. **Antialiasing:** Slight differences in edge smoothing
3. **Color Blending:** Alpha blending precision differences
4. **Animation Timing:** Frame timing precision differences
5. **Coordinate Precision:** Sub-pixel positioning differences

## Continuous Integration

### GitHub Actions Example

```yaml
name: Compatibility Tests

on: [push, pull_request]

jobs:
  compatibility:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install libass
      run: sudo apt-get install libass-dev
      
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Run compatibility tests
      run: cargo test --features libass-compare
      
    - name: Upload visual diffs
      if: failure()
      uses: actions/upload-artifact@v3
      with:
        name: visual-diffs
        path: test_output/
```

## Debugging Failed Tests

### 1. Visual Inspection
Check the generated PNG files to see visual differences:
```bash
open test_output/compatibility/{test_name}_diff.png
```

### 2. Increase Tolerance
For minor differences, adjust tolerance:
```rust
let config = TestConfig {
    pixel_tolerance: 5,  // Increase from default
    significance_threshold: 0.02, // Increase from 1% to 2%
    ..Default::default()
};
```

### 3. Animation Analysis
For animation tests, check individual frames:
```bash
# Run with detailed output
RUST_LOG=debug cargo test test_animation_compatibility --features libass-compare -- --nocapture
```

### 4. Performance Analysis
Compare rendering performance:
```bash
cargo test --features libass-compare --release -- --nocapture | grep "render time"
```

## Adding New Tests

### Create Test Script
```rust
fn create_custom_test_script() -> String {
    r#"[Script Info]
Title: Custom Test
PlayResX: 1920
PlayResY: 1080

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,48,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,1,2,50,50,50,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Your test content here"#.to_string()
}
```

### Add to Test Suite
```rust
#[test]
#[cfg(feature = "libass-compare")]
fn test_custom_feature() {
    let context = RenderContext::new(TEST_WIDTH, TEST_HEIGHT);
    let config = TestConfig::default();
    let mut tester = CompatibilityTester::new(context, config).unwrap();
    
    let script_content = create_custom_test_script();
    let script = Script::parse(&script_content).unwrap();
    let result = tester.test_script_compatibility(&script, "custom_feature").unwrap();
    
    assert!(result.passed, "Custom test failed with {}% difference", 
           result.pixel_diff_percentage * 100.0);
}
```

## Performance Targets

### Rendering Speed
- **Target:** Our renderer within 150% of libass speed
- **Acceptable:** Within 200% of libass speed
- **Poor:** More than 300% of libass speed

### Memory Usage  
- **Target:** Within 120% of libass memory usage
- **Acceptable:** Within 150% of libass memory usage
- **Poor:** More than 200% of libass memory usage

### Compatibility
- **Target:** 95%+ tests pass with <0.01% difference
- **Acceptable:** 90%+ tests pass with <0.05% difference
- **Poor:** <85% tests pass or >0.1% average difference

## Troubleshooting

### libass Not Found
```bash
# Check if libass is installed
pkg-config --modversion libass

# macOS specific
brew list libass

# Ubuntu/Debian
dpkg -l | grep libass
```

### Build Errors
```bash
# Clean rebuild
cargo clean
cargo build --features libass-compare

# Check feature flags
cargo build --features libass-compare --verbose
```

### Test Failures
```bash
# Run with maximum output
RUST_LOG=trace cargo test test_name --features libass-compare -- --nocapture

# Generate visual diffs for debugging
mkdir -p test_output/compatibility
cargo test --features libass-compare,image
```