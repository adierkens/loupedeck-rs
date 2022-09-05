use loupedeck::*;
use std::collections::HashMap;
use tokio::time;

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

    let mut screen_map: HashMap<KeyLocation, PluginIdentifier> = HashMap::default();

    screen_map.insert(
        KeyLocation::new(0, 0),
        PluginIdentifier {
            plugin_id: "time-plugin".to_string(),
            plugin_ref: "current-time".to_string(),
        },
    );

    screen_map.insert(
        KeyLocation::new(2, 2),
        PluginIdentifier {
            plugin_id: "time-plugin".to_string(),
            plugin_ref: "current-date".to_string(),
        },
    );

    controller.start(ld);

    let page_config = PageConfig {
        name: "basic".to_string(),
        screen: screen_map,
    };

    controller
        .set_page(page_config)
        .expect("Failed to add page");

    controller
        .set_current_page("basic".to_string())
        .await
        .expect("Failed to set page");

    time::sleep(time::Duration::from_secs(1000)).await
}
