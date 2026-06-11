// Scene construction and GPU submission for the charts demo.

use plev::compositor::{SceneNode, TextNodeKey};

use crate::charts::*;
use crate::palette::*;
use crate::state::{App, GpuState};

impl App {
    pub(crate) fn render(&mut self) {
        let tick = self.clock.tick();
        self.reveal.tick(tick.dt);
        self.frame += 1;
        self.data.animate(self.frame as f32 * 0.016);

        let (vw, vh) = match &self.gpu_state {
            GpuState::Ready { gpu, .. } => (
                gpu.surface_config.width as f32,
                gpu.surface_config.height as f32,
            ),
            _ => return,
        };

        self.compositor.begin_frame();

        // Header
        self.compositor.push(SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: vw,
            h: 60.0,
            color: [0.07, 0.07, 0.11, 1.0],
        });
        self.compositor.push(SceneNode::Rect {
            x: 0.0,
            y: 59.0,
            w: vw,
            h: 1.0,
            color: DIVIDER,
        });
        self.compositor.draw_text(
            TextNodeKey::new("DATA VISUALIZATION", 24.0, 30.0, Some(vw - 64.0)),
            24.0,
            12.0,
            TEXT,
        );
        self.compositor.draw_text(
            TextNodeKey::new(
                "Line + Bar + Area + Pie | PathBuilder tessellation",
                11.0,
                15.0,
                None,
            ),
            24.0,
            42.0,
            TEXT_DIM,
        );

        let margin = 20.0;
        let gap = 14.0;
        let half_w = (vw - margin * 2.0 - gap) / 2.0;
        let half_h = ((vh - 60.0 - 28.0 - margin * 2.0 - gap) / 2.0)
            .min(300.0)
            .max(160.0);
        let y0 = 60.0 + margin;
        let y1 = y0 + half_h + gap;
        let r = self.reveal.get();

        self.build_line_chart(margin, y0, half_w, half_h, r);
        self.build_bar_chart(margin + half_w + gap, y0, half_w, half_h, r);
        self.build_area_chart(margin, y1, half_w, half_h, r);
        self.build_pie_chart(margin + half_w + gap, y1, half_w, half_h, r);

        // Footer
        self.compositor.push(SceneNode::Rect {
            x: 0.0,
            y: vh - 28.0,
            w: vw,
            h: 28.0,
            color: [0.06, 0.06, 0.10, 1.0],
        });
        self.compositor.push(SceneNode::Rect {
            x: 0.0,
            y: vh - 28.0,
            w: vw,
            h: 1.0,
            color: DIVIDER,
        });
        self.compositor.draw_text(
            TextNodeKey::new(
                "PathBuilder (Lyon) tessellation | Animated data | All cross-platform",
                10.0,
                14.0,
                Some(vw - 48.0),
            ),
            24.0,
            vh - 20.0,
            TEXT_DIM,
        );

        self.gpu_submit();
    }

    fn build_line_chart(&mut self, cx: f32, cy: f32, half_w: f32, half_h: f32, r: f32) {
        push_shadow(&mut self.compositor, cx, cy, half_w, half_h);
        self.compositor.push(SceneNode::Rect {
            x: cx,
            y: cy,
            w: half_w,
            h: half_h,
            color: SURFACE,
        });
        self.compositor.push(SceneNode::Rect {
            x: cx,
            y: cy,
            w: half_w,
            h: 2.0,
            color: ACCENT,
        });
        self.compositor.draw_text(
            TextNodeKey::new("LINE CHART", 12.0, 16.0, None),
            cx + 12.0,
            cy + 8.0,
            ACCENT,
        );

        let chart_x = cx + 40.0;
        let chart_y = cy + 32.0;
        let chart_w = half_w - 52.0;
        let chart_h = half_h - 52.0;

        draw_grid(
            &mut self.compositor,
            chart_x,
            chart_y,
            chart_w,
            chart_h,
            4,
            6,
        );

        for i in 0..=4 {
            let val = i as f32 * 25.0;
            let ly = chart_y + chart_h - chart_h * (i as f32 / 4.0);
            let label = format!("{}", val as u32);
            self.compositor.draw_text(
                TextNodeKey::new(&label, 9.0, 12.0, Some(32.0)),
                cx + 6.0,
                ly - 5.0,
                TEXT_DIM,
            );
        }

        let visible_count = ((self.data.line_data.len() as f32) * r) as usize;
        let visible = &self.data.line_data[..visible_count.min(self.data.line_data.len()).max(2)];
        draw_line_chart(
            &mut self.compositor,
            visible,
            ACCENT,
            chart_x,
            chart_y,
            chart_w,
            chart_h,
            3.0,
        );
    }

    fn build_bar_chart(&mut self, cx: f32, cy: f32, half_w: f32, half_h: f32, r: f32) {
        push_shadow(&mut self.compositor, cx, cy, half_w, half_h);
        self.compositor.push(SceneNode::Rect {
            x: cx,
            y: cy,
            w: half_w,
            h: half_h,
            color: SURFACE,
        });
        self.compositor.push(SceneNode::Rect {
            x: cx,
            y: cy,
            w: half_w,
            h: 2.0,
            color: GREEN,
        });
        self.compositor.draw_text(
            TextNodeKey::new("BAR CHART", 12.0, 16.0, None),
            cx + 12.0,
            cy + 8.0,
            GREEN,
        );

        let chart_x = cx + 12.0;
        let chart_y = cy + 32.0;
        let chart_w = half_w - 24.0;
        let chart_h = half_h - 52.0;

        draw_grid(
            &mut self.compositor,
            chart_x,
            chart_y,
            chart_w,
            chart_h,
            4,
            0,
        );

        let bar_colors = [ACCENT, GREEN, PURPLE, CYAN, YELLOW, ORANGE, RED, PINK];
        let scaled: Vec<f32> = self.data.bar_data.iter().map(|v| v * r).collect();
        draw_bar_chart(
            &mut self.compositor,
            &scaled,
            &bar_colors,
            chart_x,
            chart_y,
            chart_w,
            chart_h,
        );

        let bar_gap = 6.0;
        let bar_w = (chart_w - bar_gap * (scaled.len() + 1) as f32) / scaled.len() as f32;
        let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug"];
        for (i, label) in months.iter().enumerate().take(scaled.len()) {
            let lx = chart_x + bar_gap + i as f32 * (bar_w + bar_gap);
            self.compositor.draw_text(
                TextNodeKey::new(label, 8.0, 11.0, Some(bar_w)),
                lx,
                chart_y + chart_h + 4.0,
                TEXT_DIM,
            );
        }
    }

    fn build_area_chart(&mut self, cx: f32, cy: f32, half_w: f32, half_h: f32, r: f32) {
        push_shadow(&mut self.compositor, cx, cy, half_w, half_h);
        self.compositor.push(SceneNode::Rect {
            x: cx,
            y: cy,
            w: half_w,
            h: half_h,
            color: SURFACE,
        });
        self.compositor.push(SceneNode::Rect {
            x: cx,
            y: cy,
            w: half_w,
            h: 2.0,
            color: PURPLE,
        });
        self.compositor.draw_text(
            TextNodeKey::new("STACKED AREA", 12.0, 16.0, None),
            cx + 12.0,
            cy + 8.0,
            PURPLE,
        );

        let chart_x = cx + 12.0;
        let chart_y = cy + 32.0;
        let chart_w = half_w - 24.0;
        let chart_h = half_h - 52.0;

        draw_grid(
            &mut self.compositor,
            chart_x,
            chart_y,
            chart_w,
            chart_h,
            4,
            5,
        );

        // Area 2 (back)
        {
            let n = self.data.area_data_1.len();
            let step = chart_w / (n - 1) as f32;
            let mut b = plev::path::PathBuilder::new();
            b = b.move_to(chart_x, chart_y + chart_h);
            for (i, v) in self.data.area_data_1.iter().enumerate() {
                b = b.line_to(
                    chart_x + i as f32 * step,
                    chart_y + chart_h - v * chart_h * r,
                );
            }
            b = b.line_to(chart_x + chart_w, chart_y + chart_h);
            self.compositor
                .draw_path(b.close().fill([PURPLE[0], PURPLE[1], PURPLE[2], 0.25]));
        }
        // Area 1 (front)
        {
            let n = self.data.area_data_2.len();
            let step = chart_w / (n - 1) as f32;
            let mut b = plev::path::PathBuilder::new();
            b = b.move_to(chart_x, chart_y + chart_h);
            for (i, v) in self.data.area_data_2.iter().enumerate() {
                b = b.line_to(
                    chart_x + i as f32 * step,
                    chart_y + chart_h - v * chart_h * r,
                );
            }
            b = b.line_to(chart_x + chart_w, chart_y + chart_h);
            self.compositor
                .draw_path(b.close().fill([CYAN[0], CYAN[1], CYAN[2], 0.3]));
        }

        // Legend
        let ly = cy + half_h - 18.0;
        self.compositor.push(SceneNode::Rect {
            x: cx + 12.0,
            y: ly,
            w: 8.0,
            h: 8.0,
            color: [PURPLE[0], PURPLE[1], PURPLE[2], 0.5],
        });
        self.compositor.draw_text(
            TextNodeKey::new("Series A", 9.0, 12.0, None),
            cx + 24.0,
            ly - 1.0,
            TEXT_DIM,
        );
        self.compositor.push(SceneNode::Rect {
            x: cx + 80.0,
            y: ly,
            w: 8.0,
            h: 8.0,
            color: [CYAN[0], CYAN[1], CYAN[2], 0.5],
        });
        self.compositor.draw_text(
            TextNodeKey::new("Series B", 9.0, 12.0, None),
            cx + 92.0,
            ly - 1.0,
            TEXT_DIM,
        );
    }

    fn build_pie_chart(&mut self, cx: f32, cy: f32, half_w: f32, half_h: f32, r: f32) {
        push_shadow(&mut self.compositor, cx, cy, half_w, half_h);
        self.compositor.push(SceneNode::Rect {
            x: cx,
            y: cy,
            w: half_w,
            h: half_h,
            color: SURFACE,
        });
        self.compositor.push(SceneNode::Rect {
            x: cx,
            y: cy,
            w: half_w,
            h: 2.0,
            color: ORANGE,
        });
        self.compositor.draw_text(
            TextNodeKey::new("PIE CHART", 12.0, 16.0, None),
            cx + 12.0,
            cy + 8.0,
            ORANGE,
        );

        let pie_cx = cx + half_w * 0.4;
        let pie_cy = cy + half_h / 2.0 + 10.0;
        let pie_r = (half_h / 2.0 - 30.0).min(80.0);

        let pie_colors = [ACCENT, GREEN, YELLOW, PURPLE, ORANGE];
        let scaled_values: Vec<f32> = self.data.pie_values.iter().map(|v| v * r).collect();
        draw_pie_chart(
            &mut self.compositor,
            &scaled_values,
            &pie_colors,
            pie_cx,
            pie_cy,
            pie_r,
        );

        push_circle(&mut self.compositor, pie_cx, pie_cy, pie_r * 0.45, SURFACE);
        self.compositor.draw_text(
            TextNodeKey::new("100%", 14.0, 18.0, Some(60.0)),
            pie_cx - 18.0,
            pie_cy - 8.0,
            TEXT,
        );

        let labels = ["Revenue", "Costs", "Growth", "R&D", "Other"];
        for (i, (label, color)) in labels.iter().zip(pie_colors.iter()).enumerate() {
            let lx = cx + half_w * 0.65;
            let ly = cy + 40.0 + i as f32 * 22.0;
            self.compositor.push(SceneNode::Rect {
                x: lx,
                y: ly + 2.0,
                w: 10.0,
                h: 10.0,
                color: *color,
            });
            let pct = format!("{} ({:.0}%)", label, self.data.pie_values[i]);
            self.compositor.draw_text(
                TextNodeKey::new(&pct, 10.0, 14.0, Some(half_w * 0.3)),
                lx + 16.0,
                ly,
                TEXT_DIM,
            );
        }
    }

    fn gpu_submit(&mut self) {
        let GpuState::Ready {
            ref mut gpu,
            ref mut text_system,
            ref mut pool,
        } = self.gpu_state
        else {
            return;
        };
        let _ = pool;
        let surface = match gpu.surface.as_ref() {
            Some(s) => s,
            None => return,
        };
        let output = match surface.get_current_texture() {
            Ok(t) => t,
            Err(plev::wgpu::SurfaceError::Lost | plev::wgpu::SurfaceError::Outdated) => {
                gpu.resize(gpu.surface_config.width, gpu.surface_config.height);
                return;
            }
            Err(_) => return,
        };
        let surface_view = gpu.surface_render_view(&output);
        text_system.begin_frame();
        self.compositor
            .resolve(&plev::compositor::ResolveResources {
                msaa_samples: gpu.config.msaa_samples,
                device: &gpu.device,
                queue: &gpu.queue,
                format: gpu.surface_format(),
                width: gpu.surface_config.width,
                height: gpu.surface_config.height,
                composite_bgl: &gpu.composite_bind_group_layout,
                opacity_bgl: &gpu.opacity_bind_group_layout,
                sampler: &gpu.composite_sampler,
            });
        {
            let layer_info: Vec<_> = self
                .compositor
                .layers()
                .iter()
                .map(|l| (l.id, l.is_dirty(), l.text_nodes()))
                .collect();
            for (layer_id, dirty, text_nodes) in layer_info {
                if !dirty {
                    continue;
                }
                let (vertices, indices) = text_system.resolve_for_layer(
                    &gpu.device,
                    &gpu.queue,
                    &gpu.text_bind_group_layout,
                    &text_nodes,
                );
                if let Some(layer) = self.compositor.layer_mut(layer_id) {
                    layer.set_text_data(&gpu.device, &gpu.queue, vertices, indices);
                }
            }
        }
        text_system.finish_frame();
        let mut encoder =
            gpu.device
                .create_command_encoder(&plev::wgpu::CommandEncoderDescriptor {
                    label: Some("charts_encoder"),
                });
        let dirty_ids: Vec<_> = self
            .compositor
            .layers()
            .iter()
            .filter(|l| l.visible && l.is_dirty())
            .map(|l| l.id)
            .collect();
        for layer_id in &dirty_ids {
            let layer = self.compositor.layer(*layer_id).unwrap();
            let Some(msaa_v) = layer.msaa_view() else {
                continue;
            };
            let resolve_v = layer.texture_view();
            let mut pass = encoder.begin_render_pass(&plev::wgpu::RenderPassDescriptor {
                label: Some("layer_pass"),
                color_attachments: &[Some(plev::wgpu::RenderPassColorAttachment {
                    view: msaa_v,
                    resolve_target: resolve_v,
                    ops: plev::wgpu::Operations {
                        load: plev::wgpu::LoadOp::Clear(plev::wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: plev::wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if let Some((vb, ib, count)) = layer.quad_buffers() {
                pass.set_pipeline(&gpu.quad_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), plev::wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }
            if let Some((vb, ib, count)) = layer.sdf_rect_buffers() {
                pass.set_pipeline(&gpu.rect_sdf_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), plev::wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }
            if let Some((vb, ib, count)) = layer.text_buffers() {
                pass.set_pipeline(&gpu.text_pipeline);
                pass.set_bind_group(0, &gpu.projection_bind_group, &[]);
                pass.set_bind_group(1, &text_system.atlas_bind_group, &[]);
                pass.set_vertex_buffer(0, vb.slice(..));
                pass.set_index_buffer(ib.slice(..), plev::wgpu::IndexFormat::Uint32);
                pass.draw_indexed(0..count, 0, 0..1);
            }
        }
        for id in &dirty_ids {
            self.compositor.mark_layer_clean(*id);
        }
        {
            let mut pass = encoder.begin_render_pass(&plev::wgpu::RenderPassDescriptor {
                label: Some("composite_pass"),
                color_attachments: &[Some(plev::wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: plev::wgpu::Operations {
                        load: plev::wgpu::LoadOp::Clear({
                            let [lr, lg, lb, la] =
                                plev::color::Color::rgb(BG[0], BG[1], BG[2]).to_linear_array();
                            plev::wgpu::Color {
                                r: lr as f64,
                                g: lg as f64,
                                b: lb as f64,
                                a: la as f64,
                            }
                        }),
                        store: plev::wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&gpu.composite_pipeline);
            for layer in self.compositor.layers() {
                if !layer.visible {
                    continue;
                }
                if let (Some(bg), Some(obg)) =
                    (layer.composite_bind_group(), layer.opacity_bind_group())
                {
                    pass.set_bind_group(0, bg, &[]);
                    pass.set_bind_group(1, obg, &[]);
                    pass.draw(0..3, 0..1);
                }
            }
        }
        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
