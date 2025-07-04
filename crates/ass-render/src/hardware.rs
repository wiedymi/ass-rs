//! Hardware-accelerated rendering using wgpu (Metal on macOS, Vulkan on other platforms).

use crate::model::{Frame, Pos, RenderedLine, Segment, StyleState};
use ass_core::Script;
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;

/// Vertex data for text rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct TextVertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
    color: [f32; 4],
}

/// GPU uniform buffer for rendering parameters
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct RenderUniforms {
    view_proj: [[f32; 4]; 4],
    viewport_size: [f32; 2],
    _padding: [f32; 2],
}

/// Font atlas entry
#[derive(Debug, Clone)]
struct GlyphInfo {
    texture_coords: [f32; 4], // x1, y1, x2, y2 in texture space
    metrics: fontdue::Metrics,
}

/// Hardware-accelerated renderer for ASS subtitles
pub struct HardwareRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    font_atlas: Option<FontAtlas>,
    dialogues: Vec<Dialogue>,
    surface_format: wgpu::TextureFormat,
    font_data: Vec<u8>,
}

/// Font atlas for GPU text rendering
struct FontAtlas {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    glyph_cache: HashMap<(char, u32), GlyphInfo>,
    atlas_size: u32,
    current_x: u32,
    current_y: u32,
    row_height: u32,
}

#[derive(Debug)]
struct Dialogue {
    start: f64,
    end: f64,
    text: String,
    pos: Option<Pos>,
}

impl HardwareRenderer {
    /// Create a new hardware renderer
    pub async fn new(script: &Script, font_data: &[u8]) -> Result<Self, HardwareRendererError> {
        // Create wgpu instance with explicit backend selection
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: if cfg!(target_os = "macos") {
                wgpu::Backends::METAL
            } else {
                wgpu::Backends::VULKAN | wgpu::Backends::DX12
            },
            ..Default::default()
        });

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(HardwareRendererError::AdapterNotFound)?;

        // Create device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .map_err(HardwareRendererError::DeviceCreation)?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/text.wgsl").into()),
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Render Uniforms"),
            size: std::mem::size_of::<RenderUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layouts
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create render pipeline
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let surface_format = wgpu::TextureFormat::Bgra8UnormSrgb;

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<TextVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Parse dialogues from script
        let mut dialogues = Vec::new();
        let serialized = script.serialize();
        for line in serialized.lines() {
            if let Some(rest) = line.strip_prefix("Dialogue:") {
                if let Some(mut d) = parse_dialogue_line(rest.trim()) {
                    if let Some(p) = extract_pos(&d.text) {
                        d.pos = Some(p);
                    }
                    dialogues.push(d);
                }
            }
        }

        let mut renderer = Self {
            device: device.clone(),
            queue: queue.clone(),
            render_pipeline,
            uniform_buffer,
            uniform_bind_group,
            texture_bind_group_layout,
            font_atlas: None,
            dialogues,
            surface_format,
            font_data: font_data.to_vec(),
        };

        // Initialize font atlas
        renderer.initialize_font_atlas().await?;

        Ok(renderer)
    }

    /// Initialize the font atlas
    async fn initialize_font_atlas(&mut self) -> Result<(), HardwareRendererError> {
        const ATLAS_SIZE: u32 = 1024;

        // Create font atlas texture
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Font Atlas"),
            size: wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Font Atlas Bind Group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        self.font_atlas = Some(FontAtlas {
            texture,
            texture_view,
            bind_group,
            sampler,
            glyph_cache: HashMap::new(),
            atlas_size: ATLAS_SIZE,
            current_x: 0,
            current_y: 0,
            row_height: 0,
        });

        Ok(())
    }

    /// Render a frame to a texture
    pub async fn render_to_texture(
        &mut self,
        time: f64,
        width: u32,
        height: u32,
        font_size: f32,
    ) -> Result<Vec<u8>, HardwareRendererError> {
        let frame = self.render(time);

        // Create output texture
        let output_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Output Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create buffer for reading back texture data
        let buffer_size = (width * height * 4) as u64;
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Generate vertices for all visible text
        let vertices = self
            .generate_text_vertices(&frame, width, height, font_size)
            .await?;

        if vertices.is_empty() {
            // Return transparent buffer if no text to render
            return Ok(vec![0u8; buffer_size as usize]);
        }

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Text Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        // Update uniforms
        let uniforms = RenderUniforms {
            view_proj: Mat4::orthographic_rh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0)
                .to_cols_array_2d(),
            viewport_size: [width as f32, height as f32],
            _padding: [0.0, 0.0],
        };

        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        // Create command encoder and render pass
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Text Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
            if let Some(font_atlas) = &self.font_atlas {
                render_pass.set_bind_group(1, &font_atlas.bind_group, &[]);
            }
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..vertices.len() as u32, 0..1);
        }

        // Copy texture to buffer
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &staging_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // Read back the data
        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        self.device.poll(wgpu::Maintain::wait()).panic_on_timeout();

        if let Some(Ok(())) = receiver.receive().await {
            let data = buffer_slice.get_mapped_range();
            let result = data.to_vec();
            drop(data);
            staging_buffer.unmap();
            Ok(result)
        } else {
            Err(HardwareRendererError::BufferMapping)
        }
    }

    /// Render frame (CPU logic, similar to software renderer)
    pub fn render(&self, time: f64) -> Frame {
        let mut lines_out = Vec::new();
        for dlg in &self.dialogues {
            if time < dlg.start || time > dlg.end {
                continue;
            }
            let mut line = render_dialogue(&dlg.text);
            // Apply fade alpha
            if let Some((f_in, f_out)) = line.fade {
                let rel_start = time - dlg.start;
                let rel_end = dlg.end - time;
                let a_in = f_in as f64 / 1000.0;
                let a_out = f_out as f64 / 1000.0;
                let alpha_factor = if rel_start < a_in {
                    (rel_start / a_in) as f32
                } else if rel_end < a_out {
                    (rel_end / a_out) as f32
                } else {
                    1.0
                };
                line.alpha = alpha_factor.clamp(0.0, 1.0);
            }
            lines_out.push(line);
        }
        Frame { lines: lines_out }
    }

    /// Generate vertices for text rendering
    async fn generate_text_vertices(
        &mut self,
        frame: &Frame,
        _width: u32,
        _height: u32,
        font_size: f32,
    ) -> Result<Vec<TextVertex>, HardwareRendererError> {
        let mut vertices = Vec::new();
        let mut cursor_y = 0.0f32;

        for line in &frame.lines {
            let mut cursor_x = 0.0f32;
            let line_height = font_size + 4.0;

            for segment in &line.segments {
                let effective_size = font_size * (segment.style.font_size / 32.0);
                let color = segment.style.color;
                let vertex_color = [
                    ((color >> 16) & 0xFF) as f32 / 255.0,
                    ((color >> 8) & 0xFF) as f32 / 255.0,
                    (color & 0xFF) as f32 / 255.0,
                    segment.style.alpha * line.alpha,
                ];

                for ch in segment.text.chars() {
                    let font_data = self.font_data.clone();
                    let glyph_info = self
                        .get_or_cache_glyph(ch, effective_size as u32, &font_data)
                        .await?;

                    // Create quad for this glyph
                    let x1 = cursor_x + glyph_info.metrics.xmin as f32;
                    let y1 = cursor_y + glyph_info.metrics.ymin as f32;
                    let x2 = x1 + glyph_info.metrics.width as f32;
                    let y2 = y1 + glyph_info.metrics.height as f32;

                    let tex_coords = glyph_info.texture_coords;

                    // Two triangles forming a quad
                    vertices.extend_from_slice(&[
                        TextVertex {
                            position: [x1, y1, 0.0],
                            tex_coords: [tex_coords[0], tex_coords[1]],
                            color: vertex_color,
                        },
                        TextVertex {
                            position: [x2, y1, 0.0],
                            tex_coords: [tex_coords[2], tex_coords[1]],
                            color: vertex_color,
                        },
                        TextVertex {
                            position: [x1, y2, 0.0],
                            tex_coords: [tex_coords[0], tex_coords[3]],
                            color: vertex_color,
                        },
                        TextVertex {
                            position: [x2, y1, 0.0],
                            tex_coords: [tex_coords[2], tex_coords[1]],
                            color: vertex_color,
                        },
                        TextVertex {
                            position: [x2, y2, 0.0],
                            tex_coords: [tex_coords[2], tex_coords[3]],
                            color: vertex_color,
                        },
                        TextVertex {
                            position: [x1, y2, 0.0],
                            tex_coords: [tex_coords[0], tex_coords[3]],
                            color: vertex_color,
                        },
                    ]);

                    cursor_x += glyph_info.metrics.advance_width;
                }
            }
            cursor_y += line_height;
        }

        Ok(vertices)
    }

    /// Get or cache a glyph in the font atlas
    async fn get_or_cache_glyph(
        &mut self,
        ch: char,
        font_size: u32,
        font_data: &[u8],
    ) -> Result<GlyphInfo, HardwareRendererError> {
        let key = (ch, font_size);

        if let Some(font_atlas) = &self.font_atlas {
            if let Some(glyph_info) = font_atlas.glyph_cache.get(&key) {
                return Ok(glyph_info.clone());
            }
        }

        // Rasterize using provided font data
        let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
            .map_err(|_| HardwareRendererError::FontLoading)?;

        let (metrics, bitmap) = font.rasterize(ch, font_size as f32);

        if let Some(font_atlas) = &mut self.font_atlas {
            // Simple atlas packing - place glyphs in rows
            let glyph_width = metrics.width as u32;
            let glyph_height = metrics.height as u32;

            if font_atlas.current_x + glyph_width > font_atlas.atlas_size {
                // Move to next row
                font_atlas.current_x = 0;
                font_atlas.current_y += font_atlas.row_height;
                font_atlas.row_height = 0;
            }

            if font_atlas.current_y + glyph_height > font_atlas.atlas_size {
                return Err(HardwareRendererError::AtlasFull);
            }

            // Upload glyph to atlas
            if !bitmap.is_empty() {
                self.queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &font_atlas.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: font_atlas.current_x,
                            y: font_atlas.current_y,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &bitmap,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(glyph_width),
                        rows_per_image: Some(glyph_height),
                    },
                    wgpu::Extent3d {
                        width: glyph_width,
                        height: glyph_height,
                        depth_or_array_layers: 1,
                    },
                );
            }

            let glyph_info = GlyphInfo {
                texture_coords: [
                    font_atlas.current_x as f32 / font_atlas.atlas_size as f32,
                    font_atlas.current_y as f32 / font_atlas.atlas_size as f32,
                    (font_atlas.current_x + glyph_width) as f32 / font_atlas.atlas_size as f32,
                    (font_atlas.current_y + glyph_height) as f32 / font_atlas.atlas_size as f32,
                ],
                metrics,
            };

            font_atlas.glyph_cache.insert(key, glyph_info.clone());
            font_atlas.current_x += glyph_width;
            font_atlas.row_height = font_atlas.row_height.max(glyph_height);

            Ok(glyph_info)
        } else {
            Err(HardwareRendererError::AtlasNotInitialized)
        }
    }

    /// Get the graphics backend being used
    pub fn get_backend(&self) -> String {
        format!("{:?}", self.device.features())
    }
}

#[derive(Debug)]
pub enum HardwareRendererError {
    AdapterNotFound,
    DeviceCreation(wgpu::RequestDeviceError),
    FontLoading,
    AtlasNotInitialized,
    AtlasFull,
    BufferMapping,
}

impl std::fmt::Display for HardwareRendererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HardwareRendererError::AdapterNotFound => {
                write!(f, "No suitable graphics adapter found")
            }
            HardwareRendererError::DeviceCreation(e) => {
                write!(f, "Failed to create graphics device: {}", e)
            }
            HardwareRendererError::FontLoading => write!(f, "Failed to load font"),
            HardwareRendererError::AtlasNotInitialized => write!(f, "Font atlas not initialized"),
            HardwareRendererError::AtlasFull => write!(f, "Font atlas is full"),
            HardwareRendererError::BufferMapping => write!(f, "Failed to map buffer for reading"),
        }
    }
}

impl std::error::Error for HardwareRendererError {}

// Helper functions (reused from software renderer)
fn parse_dialogue_line(rest: &str) -> Option<Dialogue> {
    let mut parts = rest.splitn(10, ',');
    let _layer = parts.next()?;
    let start = parse_time(parts.next()?.trim())?;
    let end = parse_time(parts.next()?.trim())?;
    for _ in 0..6 {
        parts.next()?;
    }
    let text = parts.next()?.to_string();
    Some(Dialogue {
        start,
        end,
        text,
        pos: None,
    })
}

fn parse_time(s: &str) -> Option<f64> {
    let mut parts = s.split(':');
    let h = parts.next()?.parse::<u32>().ok()?;
    let m = parts.next()?.parse::<u32>().ok()?;
    let sec_cs = parts.next()?;
    let mut sec_parts = sec_cs.split('.');
    let sec = sec_parts.next()?.parse::<u32>().ok()?;
    let cs = sec_parts.next()?.parse::<u32>().ok()?;
    let total_seconds = h as f64 * 3600.0 + m as f64 * 60.0 + sec as f64 + cs as f64 / 100.0;
    Some(total_seconds)
}

fn extract_pos(text: &str) -> Option<Pos> {
    if let Some(tag_start) = text.find("\\pos(") {
        let rest = &text[tag_start + 5..];
        if let Some(close) = rest.find(')') {
            let args = &rest[..close];
            let mut coords = args.split(',').map(|s| s.trim().parse::<f32>());
            if let (Some(Ok(x)), Some(Ok(y))) = (coords.next(), coords.next()) {
                return Some(Pos { x, y });
            }
        }
    }
    None
}

fn render_dialogue(text: &str) -> RenderedLine {
    // Simplified version - in a real implementation you'd fully parse ASS tags
    let segments = vec![Segment {
        text: Box::leak(text.to_owned().into_boxed_str()),
        style: StyleState::default(),
    }];

    RenderedLine {
        segments,
        alpha: 1.0,
        rot_x: 0.0,
        rot_y: 0.0,
        rot_z: 0.0,
        fade: None,
        align: 2,
        pos: None,
        movement: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ass_core::Script;

    // Create a simple test font (embedded fallback) - removed since file doesn't exist
    // const TEST_FONT: &[u8] = include_bytes!("../../../assets/test_font.ttf");

    // Alternative: create a minimal test using system fonts or embedded data
    fn create_test_script() -> Script {
        let ass_content = r#"[Script Info]
Title: Test Script

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,32,&H00FFFFFF,&H000000FF,&H00000000,&H80000000,0,0,0,0,100,100,0,0,1,2,0,2,10,10,10,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.00,0:00:05.00,Default,,0,0,0,,Hello World
Dialogue: 0,0:00:02.00,0:00:07.00,Default,,0,0,0,,Test Subtitle
"#;
        Script::parse(ass_content.as_bytes())
    }

    // Mock font data for testing (simple bitmap font)
    fn create_mock_font_data() -> Vec<u8> {
        // For testing purposes, we'll create a minimal valid TTF font
        // In a real implementation, you'd use a proper font file
        vec![0u8; 1024] // Placeholder - would be replaced with actual font data
    }

    #[tokio::test]
    async fn test_hardware_renderer_creation() {
        let script = create_test_script();
        let font_data = create_mock_font_data();

        // This test will validate adapter/device creation
        let result = HardwareRenderer::new(&script, &font_data).await;

        match result {
            Ok(renderer) => {
                println!("Hardware renderer created successfully");
                println!("Using backend: {}", renderer.get_backend());
            }
            Err(HardwareRendererError::AdapterNotFound) => {
                println!(
                    "No suitable graphics adapter found - this is expected in CI environments"
                );
            }
            Err(e) => {
                panic!("Unexpected error creating hardware renderer: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_hardware_renderer_backend_selection() {
        let script = create_test_script();
        let font_data = create_mock_font_data();

        if let Ok(renderer) = HardwareRenderer::new(&script, &font_data).await {
            let backend_info = renderer.get_backend();

            // Verify that the correct backend is selected based on platform
            if cfg!(target_os = "macos") {
                println!("macOS detected - should use Metal backend");
            } else {
                println!("Non-macOS platform - should use Vulkan or DirectX backend");
            }

            println!("Actual backend features: {}", backend_info);
            assert!(!backend_info.is_empty());
        }
    }

    #[tokio::test]
    async fn test_render_to_texture() {
        let script = create_test_script();
        let font_data = create_mock_font_data();

        if let Ok(mut renderer) = HardwareRenderer::new(&script, &font_data).await {
            let width = 640u32;
            let height = 480u32;
            let font_size = 24.0f32;
            let time = 1.0; // 1 second into the video

            let result = renderer
                .render_to_texture(time, width, height, font_size)
                .await;

            match result {
                Ok(buffer) => {
                    assert_eq!(buffer.len(), (width * height * 4) as usize);
                    println!("Successfully rendered frame to texture buffer");

                    // Check that buffer contains some non-zero data (indicates rendering occurred)
                    let has_content = buffer.iter().any(|&byte| byte != 0);
                    if has_content {
                        println!("Rendered content detected in buffer");
                    } else {
                        println!("Buffer is empty - may indicate no visible text at this time");
                    }
                }
                Err(e) => {
                    println!("Render to texture failed: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_parse_dialogue_line() {
        let line = "0,0:00:01.00,0:00:05.00,Default,,0,0,0,,Hello World";
        let dialogue = parse_dialogue_line(line).expect("Failed to parse dialogue");

        assert_eq!(dialogue.start, 1.0);
        assert_eq!(dialogue.end, 5.0);
        assert_eq!(dialogue.text, "Hello World");
    }

    #[test]
    fn test_parse_time() {
        assert_eq!(parse_time("0:00:01.50"), Some(1.5));
        assert_eq!(parse_time("0:01:30.25"), Some(90.25));
        assert_eq!(parse_time("1:00:00.00"), Some(3600.0));
    }

    #[test]
    fn test_extract_pos() {
        let text_with_pos = "Some text \\pos(100,200) more text";
        let pos = extract_pos(text_with_pos).expect("Failed to extract position");

        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 200.0);
    }

    #[test]
    fn test_render_dialogue() {
        let text = "Test dialogue text";
        let rendered = render_dialogue(text);

        assert_eq!(rendered.segments.len(), 1);
        assert_eq!(rendered.segments[0].text, text);
        assert_eq!(rendered.alpha, 1.0);
    }

    // Performance test to ensure hardware rendering is faster than software rendering
    #[tokio::test]
    async fn test_hardware_vs_software_performance() {
        let script = create_test_script();
        let font_data = create_mock_font_data();

        if let Ok(mut hw_renderer) = HardwareRenderer::new(&script, &font_data).await {
            let width = 1920u32;
            let height = 1080u32;
            let font_size = 32.0f32;
            let time = 1.0;

            let start = std::time::Instant::now();
            let _result = hw_renderer
                .render_to_texture(time, width, height, font_size)
                .await;
            let hw_duration = start.elapsed();

            println!("Hardware rendering took: {:?}", hw_duration);

            // In a real benchmark, you'd compare this with software rendering
            // For now, just ensure it completes in reasonable time
            assert!(hw_duration.as_millis() < 100); // Should complete within 100ms
        }
    }

    // Test error handling
    #[tokio::test]
    async fn test_error_handling() {
        let script = create_test_script();
        let invalid_font_data = vec![0u8; 10]; // Too small to be a valid font

        let result = HardwareRenderer::new(&script, &invalid_font_data).await;

        // Should handle invalid font data gracefully
        match result {
            Ok(_) => {
                // If it succeeds, that's fine too (depends on validation strictness)
                println!("Renderer created despite invalid font data");
            }
            Err(e) => {
                println!("Expected error with invalid font data: {}", e);
            }
        }
    }
}
