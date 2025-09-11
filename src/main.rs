use display_interface_spi::SPIInterface;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, StyledDrawable, Triangle};
use embedded_graphics::{geometry::Point, Drawable};
use esp_idf_hal::delay::Delay;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::*;
use weact_studio_epd::graphics::Display;
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

    // --- Build info from vergen + git short emitted by build.rs ---
    let build_date = option_env!("GIT_SHORT").unwrap_or("unknown");
    let build_info = format!("commit: {}", build_date);
    add_footer_info(&mut display, &build_info);

    driver.full_update(&display).unwrap();
}

fn add_footer_info(display: &mut Display290BlackWhite, build_info: &str) {
    use embedded_graphics::mono_font::MonoTextStyle;
    use embedded_graphics::text::{Baseline, Text};

    let font = profont::PROFONT_7_POINT;
    let text_style = MonoTextStyle::new(&font, Color::Black);

    let br = display.bounding_box().bottom_right().unwrap();

    let text_width = build_info.chars().count() as i32 * font.character_size.width as i32;
    let pos = Point::new(br.x - text_width, br.y - font.character_size.height as i32);

    // Draw the build info (adjust coordinates as needed)
    Text::with_baseline(build_info, pos, text_style, Baseline::Top)
        .draw(display)
        .unwrap();
}
