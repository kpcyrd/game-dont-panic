/// This is only used when needing to test-render things
use embedded_graphics::{
    image::{Image, ImageRaw},
    mono_font::{ascii, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Triangle},
};
use embedded_graphics_simulator::{
    BinaryColorTheme, OutputSettingsBuilder, SimulatorDisplay, Window,
};
use std::fs;

mod gfx {
    use super::*;
    pub const FERRIS_REVOLVER: ImageRaw<BinaryColor> =
        ImageRaw::new(include_bytes!("../data/ferris-revolver.raw"), 52);
}

mod game {
    pub const START_Y: u8 = 15;
}

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

#[derive(serde::Deserialize)]
pub enum Chamber {
    Empty,
    Loaded,
    Shot,
}

#[derive(serde::Deserialize)]
struct Conf {
    x: i32,
    y: i32,
    chamber: Chamber,
}

fn main() -> Result<(), core::convert::Infallible> {
    let mut display = SimulatorDisplay::<BinaryColor>::new(Size::new(128, 64));

    Circle::new(Point::new(64, 2), 60)
        .into_styled(WHITE_LINE)
        .draw(&mut display)?;

    let buf = fs::read_to_string("examples/positions.json").unwrap();
    let chambers: Vec<Conf> = serde_json::from_str(&buf).unwrap();

    let chambers = chambers
        .into_iter()
        .map(|c| (c.x, c.y, c.chamber))
        .collect::<Vec<_>>();

    for (x, y, chamber) in chambers {
        match chamber {
            Chamber::Empty => {
                Circle::new(Point::new(x, y), 16)
                    .into_styled(WHITE_LINE)
                    .draw(&mut display)?;
            }
            Chamber::Loaded => {
                Circle::new(Point::new(x, y), 16)
                    .into_styled(WHITE_FILL)
                    .draw(&mut display)?;
                Circle::new(Point::new(x + 5, y + 5), 6)
                    .into_styled(BLACK_LINE)
                    .draw(&mut display)?;
            }
            Chamber::Shot => {
                Circle::new(Point::new(x, y), 16)
                    .into_styled(WHITE_FILL)
                    .draw(&mut display)?;
                Circle::new(Point::new(x + 6, y + 6), 4)
                    .into_styled(BLACK_FILL)
                    .draw(&mut display)?;
            }
        }
    }

    Triangle::new(Point::new(62, 6), Point::new(57, 13), Point::new(66, 13))
        .into_styled(WHITE_FILL)
        .draw(&mut display)?;

    let im = Image::new(&gfx::FERRIS_REVOLVER, Point::new(0, game::START_Y as i32));
    im.draw(&mut display).unwrap();

    let output_settings = OutputSettingsBuilder::new()
        .theme(BinaryColorTheme::OledBlue)
        .build();
    Window::new("Hello World", &output_settings).show_static(&display);

    Ok(())
}
