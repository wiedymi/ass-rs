# ass-renderer

> **⚠️ WORK IN PROGRESS** - This renderer is currently under active development. While the software backend is functional, some features may be incomplete and the API is subject to change.

High-performance ASS (Advanced SubStation Alpha) subtitle renderer with modular backend support.

## Features

- **Rendering Backends**
  - Software (CPU) rendering with tiny-skia (RECOMMENDED - fully implemented)
  - GPU hybrid compositor via `wgpu` (uploads the software backend's coverage/RGBA
    tiles and composites them on the GPU; one path covers Vulkan/Metal/DX12/GL natively)
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

### GPU Backend (`gpu` feature)
A `wgpu`-based hybrid compositor. It does **not** re-rasterize glyphs/shapes on the
GPU; instead it reuses the software backend's cached coverage/RGBA tiles, uploads them
as textures, and composites them with a textured-quad shader — so it inherits the
software backend's parity. A single `wgpu` path covers Vulkan/Metal/DX12/GL natively.

```rust
use ass_renderer::{Renderer, RenderContext, BackendType};

// Requires building with --features gpu
let mut renderer = Renderer::new(BackendType::Gpu, context)?;
```

Note: for this lightweight compositing workload the GPU path pays an upload + readback
cost and is currently slower than the already-fast CPU backend; its value is GPU offload
and a future browser (WebGPU) target. For production we recommend the Software backend.

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
- `gpu`: wgpu hybrid GPU compositor (native; requires `software-backend`)
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