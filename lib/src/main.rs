use font_kit::family_name::FamilyName;
use font_kit::source::SystemSource;
use font_kit::{font::Font, properties::Properties};
use loupedeck::*;
use raqote::{DrawOptions, DrawTarget, Point, SolidSource, Source};
use tokio::time::{self, Duration, Instant};
use tokio_serial::SerialPortBuilderExt;

fn convert_ARGB888_to_RGB565(argb: u32) -> u16 {
    let r = ((argb >> 16) & 0xff) as u16;
    let g = ((argb >> 8) & 0xff) as u16;
    let b = (argb & 0xff) as u16;
    ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3)
}

#[cfg(test)]
mod tests {
    use raqote::{DrawOptions, DrawTarget, SolidSource, Source};

    use crate::convert_to_rgb565;

    use super::convert_ARGB888_to_RGB565;

    #[test]
    fn it_converts() {
        assert_eq!(convert_ARGB888_to_RGB565(0xFF000000), 0);
        assert_eq!(convert_ARGB888_to_RGB565(0xFFFFFFFF), 0xffff);
        assert_eq!(convert_ARGB888_to_RGB565(0xFF0000FF), 0x001f);
        assert_eq!(convert_ARGB888_to_RGB565(0xFFFF0000), 0xf800);
        assert_eq!(convert_ARGB888_to_RGB565(0xFF00FF00), 0x07e0);
    }

    #[test]
    fn it_works_for_dt_red() {
        let mut dt = DrawTarget::new(1, 1);

        let mut solidRed: SolidSource =
            SolidSource::from_unpremultiplied_argb(255, 0xFF, 0x00, 0x00);

        dt.fill_rect(
            0.0,
            0.0,
            1.0,
            1.0,
            &Source::Solid(solidRed),
            &DrawOptions::new(),
        );

        assert_eq!(convert_to_rgb565(dt), vec![0x00, 0xF8,]);
    }
    #[test]
    fn it_works_for_dt_green() {
        let mut dt = DrawTarget::new(1, 1);

        let mut solidGreen: SolidSource =
            SolidSource::from_unpremultiplied_argb(255, 0x00, 0xFF, 0x00);

        dt.fill_rect(
            0.0,
            0.0,
            1.0,
            1.0,
            &Source::Solid(solidGreen),
            &DrawOptions::new(),
        );

        assert_eq!(convert_to_rgb565(dt), vec![0xE0, 0x07,]);
    }

    #[test]
    fn it_works_for_dt_blue() {
        let mut dt = DrawTarget::new(1, 1);

        let mut solidBlue: SolidSource =
            SolidSource::from_unpremultiplied_argb(255, 0x00, 0x00, 0xFF);

        dt.fill_rect(
            0.0,
            0.0,
            1.0,
            1.0,
            &Source::Solid(solidBlue),
            &DrawOptions::new(),
        );

        assert_eq!(convert_to_rgb565(dt), vec![0x1F, 0x00]);
    }
}

fn convert_to_rgb565(dt: DrawTarget) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    let bg_color: u32 = 0x000000;

    // orig is AARRGGBB

    let draw_data = dt.get_data();
    for px in draw_data {
        let rgb16 = convert_ARGB888_to_RGB565(*px);
        result.append(&mut rgb16.to_le_bytes().to_vec());
    }

    result
}

fn create_key() -> Vec<u8> {
    let mut dt = DrawTarget::new(90, 90);

    let mut solidRed: SolidSource = SolidSource::from_unpremultiplied_argb(255, 0xFF, 0x00, 0x00);
    let mut solidGreen: SolidSource = SolidSource::from_unpremultiplied_argb(255, 0x00, 0xFF, 0x00);
    let mut solidBlue: SolidSource = SolidSource::from_unpremultiplied_argb(255, 0x00, 0x00, 0xFF);
    let mut solidWhite: SolidSource = SolidSource::from_unpremultiplied_argb(255, 0xFF, 0xFF, 0xFF);
    let mut solidBlack: SolidSource = SolidSource::from_unpremultiplied_argb(255, 0x0, 0x0, 0x0);

    let mut purple = SolidSource::from_unpremultiplied_argb(255, 204, 52, 235);

    dt.fill_rect(
        0.0,
        0.0,
        90.0,
        90.0,
        &Source::Solid(solidWhite),
        &DrawOptions::new(),
    );

    let font = SystemSource::new()
        .select_best_match(&[FamilyName::Monospace], &Properties::new())
        .unwrap()
        .load()
        .unwrap();

    dt.draw_text(
        &font,
        16.0,
        "test",
        Point::new(45.0, 45.0),
        &Source::Solid(solidRed),
        &DrawOptions::new(),
    );

    return convert_to_rgb565(dt);
}

#[tokio::main]
async fn main() {
    let mut ld = loupedeck::Device::new("COM3".to_string());
    ld.connect().await;

    println!("Connected to loupedeck");

    let info = ld.get_info().await;
    println!("Info {:?}", info.unwrap());

    ld.draw_key(create_key()).await;

    let mut rx_event = ld.tx_event.clone().unwrap().subscribe();

    tokio::spawn(async move {
        while let Ok(event) = rx_event.recv().await {
            // println!("LD Event {:?}", event);

            match event {
                Event::ButtonPress(ButtonPressEvent {
                    tx_id: _,
                    button: Button::Circle4,
                    dir: PressDirection::Down,
                }) => {
                    println!("Button 4 Down");
                    ld.vibrate(Haptic::Medium).await;
                }
                _ => {}
            }
        }
    });

    loop {}

    ld.disconnect();
}
