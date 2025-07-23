## Overview

`ass-benchmarks` is a dedicated benchmarking suite for the `ass-rs` ecosystem, providing comprehensive performance and memory profiling across `ass-core`, `ass-renderer`, `ass-editor`, and `ass-cli`. It leverages Criterion.rs for micro-benchmarks (e.g., tag parsing hot paths) and custom harnesses for end-to-end scenarios (e.g., full-script rendering at 4K), with optional comparisons to libass (via Rust bindings or external subprocesses to avoid FFI overhead). Critically, it exposes flaws in libass like CPU-bound blurs (50-100ms/frame on complex karaoke) vs. our GPU-accelerated <5ms, while highlighting our own weaknesses (e.g., WASM overhead in web backends, ~20% slower without SIMD). Designed as a reusable tool (e.g., `ass-bench parse --input-dir samples/` for CI), it's extensible via plugins for custom metrics (e.g., VRAM usage on Vulkan) and modular to isolate components (e.g., bench only shaping without full pipeline).

### Core Functionalities

- **Micro-Benchmarks**: Granular tests (e.g., tokenizer SIMD vs. scalar, \t animation interp).
- **End-to-End**: Script load → parse → analyze → render frames (time-series for animations).
- **Memory Profiling**: Heaptrack/massif integration (feature-gated), tracking peaks/leaks.
- **Comparisons**: Libass baselines (subprocess or bindings; critique its inconsistencies, e.g., color matrix bugs).
- **Reporting**: HTML/JSON outputs with plots (flamegraphs, throughput over iterations).
- **CI Integration**: Headless mode, thresholds for regressions (e.g., fail if >10% slower).

### Targets

- Run <1s/micro, <30s/full suite on CI (4-core).
- Detect regressions early (e.g., parse <5ms/1KB).
- Memory: <100MB peak for 10MB scripts; critique if arenas fail to reset.

## Key Principles

- **Performance Priority**: Criterion warmups/iterations for stable metrics; parallel groups via rayon. Target hot paths (e.g., drawing Beziers, blur convolutions) with SIMD/hardware variants. Critique naive setups (e.g., no warmups inflate variance by 50%).
- **Memory Efficiency**: Reuse pooled inputs (e.g., mmap samples), drop after each iter; feature for alloc tracking (mimalloc override). Avoid libass's leaks by design—our zero-copy shines in comparisons.
- **Modularity and Reusability**: No flat `src/`; subcrates-like modules (e.g., `benchmarks/core/parsing.rs` reusable as `pub fn bench_parse(c: &mut Criterion)`). CLI for standalone use.
- **Extensibility**: Registry for custom benches/metrics (e.g., register_webgpu_vram()); plugins load from paths.
- **Criticisms Addressed**: Libass lacks benches— we force comparisons to expose its bottlenecks (e.g., no multi-thread, poor WASM ports). Our suite lints for platform quirks (e.g., WebGPU adapter limits cap throughput).
- **Thread-Safety**: All benches Send+Sync; parallel groups default.
- **Compliance**: Use real-world samples (karaoke-heavy, embedded fonts) from specs/Aegisub; vary resolutions (720p-8K).

## Dependencies and Feature Flags

### External Dependencies

- `criterion = { version = "0.5", features = ["html_reports"] }`: Core benchmarking.
- `ass-core = { path = "../ass-core" }`, `ass-renderer = { path = "../ass-renderer" }`, etc.: Ecosystem crates.
- `rayon = "1.10"`: Parallel bench groups.
- `serde_json = "1.0"`: JSON reports.
- `heaptrack-rs = "0.1"` (or similar; feature-gated for memory).
- `valgrind = { version = "1.0", optional = true }`: Leak detection without deps.
- `libass-sys = "0.17.4"` (bindings for comparisons; updated for libass 0.17.4 June 2025 release; optional to avoid C deps).
- `wasm-bindgen-test = "0.3"` (for WASM benches).

### Feature Flags

- `core` (default): ass-core benches.
- `renderer`: ass-renderer (software/hardware/web).
- `editor`: ass-editor.
- `cli`: ass-cli.
- `libass-compare`: Subprocess/bindings for baselines.
- `libass-0.17.4-compare`: Updated baselines for libass 0.17.4 features.
- `memory`: Heap profiling (heaptrack/massif).
- `valgrind`: Leak detection via valgrind integration.
- `wasm`: WASM targets (browser/node).
- `parallel` (default): Rayon groups.
- `simd`: Enable in dependents.
- `thresholds`: Environment variable threshold configuration.
- `nostd`: Minimal (alloc-only for core micros).

Expectations: Suite <200KB; features gate heavy deps (e.g., libass +1MB). CI thresholds configurable via env vars.

## Architecture

### High-Level Flow

CLI args → Dispatcher (select modules) → Criterion setup (groups, iters) → Run benches → Collect metrics → Report (HTML/JSON/flame).

### Text-Based Diagram

```mermaid
graph TD
    A[CLI Args: --modules core,renderer --samples dir] --> B[Dispatcher: load plugins, select benches]
    B --> C[Criterion Harness: groups (micro/end2end)]
    C --> D[Modules: e.g., core_parsing(c), renderer_pipeline(c)]
    D --> E[Inputs: mmap samples, generated (fuzz-like)]
    D --> F[Metrics: throughput, memory, comparisons]
    F --> G[Reporters: HTML plots, JSON, console]
    G --> H[Outputs: benches/results/, flamegraphs]
```

- **Data Flow**: Samples dir → loaded Scripts → bench fns (e.g., |b| b.iter(|| script.parse())).
- **Error Handling**: BenchError (skip on fails); thresholds panic on regressions.
- **Optimization Hooks**: Warmups auto; plot configs for trends.
- **Platform Variants**: WASM uses browser timing; hardware probes adapters.

## Folder Structure and Modules

Root: `lib.rs` (CLI entry, re-exports `pub mod benchmarks;`).

```plaintext
crates/ass-benchmarks/
├── Cargo.toml
├── lib.rs
├── dispatcher/
│   ├── mod.rs      # BenchDispatcher { modules: Vec, plugins }
│   └── cli.rs      # Clap struct (modules, samples, iters, reports)
├── benchmarks/     # Component-specific
│   ├── mod.rs      # Registry (HashMap<Name, Box<dyn BenchFn>>)
│   ├── core/       # ass-core
│   │   ├── mod.rs  # group_core(c: &mut Criterion)
│   │   ├── parsing.rs  # bench_tokenizer, bench_ast_build
│   │   ├── analysis.rs # bench_lint, bench_resolve_styles
│   │   └── tags.rs     # Per-tag (e.g., bench_drawing_bezier)
│   ├── renderer/   # ass-renderer
│   │   ├── mod.rs  # group_renderer(c)
│   │   ├── pipeline.rs # bench_shaping, bench_effects_blur
│   │   ├── backends.rs # bench_software_raster, bench_webgpu_frame
│   │   └── compositing.rs # bench_xor_shapes
│   ├── editor/     # ass-editor
│   │   ├── mod.rs  # group_editor(c)
│   │   └── commands.rs # bench_incremental_edit, bench_search
│   └── cli/        # ass-cli
│       ├── mod.rs  # group_cli(c)
│       └── ops.rs  # bench_lint_batch, bench_convert
├── inputs/         # Sample management
│   ├── mod.rs   # SamplesLoader (dir mmap, generators for synthetic)
│   └── generators.rs # e.g., gen_karaoke(events: usize). Gen RTL/XOR stress (bidir text + 1000 overlaps); large UUencoded for heap tests
├── comparisons/    # Libass baselines
│   ├── mod.rs      # LibassRunner (subprocess or ffi)
│   ├── utils.rs     # parse_libass_times
│   └── targets.rs   # Baselines for libass 0.17.4 (June 2025); env var LIBASS_VERSION for dynamic
├── plugins/        # Extensibility
│   ├── mod.rs      # BenchPlugin trait (register_benches, custom_metrics)
│   └── metrics/     # e.g., VramMetric (wgpu query)
├── reporters/      # Outputs
│   ├── mod.rs      # ReporterTrait (console, json, html)
│   ├── html.rs      # Criterion integration + custom
│   ├── json.rs
│   └── flamegraph.rs # inferno/flamegraph-rs
└── utils/
    ├── mod.rs     # BenchUtils (thresholds, platforms)
    ├── errors.rs   # BenchError enum
    ├── thresholds.rs # Environment variable threshold parsing (BENCH_FAIL_IF_SLOWER, etc.)
    └── ci.rs      # CI integration helpers and regression detection
```

Expectations: <150 LOC/file; 90% coverage (mock Criterion); extensible for new crates (e.g., add `ass-wasm` module.

## Expectations

- **Performance**: Suite detects configurable regressions (env: `BENCH_FAIL_IF_SLOWER=10%`); micros stable <1% stddev.
- **Memory**: Profiles expose leaks via Valgrind integration (e.g., fail if >5% over baseline).
- **Testing**: Unit for loaders/reporters; integration via sample runs. 90% coverage with mocked Criterion.
- **Edge Cases**: Large scripts (10MB+), WASM limits, no-SIMD fallbacks. Browser variance >50% without warmups.
- **CI Integration**: Environment variable thresholds (BENCH_FAIL_IF_SLOWER, BENCH_MAX_MEMORY). Subprocess libass to avoid FFI overhead. Semver checks: cargo semver-checks in workflow; fail on version conflicts.
- **Future-Proof**: Plugins for new features (e.g., bench \kt extensions; compare to libass 0.17.4+ with LayoutResX/Y, AlphaLevel, Unicode wrapping features). Auto-register new crate benchmarks.
