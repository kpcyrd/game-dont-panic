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
use embedded_hal::digital::v2::ToggleableOutputPin;
use fugit::RateExtU32;
use panic_halt as _;
use sh1106::{prelude::*, Builder};
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};
use usbd_serial::SerialPort;
use usbd_serial::USB_CLASS_CDC;
use core::cell::RefCell;
use critical_section::Mutex;
use waveshare_rp2040_zero::entry;
use waveshare_rp2040_zero::{
    hal::{
        clocks::{init_clocks_and_plls, Clock},
        i2c::I2C,
        pac,
        pac::interrupt,
        gpio::{self, Interrupt},
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

type LedPin = gpio::Pin<gpio::bank0::Gpio15, gpio::FunctionSioOutput, gpio::PullNone>;
type ButtonPin1 = gpio::Pin<gpio::bank0::Gpio10, gpio::FunctionSioInput, gpio::PullUp>;
type ButtonPin2 = gpio::Pin<gpio::bank0::Gpio11, gpio::FunctionSioInput, gpio::PullUp>;
type ButtonPin3 = gpio::Pin<gpio::bank0::Gpio12, gpio::FunctionSioInput, gpio::PullUp>;
type LedAndButton = (LedPin, ButtonPin1, ButtonPin2, ButtonPin3);
static GLOBAL_PINS: Mutex<RefCell<Option<LedAndButton>>> = Mutex::new(RefCell::new(None));
static EVENTS: Mutex<RefCell<[u8; 3]>> = Mutex::new(RefCell::new([0x31; 3]));

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

    // let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);
    // let mut delay = timer.count_down();

    // The single-cycle I/O block controls our GPIO pins
    let sio = Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure GPIO 25 as an output to drive our LED.
    // we can use reconfigure() instead of into_pull_up_input()
    // since the variable we're pushing it into has that type
    let led = pins.gpio15.reconfigure();

    // Set up the GPIO pin that will be our input
    let button1 = pins.gpio10.reconfigure();
    let button2 = pins.gpio11.reconfigure();
    let button3 = pins.gpio12.reconfigure();

    // Trigger on the 'falling edge' of the input pin.
    // This will happen as the button is being pressed
    button1.set_interrupt_enabled(Interrupt::EdgeHigh, true);
    button1.set_interrupt_enabled(Interrupt::EdgeLow, true);
    button2.set_interrupt_enabled(Interrupt::EdgeHigh, true);
    button2.set_interrupt_enabled(Interrupt::EdgeLow, true);
    button3.set_interrupt_enabled(Interrupt::EdgeHigh, true);
    button3.set_interrupt_enabled(Interrupt::EdgeLow, true);

    // Give away our pins by moving them into the `GLOBAL_PINS` variable.
    // We won't need to access them in the main thread again
    critical_section::with(|cs| {
        GLOBAL_PINS.borrow(cs).replace(Some((led, button1, button2, button3)));
    });

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

    // Unmask the IO_BANK0 IRQ so that the NVIC interrupt controller
    // will jump to the interrupt function when the interrupt occurs.
    // We do this last so that the interrupt can't go off while
    // it is in the middle of being configured
    unsafe {
        pac::NVIC::unmask(pac::Interrupt::IO_IRQ_BANK0);
    }

    loop {
        // interrupts handle everything else in this example.
        // cortex_m::asm::wfi();

        let mut buf = [0u8; 3];
        critical_section::with(|cs| {
            let events = EVENTS.borrow(cs);
            // .replace(Some((led, button1, button2, button3)));
            buf.copy_from_slice(&*events.borrow());
        });

        serial.write(&buf).ok();
        serial.write(b"\n").ok();

        /*
        delay.start(1000.millis());
        let _ = nb::block!(delay.wait());
        */

        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];
            serial.read(&mut buf[..]).ok();
        }
    }

    /*
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
    */
}

#[interrupt]
fn IO_IRQ_BANK0() {
    // The `#[interrupt]` attribute covertly converts this to `&'static mut Option<LedAndButton>`
    static mut LED_AND_BUTTON: Option<LedAndButton> = None;

    // This is one-time lazy initialisation. We steal the variables given to us
    // via `GLOBAL_PINS`.
    if LED_AND_BUTTON.is_none() {
        critical_section::with(|cs| {
            *LED_AND_BUTTON = GLOBAL_PINS.borrow(cs).take();
        });
    }

    // Need to check if our Option<LedAndButtonPins> contains our pins
    if let Some(gpios) = LED_AND_BUTTON {
        // borrow led and button by *destructuring* the tuple
        // these will be of type `&mut LedPin` and `&mut ButtonPin`, so we don't have
        // to move them back into the static after we use them
        let (led, button1, button2, button3) = gpios;

        /*
        // Check if the interrupt source is from the pushbutton going from high-to-low.
        // Note: this will always be true in this example, as that is the only enabled GPIO interrupt source
        if button.interrupt_status(Interrupt::EdgeLow) {
            // toggle can't fail, but the embedded-hal traits always allow for it
            // we can discard the return value by assigning it to an unnamed variable
            let _ = led.toggle();

            // Our interrupt doesn't clear itself.
            // Do that now so we don't immediately jump back to this interrupt handler.
            button.clear_interrupt(Interrupt::EdgeLow);
        }
        */

        critical_section::with(|cs| {
            let events = EVENTS.borrow(cs);

            if button1.interrupt_status(Interrupt::EdgeLow) {
                // let _ = led.toggle();
                events.replace_with(|events| {
                    events[0] = 0x30;
                    *events
                });
                button1.clear_interrupt(Interrupt::EdgeLow);
            }
            if button1.interrupt_status(Interrupt::EdgeHigh) {
                events.replace_with(|events| {
                    events[0] = 0x31;
                    *events
                });
                button1.clear_interrupt(Interrupt::EdgeHigh);
            }

            if button2.interrupt_status(Interrupt::EdgeLow) {
                events.replace_with(|events| {
                    events[1] = 0x30;
                    *events
                });
                button2.clear_interrupt(Interrupt::EdgeLow);
            }
            if button2.interrupt_status(Interrupt::EdgeHigh) {
                events.replace_with(|events| {
                    events[1] = 0x31;
                    *events
                });
                button2.clear_interrupt(Interrupt::EdgeHigh);
            }

            if button3.interrupt_status(Interrupt::EdgeLow) {
                events.replace_with(|events| {
                    events[2] = 0x30;
                    *events
                });
                button3.clear_interrupt(Interrupt::EdgeLow);
            }
            if button3.interrupt_status(Interrupt::EdgeHigh) {
                events.replace_with(|events| {
                    events[2] = 0x31;
                    *events
                });
                button3.clear_interrupt(Interrupt::EdgeHigh);
            }
        });
    }
}
