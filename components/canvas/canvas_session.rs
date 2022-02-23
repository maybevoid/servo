use ferrite_session::prelude::*;

use crate::canvas_data::*;
use crate::canvas_paint_thread::{AntialiasMode, WebrenderApi};
use euclid::default::{Rect, Size2D};
use gfx::font_cache_thread::FontCacheThread;
use ipc_channel::ipc::IpcSharedMemory;
use serde_bytes::ByteBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::{task, time};

use crate::canvas_protocol::*;

fn handle_canvas_message(canvas: &mut CanvasData<'static>, message: CanvasMessage) {
    match message {
        CanvasMessage::FillText(text, x, y, max_width, style, is_rtl) => {
            canvas.set_fill_style(style);
            canvas.fill_text(text, x, y, max_width, is_rtl);
        },
        CanvasMessage::FillRect(rect, style) => {
            canvas.set_fill_style(style);
            canvas.fill_rect(&rect);
        },
        CanvasMessage::StrokeRect(rect, style) => {
            canvas.set_stroke_style(style);
            canvas.stroke_rect(&rect);
        },
        CanvasMessage::ClearRect(ref rect) => {
            canvas.clear_rect(rect);
        },
        CanvasMessage::BeginPath => canvas.begin_path(),
        CanvasMessage::ClosePath => canvas.close_path(),
        CanvasMessage::Fill(style) => {
            canvas.set_fill_style(style);
            canvas.fill();
        },
        CanvasMessage::Stroke(style) => {
            canvas.set_stroke_style(style);
            canvas.stroke();
        },
        CanvasMessage::Clip => canvas.clip(),
        CanvasMessage::DrawImage(
            imagedata,
            image_size,
            dest_rect,
            source_rect,
            smoothing_enabled,
        ) => {
            let data = imagedata.map_or_else(
                || vec![0; image_size.width as usize * image_size.height as usize * 4],
                |bytes| bytes.into_vec(),
            );
            canvas.draw_image(data, image_size, dest_rect, source_rect, smoothing_enabled)
        },
        CanvasMessage::MoveTo(ref point) => canvas.move_to(point),
        CanvasMessage::LineTo(ref point) => canvas.line_to(point),
        CanvasMessage::Rect(ref rect) => canvas.rect(rect),
        CanvasMessage::PutImageData(rect, img) => canvas.put_image_data(img.into_vec(), rect),
        CanvasMessage::QuadraticCurveTo(ref cp, ref pt) => canvas.quadratic_curve_to(cp, pt),
        CanvasMessage::BezierCurveTo(ref cp1, ref cp2, ref pt) => {
            canvas.bezier_curve_to(cp1, cp2, pt)
        },
        CanvasMessage::Arc(ref center, radius, start, end, ccw) => {
            canvas.arc(center, radius, start, end, ccw)
        },
        CanvasMessage::ArcTo(ref cp1, ref cp2, radius) => canvas.arc_to(cp1, cp2, radius),
        CanvasMessage::Ellipse(ref center, radius_x, radius_y, rotation, start, end, ccw) => {
            canvas.ellipse(center, radius_x, radius_y, rotation, start, end, ccw)
        },
        CanvasMessage::RestoreContext => canvas.restore_context_state(),
        CanvasMessage::SaveContext => canvas.save_context_state(),
        CanvasMessage::SetLineWidth(width) => canvas.set_line_width(width),
        CanvasMessage::SetLineCap(cap) => canvas.set_line_cap(cap),
        CanvasMessage::SetLineJoin(join) => canvas.set_line_join(join),
        CanvasMessage::SetMiterLimit(limit) => canvas.set_miter_limit(limit),
        CanvasMessage::SetTransform(ref matrix) => canvas.set_transform(matrix),
        CanvasMessage::SetGlobalAlpha(alpha) => canvas.set_global_alpha(alpha),
        CanvasMessage::SetGlobalComposition(op) => canvas.set_global_composition(op),
        CanvasMessage::SetShadowOffsetX(value) => canvas.set_shadow_offset_x(value),
        CanvasMessage::SetShadowOffsetY(value) => canvas.set_shadow_offset_y(value),
        CanvasMessage::SetShadowBlur(value) => canvas.set_shadow_blur(value),
        CanvasMessage::SetShadowColor(color) => canvas.set_shadow_color(color),
        CanvasMessage::SetFont(font_style) => canvas.set_font(font_style),
        CanvasMessage::SetTextAlign(text_align) => canvas.set_text_align(text_align),
        CanvasMessage::SetTextBaseline(text_baseline) => canvas.set_text_baseline(text_baseline),
        CanvasMessage::Recreate(size) => {
            canvas.recreate(size);
        },
    }
}

fn run_canvas_session(mut canvas: CanvasData<'static>) -> SharedSession<CanvasProtocol> {
    accept_shared_session(async move {
        offer_choice! {
          Message => {
            receive_value ( move | message | {
              handle_canvas_message (&mut canvas, message);
              detach_shared_session (
                run_canvas_session ( canvas )
              )
            })
          },
          Messages => {
            receive_value ( move | messages | {
              for message in messages {
                handle_canvas_message (&mut canvas, message);
              }

              detach_shared_session (
                run_canvas_session ( canvas )
              )
            })
          },
          GetTransform => {
            let transform = canvas.get_transform();
            send_value ( transform,
              detach_shared_session (
                run_canvas_session ( canvas )
              ))
          },
          GetImageData => {
            receive_value ( move | (dest_rect, canvas_size) | {
              let pixels = canvas.read_pixels(dest_rect, canvas_size);

              send_value( ByteBuf::from(pixels),
                detach_shared_session (
                  run_canvas_session ( canvas )
                ))
            })
          },
          IsPointInPath => {
            receive_value ( move | msg | {
              let (x, y, fill_rule) = msg;
              let res = canvas.is_point_in_path_bool(x, y, fill_rule);

              send_value ( res,
                detach_shared_session (
                  run_canvas_session ( canvas )
                ))
            })
          },
          FromLayout => {
            send_value ( canvas.get_data(),
              detach_shared_session (
                run_canvas_session ( canvas )
              ))
          },
          FromScript => {
            let bytes = canvas.get_pixels();
            send_value( IpcSharedMemory::from_bytes(&bytes),
              detach_shared_session (
                run_canvas_session ( canvas )
              ))
          },
        }
    })
}

struct CanvasConfig {
    webrender_api: Box<dyn WebrenderApi>,
    font_cache_thread: FontCacheThread,
}

fn run_create_canvas_session(ctx: CanvasConfig) -> SharedSession<CreateCanvasProtocol> {
    accept_shared_session(async move {
        receive_value(move |param| {
            let (size, antialias) = param;

            let antialias_mode = if antialias {
                AntialiasMode::Default
            } else {
                AntialiasMode::None
            };

            let canvas = CanvasData::new(
                size,
                ctx.webrender_api.clone(),
                antialias_mode,
                ctx.font_cache_thread.clone(),
            );

            let session = run_shared_session(run_canvas_session(canvas));

            send_value(
                session,
                detach_shared_session(run_create_canvas_session(ctx)),
            )
        })
    })
}

pub fn create_canvas_session(
    webrender_api: Box<dyn WebrenderApi>,
    font_cache_thread: FontCacheThread,
) -> SharedChannel<CreateCanvasProtocol> {
    let ctx = CanvasConfig {
        webrender_api: webrender_api,
        font_cache_thread: font_cache_thread,
    };

    let channel = run_shared_session(run_create_canvas_session(ctx));

    channel
}

impl CanvasSession {
    pub fn new(shared_channel: SharedChannel<CanvasProtocol>) -> CanvasSession {
        CanvasSession {
            shared_channel,
            message_buffer: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn flush_messages(&self) {
        let mut messages = self.message_buffer.lock().unwrap();
        if !messages.is_empty() {
            let messages2 = messages.split_off(0);
            send_canvas_messages(self.shared_channel.clone(), messages2);
        }
    }

    pub fn send_canvas_message(&self, message: CanvasMessage) {
        let mut messages = self.message_buffer.lock().unwrap();
        let was_empty = messages.is_empty();
        messages.push(message);

        if was_empty {
            let cloned = self.clone();
            task::spawn(async move {
                time::sleep(Duration::from_millis(10)).await;
                cloned.flush_messages();
            });
        }
    }

    pub fn get_shared_channel(&self) -> SharedChannel<CanvasProtocol> {
        self.flush_messages();
        self.shared_channel.clone()
    }
}

fn send_canvas_messages(session: SharedChannel<CanvasProtocol>, messages: Vec<CanvasMessage>) {
    async_acquire_shared_session(session, move |chan| {
        choose!(
            chan,
            Messages,
            send_value_to(chan, messages, release_shared_session(chan, terminate()))
        )
    });
}

pub async fn draw_image_in_other(
    source: SharedChannel<CanvasProtocol>,
    target: SharedChannel<CanvasProtocol>,
    image_size: Size2D<f64>,
    dest_rect: Rect<f64>,
    source_rect: Rect<f64>,
    smoothing: bool,
) {
    run_session(acquire_shared_session(source, move |source_chan| {
        choose!(
            source_chan,
            GetImageData,
            send_value_to(
                source_chan,
                (source_rect.to_u64(), image_size.to_u64()),
                receive_value_from(source_chan, move |image: ByteBuf| release_shared_session(
                    source_chan,
                    acquire_shared_session(target, move |target_chan| choose!(
                        target_chan,
                        Message,
                        send_value_to(
                            target_chan,
                            CanvasMessage::DrawImage(
                                Some(image),
                                source_rect.size,
                                dest_rect,
                                source_rect,
                                smoothing
                            ),
                            release_shared_session(target_chan, terminate())
                        )
                    ))
                ))
            )
        )
    }))
    .await;
}
