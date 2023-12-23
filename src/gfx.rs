use embedded_graphics::{
    geometry::Point,
    image::ImageRaw,
    mono_font::{ascii, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    primitives::PrimitiveStyle,
};

pub const SCREEN_WIDTH: u8 = 128;
pub const SCREEN_HEIGHT: u8 = 64;

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

pub const FERRIS_HEIGHT: u8 = 30;
pub const FERRIS_OFFSET: u8 = 62; // the largest possible ferris
pub const FERRIS_MAX_Y: u8 = SCREEN_HEIGHT - FERRIS_HEIGHT;

pub const OPPONENT_HEIGHT: u8 = 21;

// Ferris: 53x30
pub const FERRIS_REVOLVER: ImageRaw<BinaryColor> =
    ImageRaw::new(include_bytes!("../data/ferris-revolver.raw"), 52);
// Ferris: 62x30
pub const FERRIS_SCORPIO: ImageRaw<BinaryColor> =
    ImageRaw::new(include_bytes!("../data/ferris-scorpio.raw"), 61);
// Opponent: 30x22
pub const OPPONENT: ImageRaw<BinaryColor> =
    ImageRaw::new(include_bytes!("../data/opponent.raw"), 29);
// Game Over screen
pub const WASTED: ImageRaw<BinaryColor> = ImageRaw::new(include_bytes!("../data/wasted.raw"), 127);

pub const fn text_align_right(text: &str, total: u8) -> i32 {
    (total as usize - (text.len() * CHAR_WIDTH)) as i32
}

pub const CHAMBER_POSITIONS: &[Point] = &[
    Point::new(70, 14),
    Point::new(70, 32),
    Point::new(86, 42),
    Point::new(102, 33),
    Point::new(102, 15),
    Point::new(86, 6),
];

// u8::min is not const yet, so we make our own
pub const fn min(a: u8, b: u8) -> u8 {
    if a < b {
        a
    } else {
        b
    }
}
