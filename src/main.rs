use dht11::Dht11;
use esp_idf_hal::{
    delay::{Ets, FreeRtos},
    gpio::*,
    i2c::*,
    peripherals::Peripherals,
    prelude::*,
};
// use sh1106::{prelude::*, Builder};
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306, command::Command};
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    mono_font::{ascii::FONT_6X12, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle, Triangle, PrimitiveStyle},
};


fn draw_shapes<D: DrawTarget<Color = BinaryColor>>(display: &mut D) where <D as DrawTarget>::Error: std::fmt::Debug
{
    let yoffset = 8;

    let style = PrimitiveStyleBuilder::new()
        .stroke_width(1)
        .stroke_color(BinaryColor::On)
        .build();

    // screen outline
    // default display size is 128x64 if you don't pass a _DisplaySize_
    // enum to the _Builder_ struct
    Rectangle::new(Point::new(0, 0), Size::new(127, 31))
        .into_styled(style)
        .draw(display)
        .unwrap();

    // triangle
    Triangle::new(
        Point::new(16, 16 + yoffset),
        Point::new(16 + 16, 16 + yoffset),
        Point::new(16 + 8, yoffset),
    )
    .into_styled(style)
    .draw(display)
    .unwrap();

    // square
    Rectangle::new(Point::new(52, yoffset), Size::new_equal(16))
        .into_styled(style)
        .draw(display)
        .unwrap();

    // circle
    Circle::new(Point::new(88, yoffset), 16)
        .into_styled(style)
        .draw(display)
        .unwrap();

}

fn draw_some_text<D: DrawTarget<Color = BinaryColor>>(display: &mut D) where <D as DrawTarget>::Error: std::fmt::Debug
{
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X12)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
        .draw(display)
        .unwrap();

    Text::with_baseline("Hello Rust!", Point::new(0, 16), text_style, Baseline::Top)
        .draw(display)
        .unwrap();
}

const SOLID_STYLE:PrimitiveStyle<BinaryColor> = PrimitiveStyle::with_fill(BinaryColor::On);

fn draw_bar<D: DrawTarget<Color = BinaryColor>>(col: i32, display: &mut D)
-> Result<(), <D as DrawTarget>::Error> where <D as DrawTarget>::Error: std::fmt::Debug {
    let top_left = Point { x: col, y: 0 };
    let height: i32 = display.bounding_box().anchor_y(embedded_graphics::geometry::AnchorY::Bottom);
    let width = Size::new(1, (height - 1).try_into().unwrap());

    // screen outline
    // default display size is 128x64 if you don't pass a _DisplaySize_
    // enum to the _Builder_ struct
    Rectangle::new(top_left, width)
        .into_styled(SOLID_STYLE)
        .draw(display)
}


fn deg_c_to_f(deg_c: f32) -> f32 {
    // Formula
    // (0°C × 9/5) + 32 = 32°F
    let nine_fifths = 9f32 / 5f32;
    (deg_c * nine_fifths) + 32f32
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();
    let dht11_pin = PinDriver::input_output_od(peripherals.pins.gpio0.downgrade()).unwrap();
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio10;
    let scl = peripherals.pins.gpio1;

    let config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config).unwrap();

    let mut dht11 = Dht11::new(dht11_pin);
    // let mut display: GraphicsMode<_> = Builder::new()
    //     .with_rotation(DisplayRotation::Rotate180)
    //     .with_size(DisplaySize::Display128x64NoOffset)
    //     .connect_i2c(i2c).into();
    let mut interface = I2CDisplayInterface::new(i2c);
    Command::AllOn(true).send(&mut interface).unwrap();
    FreeRtos::delay_ms(2000);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    // display.set_addr_mode(ssd1306::command::AddrMode::Horizontal).unwrap();
    // display.set_addr_mode(mode)

    display.init().unwrap();


    draw_shapes(&mut display);
    FreeRtos::delay_ms(2000);
    draw_some_text(&mut display);

    display.flush().unwrap();

    let mut col = 0;
    loop {
        if let Err(e) = draw_bar(col, &mut display) {
            println!("{:?}", e)
        }
        if let Err(e) = display.flush() {
            println!("{:?}", e)
        }
        if col < display.dimensions().1.into() {
            col += 1;
        } else {
            col = 0;
        }
        let mut dht11_delay = Ets;
        match dht11.perform_measurement(&mut dht11_delay) {
            Ok(measurement) => println!(
                "temp: {}F, humidity: {}%",
                deg_c_to_f(measurement.temperature as f32 / 10.0),
                (measurement.humidity as f32 / 10.0)
            ),
            Err(e) => println!("{:?}", e),
        }
        FreeRtos::delay_ms(2000);
    }

}
