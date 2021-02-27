/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use cssparser::RGBA;
use euclid::default::{Size2D};
use serde_bytes::ByteBuf;
use std::default::Default;
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FillRule {
    Nonzero,
    Evenodd,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CanvasImageData {
    pub image_key: webrender_api::ImageKey,
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct CanvasGradientStop {
    pub offset: f64,
    pub color: RGBA,
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct LinearGradientStyle {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
    pub stops: Vec<CanvasGradientStop>,
}

impl LinearGradientStyle {
    pub fn new(
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        stops: Vec<CanvasGradientStop>,
    ) -> LinearGradientStyle {
        LinearGradientStyle {
            x0: x0,
            y0: y0,
            x1: x1,
            y1: y1,
            stops: stops,
        }
    }
}

#[derive(Clone, Debug, Deserialize, MallocSizeOf, Serialize)]
pub struct RadialGradientStyle {
    pub x0: f64,
    pub y0: f64,
    pub r0: f64,
    pub x1: f64,
    pub y1: f64,
    pub r1: f64,
    pub stops: Vec<CanvasGradientStop>,
}

impl RadialGradientStyle {
    pub fn new(
        x0: f64,
        y0: f64,
        r0: f64,
        x1: f64,
        y1: f64,
        r1: f64,
        stops: Vec<CanvasGradientStop>,
    ) -> RadialGradientStyle {
        RadialGradientStyle {
            x0: x0,
            y0: y0,
            r0: r0,
            x1: x1,
            y1: y1,
            r1: r1,
            stops: stops,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SurfaceStyle {
    pub surface_data: ByteBuf,
    pub surface_size: Size2D<u32>,
    pub repeat_x: bool,
    pub repeat_y: bool,
}

impl SurfaceStyle {
    pub fn new(
        surface_data: Vec<u8>,
        surface_size: Size2D<u32>,
        repeat_x: bool,
        repeat_y: bool,
    ) -> Self {
        Self {
            surface_data: ByteBuf::from(surface_data),
            surface_size,
            repeat_x,
            repeat_y,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FillOrStrokeStyle {
    Color(RGBA),
    LinearGradient(LinearGradientStyle),
    RadialGradient(RadialGradientStyle),
    Surface(SurfaceStyle),
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum LineCapStyle {
    Butt = 0,
    Round = 1,
    Square = 2,
}

impl FromStr for LineCapStyle {
    type Err = ();

    fn from_str(string: &str) -> Result<LineCapStyle, ()> {
        match string {
            "butt" => Ok(LineCapStyle::Butt),
            "round" => Ok(LineCapStyle::Round),
            "square" => Ok(LineCapStyle::Square),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum LineJoinStyle {
    Round = 0,
    Bevel = 1,
    Miter = 2,
}

impl FromStr for LineJoinStyle {
    type Err = ();

    fn from_str(string: &str) -> Result<LineJoinStyle, ()> {
        match string {
            "round" => Ok(LineJoinStyle::Round),
            "bevel" => Ok(LineJoinStyle::Bevel),
            "miter" => Ok(LineJoinStyle::Miter),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum RepetitionStyle {
    Repeat,
    RepeatX,
    RepeatY,
    NoRepeat,
}

impl FromStr for RepetitionStyle {
    type Err = ();

    fn from_str(string: &str) -> Result<RepetitionStyle, ()> {
        match string {
            "repeat" => Ok(RepetitionStyle::Repeat),
            "repeat-x" => Ok(RepetitionStyle::RepeatX),
            "repeat-y" => Ok(RepetitionStyle::RepeatY),
            "no-repeat" => Ok(RepetitionStyle::NoRepeat),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum CompositionStyle {
    SrcIn,
    SrcOut,
    SrcOver,
    SrcAtop,
    DestIn,
    DestOut,
    DestOver,
    DestAtop,
    Copy,
    Lighter,
    Xor,
    Clear,
}

impl FromStr for CompositionStyle {
    type Err = ();

    fn from_str(string: &str) -> Result<CompositionStyle, ()> {
        match string {
            "source-in" => Ok(CompositionStyle::SrcIn),
            "source-out" => Ok(CompositionStyle::SrcOut),
            "source-over" => Ok(CompositionStyle::SrcOver),
            "source-atop" => Ok(CompositionStyle::SrcAtop),
            "destination-in" => Ok(CompositionStyle::DestIn),
            "destination-out" => Ok(CompositionStyle::DestOut),
            "destination-over" => Ok(CompositionStyle::DestOver),
            "destination-atop" => Ok(CompositionStyle::DestAtop),
            "copy" => Ok(CompositionStyle::Copy),
            "lighter" => Ok(CompositionStyle::Lighter),
            "xor" => Ok(CompositionStyle::Xor),
            "clear" => Ok(CompositionStyle::Clear),
            _ => Err(()),
        }
    }
}

impl CompositionStyle {
    pub fn to_str(&self) -> &str {
        match *self {
            CompositionStyle::SrcIn => "source-in",
            CompositionStyle::SrcOut => "source-out",
            CompositionStyle::SrcOver => "source-over",
            CompositionStyle::SrcAtop => "source-atop",
            CompositionStyle::DestIn => "destination-in",
            CompositionStyle::DestOut => "destination-out",
            CompositionStyle::DestOver => "destination-over",
            CompositionStyle::DestAtop => "destination-atop",
            CompositionStyle::Copy => "copy",
            CompositionStyle::Lighter => "lighter",
            CompositionStyle::Xor => "xor",
            CompositionStyle::Clear => "clear",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum BlendingStyle {
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl FromStr for BlendingStyle {
    type Err = ();

    fn from_str(string: &str) -> Result<BlendingStyle, ()> {
        match string {
            "multiply" => Ok(BlendingStyle::Multiply),
            "screen" => Ok(BlendingStyle::Screen),
            "overlay" => Ok(BlendingStyle::Overlay),
            "darken" => Ok(BlendingStyle::Darken),
            "lighten" => Ok(BlendingStyle::Lighten),
            "color-dodge" => Ok(BlendingStyle::ColorDodge),
            "color-burn" => Ok(BlendingStyle::ColorBurn),
            "hard-light" => Ok(BlendingStyle::HardLight),
            "soft-light" => Ok(BlendingStyle::SoftLight),
            "difference" => Ok(BlendingStyle::Difference),
            "exclusion" => Ok(BlendingStyle::Exclusion),
            "hue" => Ok(BlendingStyle::Hue),
            "saturation" => Ok(BlendingStyle::Saturation),
            "color" => Ok(BlendingStyle::Color),
            "luminosity" => Ok(BlendingStyle::Luminosity),
            _ => Err(()),
        }
    }
}

impl BlendingStyle {
    pub fn to_str(&self) -> &str {
        match *self {
            BlendingStyle::Multiply => "multiply",
            BlendingStyle::Screen => "screen",
            BlendingStyle::Overlay => "overlay",
            BlendingStyle::Darken => "darken",
            BlendingStyle::Lighten => "lighten",
            BlendingStyle::ColorDodge => "color-dodge",
            BlendingStyle::ColorBurn => "color-burn",
            BlendingStyle::HardLight => "hard-light",
            BlendingStyle::SoftLight => "soft-light",
            BlendingStyle::Difference => "difference",
            BlendingStyle::Exclusion => "exclusion",
            BlendingStyle::Hue => "hue",
            BlendingStyle::Saturation => "saturation",
            BlendingStyle::Color => "color",
            BlendingStyle::Luminosity => "luminosity",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum CompositionOrBlending {
    Composition(CompositionStyle),
    Blending(BlendingStyle),
}

impl Default for CompositionOrBlending {
    fn default() -> CompositionOrBlending {
        CompositionOrBlending::Composition(CompositionStyle::SrcOver)
    }
}

impl FromStr for CompositionOrBlending {
    type Err = ();

    fn from_str(string: &str) -> Result<CompositionOrBlending, ()> {
        if let Ok(op) = CompositionStyle::from_str(string) {
            return Ok(CompositionOrBlending::Composition(op));
        }

        if let Ok(op) = BlendingStyle::from_str(string) {
            return Ok(CompositionOrBlending::Blending(op));
        }

        Err(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum TextAlign {
    Start,
    End,
    Left,
    Right,
    Center,
}

impl FromStr for TextAlign {
    type Err = ();

    fn from_str(string: &str) -> Result<TextAlign, ()> {
        match string {
            "start" => Ok(TextAlign::Start),
            "end" => Ok(TextAlign::End),
            "left" => Ok(TextAlign::Left),
            "right" => Ok(TextAlign::Right),
            "center" => Ok(TextAlign::Center),
            _ => Err(()),
        }
    }
}

impl Default for TextAlign {
    fn default() -> TextAlign {
        TextAlign::Start
    }
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum TextBaseline {
    Top,
    Hanging,
    Middle,
    Alphabetic,
    Ideographic,
    Bottom,
}

impl FromStr for TextBaseline {
    type Err = ();

    fn from_str(string: &str) -> Result<TextBaseline, ()> {
        match string {
            "top" => Ok(TextBaseline::Top),
            "hanging" => Ok(TextBaseline::Hanging),
            "middle" => Ok(TextBaseline::Middle),
            "alphabetic" => Ok(TextBaseline::Alphabetic),
            "ideographic" => Ok(TextBaseline::Ideographic),
            "bottom" => Ok(TextBaseline::Bottom),
            _ => Err(()),
        }
    }
}

impl Default for TextBaseline {
    fn default() -> TextBaseline {
        TextBaseline::Alphabetic
    }
}

#[derive(Clone, Copy, Debug, Deserialize, MallocSizeOf, PartialEq, Serialize)]
pub enum Direction {
    Ltr,
    Rtl,
    Inherit,
}

impl FromStr for Direction {
    type Err = ();

    fn from_str(string: &str) -> Result<Direction, ()> {
        match string {
            "ltr" => Ok(Direction::Ltr),
            "rtl" => Ok(Direction::Rtl),
            "inherit" => Ok(Direction::Inherit),
            _ => Err(()),
        }
    }
}

impl Default for Direction {
    fn default() -> Direction {
        Direction::Inherit
    }
}
