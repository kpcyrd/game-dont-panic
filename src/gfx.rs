use embedded_graphics::{
    image::ImageRaw,
    mono_font::{ascii, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
};

pub const SCREEN_WIDTH: usize = 128;

pub const TEXT_STYLE: MonoTextStyle<BinaryColor> = MonoTextStyleBuilder::new()
    .font(&ascii::FONT_4X6)
    .text_color(BinaryColor::On)
    .build();
pub const CHAR_WIDTH: usize = 4;

// Ferris: 53x30
pub const FERRIS_REVOLVER: ImageRaw<BinaryColor> =
    ImageRaw::new(include_bytes!("../data/ferris-revolver.raw"), 52);

pub const fn text_align_right(text: &str, total: usize) -> i32 {
    (total - (text.len() * CHAR_WIDTH)) as i32
}
