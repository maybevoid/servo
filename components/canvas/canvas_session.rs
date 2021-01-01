use ferrite_session::*;

use cssparser::RGBA;
use euclid::default::{Point2D, Rect, Size2D, Transform2D};
use ipc_channel::ipc::{IpcBytesReceiver, IpcBytesSender, IpcSender, IpcSharedMemory};
use serde_bytes::ByteBuf;
use std::default::Default;
use std::str::FromStr;
use style::properties::style_structs::Font as FontStyleStruct;


use crate::canvas_data::*;
use crate::canvas_paint_thread::{WebrenderApi};
use canvas_traits::canvas::*;
use canvas_traits::ConstellationCanvasMsg;
use crossbeam_channel::{select, unbounded, Sender};
use gfx::font_cache_thread::FontCacheThread;
use ipc_channel::router::ROUTER;
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::thread;
use webrender_api::{ImageData, ImageDescriptor, ImageKey};

#[derive(Debug)]
pub enum CanvasMessage {
    Arc(Point2D<f32>, f32, f32, f32, bool),
    ArcTo(Point2D<f32>, Point2D<f32>, f32),
    DrawImage(Option<ByteBuf>, Size2D<f64>, Rect<f64>, Rect<f64>, bool),
    BeginPath,
    BezierCurveTo(Point2D<f32>, Point2D<f32>, Point2D<f32>),
    ClearRect(Rect<f32>),
    Clip,
    ClosePath,
    Ellipse(Point2D<f32>, f32, f32, f32, f32, f32, bool),
    Fill(FillOrStrokeStyle),
    FillText(String, f64, f64, Option<f64>, FillOrStrokeStyle, bool),
    FillRect(Rect<f32>, FillOrStrokeStyle),
    GetTransform(IpcSender<Transform2D<f32>>),
    IsPointInPath(f64, f64, FillRule, IpcSender<bool>),
    LineTo(Point2D<f32>),
    MoveTo(Point2D<f32>),
    PutImageData(Rect<u64>, IpcBytesReceiver),
    QuadraticCurveTo(Point2D<f32>, Point2D<f32>),
    Rect(Rect<f32>),
    RestoreContext,
    SaveContext,
    StrokeRect(Rect<f32>, FillOrStrokeStyle),
    Stroke(FillOrStrokeStyle),
    SetLineWidth(f32),
    SetLineCap(LineCapStyle),
    SetLineJoin(LineJoinStyle),
    SetMiterLimit(f32),
    SetGlobalAlpha(f32),
    SetGlobalComposition(CompositionOrBlending),
    SetTransform(Transform2D<f32>),
    SetShadowOffsetX(f64),
    SetShadowOffsetY(f64),
    SetShadowBlur(f64),
    SetShadowColor(RGBA),
    SetFont(FontStyleStruct),
    SetTextAlign(TextAlign),
    SetTextBaseline(TextBaseline),
}

define_choice! { CanvasOps;
  Message: ReceiveValue <
    CanvasMessage,
    Z
  >,
  GetImageData: ReceiveValue <
    ( Rect<u64>, Size2D<u64> ),
    SendValue <
      Vec < u8 >,
      Z
    >
  >,
  IsPointInPath: ReceiveValue <
    ( f64, f64, FillRule ),
    SendValue <
      bool,
      Z
    >
  >,
  PutImageData: ReceiveValue <
    Rect<u64>,
    ReceiveValue <
      Vec < u8 >,
      Z
    >
  >,
  FromLayout: SendValue <
    Option<CanvasImageData>,
    Z
  >,
  FromScript: SendValue <
    Vec<u8>,
    Z
  >,
  Recreate: ReceiveValue <
    Size2D<u64>,
    Z
  >
}

pub type CanvasSession = LinearToShared <
  ExternalChoice <
    CanvasOps
  >
>;

pub type CreateCanvas = LinearToShared <
  ReceiveValue <
    ( Size2D<u64>, bool ),
    SendValue <
      SharedChannel <
        CanvasSession
      >,
      Z
    >
  >
>;

struct CanvasContext {
  canvas_id: CanvasId,
  canvas: CanvasData< 'static >,
  webrender_api: Box<dyn WebrenderApi>,
  font_cache_thread: FontCacheThread,
}

fn canvas_session
  ( mut ctx: CanvasContext ) ->
    SharedSession <
      CanvasSession
    >
{
  accept_shared_session (
    offer_choice! {
      Message => {
        receive_value! ( message => {
          match message {
            CanvasMessage::FillText(text, x, y, max_width, style, is_rtl) => {
              ctx.canvas.set_fill_style(style);
              ctx.canvas.fill_text(text, x, y, max_width, is_rtl);
            },
            CanvasMessage::FillRect(rect, style) => {
              ctx.canvas.set_fill_style(style);
              ctx.canvas.fill_rect(&rect);
            },
            CanvasMessage::StrokeRect(rect, style) => {
                ctx.canvas.set_stroke_style(style);
                ctx.canvas.stroke_rect(&rect);
            },
            CanvasMessage::ClearRect(ref rect) => ctx.canvas.clear_rect(rect),
            CanvasMessage::BeginPath => ctx.canvas.begin_path(),
            CanvasMessage::ClosePath => ctx.canvas.close_path(),
            CanvasMessage::Fill(style) => {
                ctx.canvas.set_fill_style(style);
                ctx.canvas.fill();
            },
            CanvasMessage::Stroke(style) => {
                ctx.canvas.set_stroke_style(style);
                ctx.canvas.stroke();
            },
            CanvasMessage::Clip => ctx.canvas.clip(),
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
                ctx.canvas.draw_image(
                    data,
                    image_size,
                    dest_rect,
                    source_rect,
                    smoothing_enabled,
                )
            },
            CanvasMessage::MoveTo(ref point) => ctx.canvas.move_to(point),
            CanvasMessage::LineTo(ref point) => ctx.canvas.line_to(point),
            CanvasMessage::Rect(ref rect) => ctx.canvas.rect(rect),
            CanvasMessage::QuadraticCurveTo(ref cp, ref pt) => {
                ctx.canvas.quadratic_curve_to(cp, pt)
            },
            CanvasMessage::BezierCurveTo(ref cp1, ref cp2, ref pt) => {
                ctx.canvas.bezier_curve_to(cp1, cp2, pt)
            },
            CanvasMessage::Arc(ref center, radius, start, end, ccw) => {
                ctx.canvas.arc(center, radius, start, end, ccw)
            },
            CanvasMessage::ArcTo(ref cp1, ref cp2, radius) => {
                ctx.canvas.arc_to(cp1, cp2, radius)
            },
            CanvasMessage::Ellipse(ref center, radius_x, radius_y, rotation, start, end, ccw) =>
              ctx.canvas
                .ellipse(center, radius_x, radius_y, rotation, start, end, ccw),
            CanvasMessage::RestoreContext => ctx.canvas.restore_context_state(),
            CanvasMessage::SaveContext => ctx.canvas.save_context_state(),
            CanvasMessage::SetLineWidth(width) => ctx.canvas.set_line_width(width),
            CanvasMessage::SetLineCap(cap) => ctx.canvas.set_line_cap(cap),
            CanvasMessage::SetLineJoin(join) => ctx.canvas.set_line_join(join),
            CanvasMessage::SetMiterLimit(limit) => ctx.canvas.set_miter_limit(limit),
            CanvasMessage::GetTransform(sender) => {
                let transform = ctx.canvas.get_transform();
                sender.send(transform).unwrap();
            },
            CanvasMessage::SetTransform(ref matrix) => ctx.canvas.set_transform(matrix),
            CanvasMessage::SetGlobalAlpha(alpha) => ctx.canvas.set_global_alpha(alpha),
            CanvasMessage::SetGlobalComposition(op) => {
                ctx.canvas.set_global_composition(op)
            },
            CanvasMessage::SetShadowOffsetX(value) => {
                ctx.canvas.set_shadow_offset_x(value)
            },
            CanvasMessage::SetShadowOffsetY(value) => {
                ctx.canvas.set_shadow_offset_y(value)
            },
            CanvasMessage::SetShadowBlur(value) => ctx.canvas.set_shadow_blur(value),
            CanvasMessage::SetShadowColor(color) => ctx.canvas.set_shadow_color(color),
            CanvasMessage::SetFont(font_style) => ctx.canvas.set_font(font_style),
            CanvasMessage::SetTextAlign(text_align) => {
                ctx.canvas.set_text_align(text_align)
            },
            CanvasMessage::SetTextBaseline(text_baseline) => {
                ctx.canvas.set_text_baseline(text_baseline)
            },
            _ => {
              todo!()
            }
          }

          detach_shared_session (
            canvas_session ( ctx )
          )
        })
      },
      GetImageData => {
        receive_value!( msg => {
          let (dest_rect, canvas_size) = msg;
          let pixels = Vec::from(
            ctx.canvas.read_pixels(dest_rect, canvas_size)
          );

          send_value!(pixels,
            detach_shared_session (
              canvas_session ( ctx )
            ))
        })
      },
      IsPointInPath => {
        receive_value!( msg => {
          let (x, y, fill_rule) = msg;
          let res = ctx.canvas.is_point_in_path_bool(x, y, fill_rule);

          send_value!(res,
            detach_shared_session (
              canvas_session ( ctx )
            ))
        })
      },
      PutImageData => {
        receive_value! ( rect => {
          receive_value! ( img => {
            ctx.canvas.put_image_data(img, rect);

            detach_shared_session (
              canvas_session ( ctx )
            )
          })
        })
      },
      FromLayout => {
        let data = ctx.canvas.get_data();
        send_value! ( data,
          detach_shared_session (
            canvas_session ( ctx )
          )
        )
      },
      FromScript => {
        let data = ctx.canvas.get_pixels();
        send_value! ( data,
          detach_shared_session (
            canvas_session ( ctx )
          )
        )
      },
      Recreate => {
        receive_value! ( size => {
          ctx.canvas.recreate(size);

          detach_shared_session (
            canvas_session ( ctx )
          )
        })
      },
    }
  )
}
