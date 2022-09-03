use font_kit::family_name::FamilyName;
use font_kit::properties::Properties;
use font_kit::source::SystemSource;
use loupedeck::*;
use raqote::{DrawOptions, DrawTarget, Point, SolidSource, Source};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

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

    return convert_draw_target_to_rgb565(dt);
}

// #[tokio::main]
// async fn main() {
//     let mut ld = loupedeck::Device::new("COM3".to_string());
//     ld.connect().await;

//     println!("Connected to loupedeck");

//     let info = ld.get_info().await;
//     println!("Info {:?}", info.unwrap());

//     ld.draw_key(0, 0, create_key()).await;

//     let mut rx_event = ld.tx_event.clone().unwrap().subscribe();

//     tokio::spawn(async move {
//         while let Ok(event) = rx_event.recv().await {
//             // println!("LD Event {:?}", event);

//             match event {
//                 Event::ButtonPress(ButtonPressEvent {
//                     tx_id: _,
//                     button: Button::Circle4,
//                     dir: PressDirection::Down,
//                 }) => {
//                     println!("Button 4 Down");
//                     ld.vibrate(Haptic::Medium).await;
//                 }
//                 _ => {}
//             }
//         }
//     });

//     loop {}

//     ld.disconnect();
// }

#[tokio::main]
async fn main() {
    let mut ld = loupedeck::Device::new("COM3".to_string());
    ld.connect().await.expect("Failed to connect to loupedeck");

    let mut controller = Controller::new();

    let time_plugin_dir: &str =
        "C:\\Users\\adam\\dev\\loupedeck-conf-rs-core\\target\\debug\\loupedeck_plugin_time";

    controller
        .load_plugin(time_plugin_dir)
        .expect("Failed to load plugin");

    let mut screen_map: HashMap<u8, PluginIdentifier> = HashMap::default();

    screen_map.insert(
        0,
        PluginIdentifier {
            plugin_id: "time-plugin".to_string(),
            plugin_ref: "current-time".to_string(),
        },
    );

    let mut screen_map_2: HashMap<u8, PluginIdentifier> = HashMap::default();

    controller.start(ld);

    screen_map_2.insert(
        0,
        PluginIdentifier {
            plugin_id: "time-plugin".to_string(),
            plugin_ref: "current-date".to_string(),
        },
    );

    let mut page_config = PageConfig {
        name: "basic".to_string(),
        screen: screen_map,
    };

    let mut page_config_2 = PageConfig {
        name: "basic-2".to_string(),
        screen: screen_map_2,
    };

    controller
        .set_page(page_config)
        .expect("Failed to add page");

    controller
        .set_page(page_config_2)
        .expect("Failed to add page");

    controller
        .set_current_page("basic-2".to_string())
        .await
        .expect("Failed to set page");

    println!("Sleeping");
    sleep(Duration::from_secs(10)).await;
    println!("Setting page 2");

    controller
        .set_current_page("basic".to_string())
        .await
        .expect("Failed to set page");

    loop {}
}
