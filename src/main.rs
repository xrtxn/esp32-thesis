use display_interface_spi::SPIInterface;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, StyledDrawable, Triangle};
use embedded_graphics::{geometry::Point, Drawable};
use esp_idf_hal::delay::Delay;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::*;
use weact_studio_epd::{graphics::Display290BlackWhite, Color};
use weact_studio_epd::{graphics::DisplayRotation, WeActStudio290BlackWhiteDriver};

fn main() {
    // It is necessary to call this function once. Otherwise, some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let delay = Delay::new(50);
    let peripherals = Peripherals::take().unwrap();
    let spi = peripherals.spi2;

    let sclk = peripherals.pins.gpio12;
    //could be none
    let mosi = peripherals.pins.gpio0;
    let miso = peripherals.pins.gpio11;
    let cs = peripherals.pins.gpio10;
    let dc = peripherals.pins.gpio18;
    let rst = peripherals.pins.gpio4;
    let busy = peripherals.pins.gpio15;

    let driver =
        SpiDriver::new::<SPI2>(spi, sclk, miso, Some(mosi), &SpiDriverConfig::new()).unwrap();

    let config_1 = config::Config::new();

    let dc = PinDriver::output(dc).unwrap();

    let spi_device = SpiDeviceDriver::new(&driver, Some(cs), &config_1).unwrap();
    let spi_interface = SPIInterface::new(spi_device, dc);

    let busy = PinDriver::input(busy).unwrap();
    let rst = PinDriver::output(rst).unwrap();

    // log::info!("Intializing EPD...");
    let mut driver = WeActStudio290BlackWhiteDriver::new(spi_interface, busy, rst, delay);
    let mut display = Display290BlackWhite::new();
    display.set_rotation(DisplayRotation::Rotate90);
    driver.init().unwrap();

    let style = PrimitiveStyle::with_stroke(Color::Black, 2);
    display
        .bounding_box()
        .draw_styled(&style, &mut display)
        .unwrap();

    let mut starting_point = 32;
    while starting_point + 16 < display.bounding_box().bottom_right().unwrap().x {
        Triangle::new(
            Point::new(starting_point, 16),
            Point::new(starting_point - 16, 48),
            Point::new(starting_point + 16, 48),
        )
        .into_styled(PrimitiveStyle::with_stroke(Color::Black, 1))
        .draw(&mut display)
        .unwrap();
        starting_point = starting_point + 32;
    }

    driver.full_update(&display).unwrap();
}
