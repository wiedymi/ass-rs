# ass-renderer

High-performance ASS (Advanced SubStation Alpha) subtitle renderer with modular backend support.

## Features

- **Multiple Rendering Backends**
  - Software (CPU) rendering with tiny-skia (RECOMMENDED - fully implemented)
  - WebGPU for web and native GPU acceleration (experimental)
  - Vulkan for high-performance GPU rendering (experimental)
  - Metal for macOS/iOS (experimental)
  - WebGL is NOT supported - use Software backend instead
  - Automatic backend selection (defaults to Software)

- **Complete ASS/SSA Support**
  - All ASS v4+ tags and formatting
  - Style inheritance and resolution
  - Complex positioning (pos, move, org)
  - Animation support (\t tags)
  - Karaoke effects
  - Drawing commands (\p)
  - Clipping (\clip, \iclip)

- **Advanced Features**
  - Collision detection and resolution
  - Incremental rendering for performance
  - Plugin system for custom effects
  - Comprehensive caching system
  - Zero-copy design where possible

## Usage

```rust
use ass_renderer::{Renderer, RenderContext};
use ass_core::parser::Script;

// Parse ASS script
let script = Script::parse(script_text)?;

// Create rendering context
let context = RenderContext::new(1920, 1080);

// Create renderer with automatic backend selection
let mut renderer = Renderer::with_auto_backend(context)?;

// Render frame at specific time (in centiseconds)
let frame = renderer.render_frame(&script, 500)?;

// Access pixel data
let pixels = frame.data(); // RGBA8 format
```

## Backends

### Software Backend (RECOMMENDED)
Pure CPU rendering using tiny-skia. **Fully implemented and production-ready.** Works everywhere, no GPU dependencies.

```rust
use ass_renderer::{Renderer, RenderContext, BackendType};

// Recommended: Use Software backend explicitly
let mut renderer = Renderer::new(BackendType::Software, context)?;
```

### Hardware Backends (Experimental)
GPU acceleration backends are experimental and may have incomplete feature support:

- **WebGPU**: Cross-platform GPU acceleration (basic implementation)
- **Vulkan**: High-performance native GPU (basic implementation)  
- **Metal**: macOS/iOS GPU acceleration (basic implementation)
- **WebGL**: NOT SUPPORTED - use Software backend for web

For production use, we strongly recommend the Software backend which has full feature support and has been thoroughly tested.

## Collision Detection

Automatic collision detection prevents subtitle overlap:

```rust
use ass_renderer::collision::{CollisionResolver, PositionedEvent, BoundingBox};

let mut resolver = CollisionResolver::new(1920.0, 1080.0);

// Add fixed event
resolver.add_fixed(event1);

// Find non-colliding position for new event
let new_position = resolver.find_position(event2);
```

## Animation System

Support for ASS animation tags (\t):

```rust
use ass_renderer::animation::{AnimationController, AnimationTiming, AnimatedValue};

let mut controller = AnimationController::new();

// Add animation track
let timing = AnimationTiming::new(0, 100, 1.0);
let track = AnimationTrack::new(
    "font_size".to_string(),
    timing,
    AnimatedValue::Float { from: 20.0, to: 40.0 },
    AnimationInterpolation::Linear,
);
controller.add_track(track);

// Evaluate at specific time
let state = controller.evaluate(50);
```

## Performance

Optimized for high performance:
- Target: <5ms per 4K frame
- SIMD acceleration (when enabled)
- Incremental rendering support
- Extensive caching
- Parallel processing with rayon

## Feature Flags

- `default`: Enables software backend and analysis integration
- `software-backend`: CPU rendering support
- `hardware-backend`: Vulkan and Metal support
- `web-backend`: WebGPU support
- `simd`: SIMD acceleration
- `arena`: Arena allocator for reduced allocations
- `analysis-integration`: Integration with ass-core analysis
- `backend-metrics`: Performance metrics collection
- `serde`: Serialization support
- `nostd`: No-std support (limited backends)

## Benchmarks

Run benchmarks with:
```bash
cargo bench --package ass-renderer --features benches
```

Performance targets (vs libass):
- Simple subtitles: 2-3x faster
- Complex effects: 1.5-2x faster
- 4K rendering: <5ms per frame
- Memory usage: ~1.1x input size

## Testing

```bash
cargo test --package ass-renderer --all-features
```

## License

MIT OR Apache-2.0