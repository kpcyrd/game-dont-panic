#![no_std]
#![no_main]

use defmt_rtt as _;
use embedded_graphics::{
    image::{Image, ImageRaw},
    pixelcolor::BinaryColor,
    prelude::*,
};
use embedded_hal::timer::CountDown;
use fugit::ExtU32;
use fugit::RateExtU32;
use panic_halt as _;
use sh1106::{prelude::*, Builder};
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usbd_serial::SerialPort;
use usbd_serial::USB_CLASS_CDC;
use waveshare_rp2040_zero::entry;
use waveshare_rp2040_zero::{
    hal::{
        clocks::{init_clocks_and_plls, Clock},
        i2c::I2C,
        pac,
        timer::Timer,
        usb::UsbBus,
        watchdog::Watchdog,
        Sio,
    },
    Pins, XOSC_CRYSTAL_FREQ,
};

const FRAMES: &[ImageRaw<BinaryColor>] = &[
    ImageRaw::new(include_bytes!("../data/frame1.raw"), 128),
    ImageRaw::new(include_bytes!("../data/frame2.raw"), 128),
    ImageRaw::new(include_bytes!("../data/frame3.raw"), 128),
    ImageRaw::new(include_bytes!("../data/frame4.raw"), 128),
];

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    // Configure clocks and timers
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    let mut delay = timer.count_down();

    // Configure gpio
    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure display
    let i2c = I2C::i2c0(
        pac.I2C0,
        pins.gp4.into_function(), // sda
        pins.gp5.into_function(), // scl
        400.kHz(),
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
    );
    let mut display: GraphicsMode<_> = Builder::new()
        .with_rotation(DisplayRotation::Rotate180)
        .connect_i2c(i2c)
        .into();
    display.init().unwrap();

    // Configure USB serial
    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));
    let mut serial = SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .product("Serial port")
        .device_class(USB_CLASS_CDC)
        .build();

    // enter loop
    let mut iter = [].iter();
    loop {
        // get next frame or restart iterator
        let Some(raw) = iter.next() else {
            iter = FRAMES.iter();
            continue;
        };

        // draw image
        let im = Image::new(raw, Point::new(0, 0));
        im.draw(&mut display).unwrap();
        display.flush().unwrap();

        // sleep for frame rate
        delay.start(1000.millis());
        let _ = nb::block!(delay.wait());

        // read and discard any serial data sent to us
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];
            serial.read(&mut buf[..]).ok();
        }
    }
}
