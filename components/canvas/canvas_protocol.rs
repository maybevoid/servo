use ferrite_session::prelude::*;

use canvas_traits::canvas::*;
use cssparser::RGBA;
use euclid::default::{Point2D, Rect, Size2D, Transform2D};
use ipc_channel::ipc::IpcSharedMemory;
use serde;
use serde_bytes::ByteBuf;
use std::sync::{Arc, Mutex};
use style::properties::style_structs::Font as FontStyleStruct;

pub type CanvasProtocol = LinearToShared<ExternalChoice<CanvasOps>>;

pub type CreateCanvasProtocol =
    LinearToShared<ReceiveValue<(Size2D<u64>, bool), SendValue<SharedChannel<CanvasProtocol>, Release>>>;

#[derive(Clone)]
pub struct CanvasSession {
    pub(crate) message_buffer: Arc<Mutex<Vec<CanvasMessage>>>,
    pub(crate) shared_channel: SharedChannel<CanvasProtocol>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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
    LineTo(Point2D<f32>),
    MoveTo(Point2D<f32>),
    PutImageData(Rect<u64>, ByteBuf),
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
    Recreate(Size2D<u64>),
}

define_choice! { CanvasOps;
  Message: ReceiveValue<CanvasMessage, Release>,
  Messages: ReceiveValue<Vec<CanvasMessage>, Release>,
  GetTransform: SendValue<Transform2D<f32>, Release>,
  GetImageData: ReceiveValue<(Rect<u64>, Size2D<u64>),
    SendValue<ByteBuf, Release>>,
  IsPointInPath: ReceiveValue<(f64, f64, FillRule),
    SendValue<bool, Release>>,
  FromLayout: SendValue<Option<CanvasImageData>, Release>,
  FromScript: SendValue<IpcSharedMemory, Release>,
}
