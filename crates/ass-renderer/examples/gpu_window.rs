//! Real-time windowed demo: present the resident GPU subtitle layer to a window.
//!
//! This is the on-screen counterpart of `gpu_playback`. Subtitle tiles are still
//! rasterized on the CPU by a software [`Renderer`], composited once into the
//! hybrid backend's resident GPU layer ([`Compositor::render_layer`]) whenever the
//! active subtitle changes, and then blended over an animated background straight
//! into the window's swapchain texture ([`Compositor::present_to_view`]) every
//! frame — no readback, no per-frame re-rasterize while the subtitle is static.
//!
//! The window surface is configured with a **non-sRGB** `*Unorm` format so the
//! premultiplied, already-sRGB-encoded layer bytes are presented unchanged; an
//! sRGB surface would re-encode them and wash the colours out.
//!
//! A time-varying clear colour animates behind the crisp subtitles so the
//! resident-layer reuse is visible: the background moves every frame while the
//! cached subtitle is presented untouched.
//!
//! Usage:
//! ```text
//! cargo run --release -p ass-renderer --no-default-features \
//!     --features full,window --example gpu_window -- \
//!     --ass FILE --size 1280x720 --frames 120
//! ```
//! With `--frames N` the demo renders `N` frames, prints the average FPS and
//! ms/frame, and exits; without it, it runs until the window is closed. By
//! default it presents with vsync (`Fifo`, capping FPS at the display refresh);
//! pass `--no-vsync` to present with `Immediate` (uncapped) to expose the raw
//! per-frame throughput when the GPU compositing — not the display — is the limit.
//!
//! Playback is interactive: a controllable clock drives `position_cs` from the
//! real per-frame delta, and the keyboard seeks, pauses, and scales speed. The
//! key bindings are printed once at startup (Left/Right seek, Space pause,
//! Up/Down speed, R/Home restart, Esc quit).

use std::sync::Arc;
use std::time::Instant;

use ass_core::parser::{Event, Script, Section};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event as WinitEvent, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowBuilder};

use ass_renderer::backends::gpu::{Background, Compositor, PresentTarget};
use ass_renderer::backends::BackendType;
use ass_renderer::renderer::{EventSelector, RenderContext, Renderer};
use ass_renderer::RenderError;

/// Parsed command-line configuration.
struct Cfg {
    /// Path to the `.ass` script to play.
    ass: String,
    /// Requested window width in physical pixels.
    width: u32,
    /// Requested window height in physical pixels.
    height: u32,
    /// Optional self-terminating frame budget (`--frames N`).
    frames: Option<u32>,
    /// Present with vsync (`Fifo`); `--no-vsync` selects `Immediate` (uncapped).
    vsync: bool,
}

/// Parse `--ass FILE`, `--size WxH` and `--frames N`, defaulting to the bundled
/// benchmark script at 1280x720 and an open-ended run.
fn parse_cfg() -> Result<Cfg, String> {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut cfg = Cfg {
        ass: concat!(env!("CARGO_MANIFEST_DIR"), "/benches/benchmark.ass").to_string(),
        width: 1280,
        height: 720,
        frames: None,
        vsync: true,
    };
    let mut i = 0;
    let val = |argv: &[String], i: &mut usize| -> Result<String, String> {
        *i += 1;
        argv.get(*i)
            .cloned()
            .ok_or_else(|| format!("missing value for {}", argv[*i - 1]))
    };
    while i < argv.len() {
        match argv[i].as_str() {
            "--ass" => cfg.ass = val(&argv, &mut i)?,
            "--size" => {
                let v = val(&argv, &mut i)?;
                let (w, h) = v.split_once('x').ok_or("bad --size (expected WxH)")?;
                cfg.width = w.parse().map_err(|_| "bad width")?;
                cfg.height = h.parse().map_err(|_| "bad height")?;
            }
            "--frames" => {
                cfg.frames = Some(val(&argv, &mut i)?.parse().map_err(|_| "bad --frames")?);
            }
            "--no-vsync" => cfg.vsync = false,
            other => return Err(format!("unknown arg {other}")),
        }
        i += 1;
    }
    if cfg.width == 0 || cfg.height == 0 {
        return Err("--size must be non-zero".into());
    }
    Ok(cfg)
}

/// Mirror of `Renderer::event_is_animated`: text whose output changes between
/// frames, so it must never be served from the static resident-layer cache.
fn event_is_animated(text: &str) -> bool {
    text.contains("\\t")
        || text.contains("\\move")
        || text.contains("\\fad")
        || text.contains("\\k")
        || text.contains("\\K")
}

/// The static-frame cache key: `None` when any active event is animated (never
/// cacheable, so every frame re-renders), else the active events' text spans as
/// `(ptr, len)` pairs — identical spans mean identical output, hence a
/// present-only cache hit.
fn cache_key(events: &[&Event]) -> Option<Vec<(usize, usize)>> {
    if events.iter().any(|e| event_is_animated(e.text)) {
        return None;
    }
    Some(
        events
            .iter()
            .map(|e| (e.text.as_ptr() as usize, e.text.len()))
            .collect(),
    )
}

/// Earliest start and latest end (centiseconds) across the script's events, used
/// to loop playback. Falls back to a 10-second span when no timed events parse.
fn clip_span(script: &Script) -> (u32, u32) {
    let mut start = u32::MAX;
    let mut end = 0u32;
    for section in script.sections() {
        if let Section::Events(events) = section {
            for ev in events {
                if let Ok(s) = ev.start_time_cs() {
                    start = start.min(s);
                }
                if let Ok(e) = ev.end_time_cs() {
                    end = end.max(e);
                }
            }
        }
    }
    if start == u32::MAX || end <= start {
        (0, 1000)
    } else {
        (start, end)
    }
}

/// Pick a non-sRGB `*Unorm` surface format so the premultiplied, sRGB-encoded
/// layer bytes present unchanged. Prefers `Bgra8Unorm`, then `Rgba8Unorm`, then
/// any other non-sRGB format, and only falls back to the first reported format
/// (which may re-encode) as a last resort.
fn pick_surface_format(caps: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat {
    use wgpu::TextureFormat::{Bgra8Unorm, Rgba8Unorm};
    if caps.formats.contains(&Bgra8Unorm) {
        Bgra8Unorm
    } else if caps.formats.contains(&Rgba8Unorm) {
        Rgba8Unorm
    } else if let Some(&format) = caps.formats.iter().find(|f| !f.is_srgb()) {
        format
    } else {
        caps.formats[0]
    }
}

/// A smoothly cycling opaque clear colour, driven by elapsed seconds so motion is
/// visible behind the static subtitle layer.
fn background_color(secs: f64) -> wgpu::Color {
    wgpu::Color {
        r: 0.5 + 0.5 * (secs * 0.70).sin(),
        g: 0.5 + 0.5 * (secs * 0.90 + 2.094).sin(),
        b: 0.5 + 0.5 * (secs * 1.10 + 4.188).sin(),
        a: 1.0,
    }
}

/// Wrap `value` into `[0, span)`, mapping negative inputs forward, so a seek past
/// either clip boundary loops cleanly. `span` is assumed positive.
fn wrap(value: f64, span: f64) -> f64 {
    value.rem_euclid(span)
}

/// Format a centisecond playback position as `mm:ss.cs` for the window title.
fn format_timecode(position_cs: f64) -> String {
    let cs = position_cs.max(0.0) as u64;
    let total_secs = cs / 100;
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    let centis = cs % 100;
    format!("{minutes:02}:{seconds:02}.{centis:02}")
}

/// All persistent state the windowed render loop drives each frame.
struct Demo {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    format: wgpu::TextureFormat,
    present_mode: wgpu::PresentMode,
    compositor: Compositor,
    renderer: Renderer,
    gate: EventSelector,
    script: Script<'static>,
    width: u32,
    height: u32,
    clip_start: u32,
    clip_end: u32,
    last_key: Option<Vec<(usize, usize)>>,
    /// Current playback position in centiseconds, advanced by the real per-frame
    /// delta (scaled by `speed`) and wrapped into `[clip_start, clip_end)`.
    position_cs: f64,
    /// Whether the clock is paused (real time still elapses, position does not).
    paused: bool,
    /// Playback rate multiplier, clamped to `[0.25, 4.0]`.
    speed: f64,
    /// Wall-clock instant of the previous tick, for computing the frame delta.
    last_tick: Instant,
    /// Free-running seconds accumulator driving the background animation, so the
    /// clear colour keeps moving even while playback is paused or seeking.
    anim_secs: f64,
    /// Wall-clock instant the run began, for the final average-FPS report.
    run_start: Instant,
    title_timer: Instant,
    title_frames: u32,
    frames: u64,
    misses: u64,
    frames_target: Option<u32>,
}

impl Demo {
    /// Initialise wgpu against `window`'s surface, build the public [`Compositor`]
    /// on that device, and a software [`Renderer`] sized to the window.
    fn new(
        window: Arc<Window>,
        script: Script<'static>,
        frames_target: Option<u32>,
        vsync: bool,
    ) -> Result<Self, String> {
        let size = window.inner_size();
        let (width, height) = (size.width.max(1), size.height.max(1));

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| format!("create_surface: {e}"))?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| "no wgpu adapter compatible with the window surface".to_string())?;
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("ass-gpu-window-device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .map_err(|e| format!("request_device: {e}"))?;

        let caps = surface.get_capabilities(&adapter);
        if caps.formats.is_empty() {
            return Err("window surface reports no supported formats".into());
        }
        let format = pick_surface_format(&caps);
        if format.is_srgb() {
            eprintln!(
                "warning: only an sRGB surface format ({format:?}) is available; \
                 subtitle colours may be re-encoded and washed out"
            );
        }
        let present_mode = if vsync {
            wgpu::PresentMode::Fifo
        } else if caps.present_modes.contains(&wgpu::PresentMode::Immediate) {
            wgpu::PresentMode::Immediate
        } else if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else {
            eprintln!("warning: --no-vsync requested but only Fifo is available");
            wgpu::PresentMode::Fifo
        };
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode,
            alpha_mode: caps
                .alpha_modes
                .first()
                .copied()
                .unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let compositor = Compositor::new(&device);
        let renderer = Renderer::new(BackendType::Software, RenderContext::new(width, height))
            .map_err(|e| format!("software renderer: {e}"))?;
        let (clip_start, clip_end) = clip_span(&script);

        let now = Instant::now();
        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            format,
            present_mode,
            compositor,
            renderer,
            gate: EventSelector::new(),
            script,
            width,
            height,
            clip_start,
            clip_end,
            last_key: None,
            position_cs: f64::from(clip_start),
            paused: false,
            speed: 1.0,
            last_tick: now,
            anim_secs: 0.0,
            run_start: now,
            title_timer: now,
            title_frames: 0,
            frames: 0,
            misses: 0,
            frames_target,
        })
    }

    /// Reconfigure the surface for a new physical size and rebuild the software
    /// renderer so its tiles (and thus the resident layer) match the new size.
    fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        if width == 0 || height == 0 {
            return Ok(());
        }
        self.width = width;
        self.height = height;
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.renderer = Renderer::new(BackendType::Software, RenderContext::new(width, height))
            .map_err(|e| format!("resize renderer: {e}"))?;
        self.gate = EventSelector::new();
        self.last_key = None;
        Ok(())
    }

    /// The loop span in centiseconds, never less than one to avoid a zero divisor.
    fn span_cs(&self) -> f64 {
        f64::from((self.clip_end - self.clip_start).max(1))
    }

    /// Seek `delta_cs` centiseconds (negative rewinds), wrapping within the clip so
    /// seeking past either boundary loops around.
    fn seek(&mut self, delta_cs: f64) {
        let start = f64::from(self.clip_start);
        self.position_cs = start + wrap(self.position_cs - start + delta_cs, self.span_cs());
    }

    /// Apply a pressed key to the playback clock. `Escape` is handled by the caller
    /// (it needs the event loop to print the report and quit), so it is a no-op
    /// here.
    fn on_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::ArrowLeft => self.seek(-500.0),
            KeyCode::ArrowRight => self.seek(500.0),
            KeyCode::Space => {
                self.paused = !self.paused;
                self.last_tick = Instant::now();
            }
            KeyCode::ArrowUp => self.speed = (self.speed * 2.0).min(4.0),
            KeyCode::ArrowDown => self.speed = (self.speed / 2.0).max(0.25),
            KeyCode::KeyR | KeyCode::Home => {
                self.position_cs = f64::from(self.clip_start);
                self.paused = false;
                self.last_tick = Instant::now();
            }
            _ => {}
        }
    }

    /// Render one frame: rebuild the resident layer only on a cache miss, then
    /// present it over the animated background straight into the swapchain.
    fn redraw(&mut self) -> Result<(), RenderError> {
        let (w, h) = (self.width, self.height);
        if w == 0 || h == 0 {
            return Ok(());
        }
        let now = Instant::now();
        let dt = (now - self.last_tick).as_secs_f64();
        self.last_tick = now;
        self.anim_secs += dt;
        if !self.paused {
            self.position_cs += dt * 100.0 * self.speed;
        }
        let start = f64::from(self.clip_start);
        self.position_cs = start + wrap(self.position_cs - start, self.span_cs());
        let t = self.position_cs as u32;

        let key = cache_key(&self.gate.select_active(&self.script, t)?.events);
        let hit = key.is_some() && self.last_key == key;
        if !hit {
            let tiles = self.renderer.render_frame_bitmaps(&self.script, t)?;
            self.compositor
                .render_layer(&self.device, &self.queue, &tiles, w, h)?;
            self.misses += 1;
        }
        self.last_key = key;

        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return Ok(());
            }
            Err(wgpu::SurfaceError::Timeout) => return Ok(()),
            Err(wgpu::SurfaceError::OutOfMemory) => {
                return Err(RenderError::BackendError("surface out of memory".into()));
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.compositor.present_to_view(
            &self.device,
            &self.queue,
            PresentTarget {
                view: &view,
                format: self.format,
            },
            Background::Clear(background_color(self.anim_secs)),
            w,
            h,
        )?;
        frame.present();

        self.frames += 1;
        self.title_frames += 1;
        self.update_title();
        Ok(())
    }

    /// Refresh the window title with a rolling FPS/ms-per-frame average roughly
    /// three times a second.
    fn update_title(&mut self) {
        let dt = self.title_timer.elapsed().as_secs_f64();
        if dt < 0.3 {
            return;
        }
        let fps = f64::from(self.title_frames) / dt;
        let ms = dt * 1000.0 / f64::from(self.title_frames.max(1));
        let tc = format_timecode(self.position_cs);
        let state = if self.paused { "paused" } else { "▶" };
        let speed = self.speed;
        let (w, h) = (self.width, self.height);
        self.window.set_title(&format!(
            "ass GPU window — {tc} {state} x{speed:.2} — {w}x{h} — \
             {fps:.1} fps ({ms:.2} ms/frame)"
        ));
        self.title_timer = Instant::now();
        self.title_frames = 0;
    }

    /// Whether the `--frames N` budget has been reached.
    fn finished(&self) -> bool {
        self.frames_target
            .is_some_and(|n| self.frames >= u64::from(n))
    }

    /// Print the overall average FPS, ms/frame and layer-rebuild count.
    fn report(&self) {
        let secs = self.run_start.elapsed().as_secs_f64();
        let fps = if secs > 0.0 {
            self.frames as f64 / secs
        } else {
            0.0
        };
        let ms = if self.frames > 0 {
            secs * 1000.0 / self.frames as f64
        } else {
            0.0
        };
        println!(
            "rendered {} frames in {secs:.2}s -> avg {fps:.1} fps ({ms:.3} ms/frame), \
             {} layer rebuild(s)",
            self.frames, self.misses
        );
    }
}

fn run() -> Result<(), String> {
    let cfg = parse_cfg()?;
    let text: &'static str = Box::leak(
        std::fs::read_to_string(&cfg.ass)
            .map_err(|e| format!("read ass: {e}"))?
            .into_boxed_str(),
    );
    let script = Script::parse(text).map_err(|e| format!("parse ass: {e:?}"))?;

    let event_loop = EventLoop::new().map_err(|e| format!("event loop: {e}"))?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("ass GPU window")
            .with_inner_size(PhysicalSize::new(cfg.width, cfg.height))
            .build(&event_loop)
            .map_err(|e| format!("create window: {e}"))?,
    );

    let mut demo = Demo::new(window, script, cfg.frames, cfg.vsync)?;
    println!(
        "GPU window playback of {} at {}x{} (surface format {:?}, {:?})",
        cfg.ass, demo.width, demo.height, demo.format, demo.present_mode
    );
    println!(
        "controls: Left/Right seek 5s, Space pause/resume, Up/Down speed (0.25x-4x), \
         R or Home restart, Esc quit"
    );

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);
            match event {
                WinitEvent::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        demo.report();
                        elwt.exit();
                    }
                    WindowEvent::Resized(size) => {
                        if let Err(e) = demo.resize(size.width, size.height) {
                            eprintln!("resize error: {e}");
                            elwt.exit();
                        }
                    }
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                physical_key: PhysicalKey::Code(code),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        demo.on_key(code);
                        if code == KeyCode::Escape {
                            demo.report();
                            elwt.exit();
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        if let Err(e) = demo.redraw() {
                            eprintln!("redraw error: {e}");
                            elwt.exit();
                        } else if demo.finished() {
                            demo.report();
                            elwt.exit();
                        }
                    }
                    _ => {}
                },
                WinitEvent::AboutToWait => demo.window.request_redraw(),
                _ => {}
            }
        })
        .map_err(|e| format!("event loop run: {e}"))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("gpu_window error: {e}");
        std::process::exit(1);
    }
}
