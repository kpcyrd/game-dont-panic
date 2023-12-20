use embedded_graphics::{
    geometry::Point,
    image::ImageRaw,
    mono_font::{ascii, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    primitives::PrimitiveStyle,
};

pub const SCREEN_WIDTH: usize = 128;

pub const TEXT_STYLE: MonoTextStyle<BinaryColor> = MonoTextStyleBuilder::new()
    .font(&ascii::FONT_4X6)
    .text_color(BinaryColor::On)
    .build();
pub const WHITE_LINE: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
pub const BLACK_LINE: PrimitiveStyle<BinaryColor> =
    PrimitiveStyle::with_stroke(BinaryColor::Off, 1);
pub const WHITE_FILL: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_fill(BinaryColor::On);
pub const BLACK_FILL: PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_fill(BinaryColor::Off);
pub const CHAR_WIDTH: usize = 4;

// Ferris: 53x30
pub const FERRIS_REVOLVER: ImageRaw<BinaryColor> =
    ImageRaw::new(include_bytes!("../data/ferris-revolver.raw"), 52);

pub const fn text_align_right(text: &str, total: usize) -> i32 {
    (total - (text.len() * CHAR_WIDTH)) as i32
}

pub const CHAMBER_POSITIONS: &[Point] = &[
    Point::new(70, 14),
    Point::new(70, 32),
    Point::new(86, 42),
    Point::new(102, 33),
    Point::new(102, 15),
    Point::new(86, 6),
];
