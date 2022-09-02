use crate::{
    Button, ButtonPressEvent, Device, Event, Haptic, PluginRegistrar, PluginScreenContext,
    PressDirection, ScreenPlugin,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;
use std::io::Result;
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

mod plugin;
use plugin::*;

pub use plugin::PluginIdentifier;

struct Page {
    name: String,
    screen: HashMap<u8, ScreenPluginProxy>,
}

impl From<PageConfig> for Page {
    fn from(config: PageConfig) -> Self {
        Page {
            name: config.name,
            screen: HashMap::new(),
        }
    }
}

pub struct ScreenPluginProxy {
    plugin: Box<dyn ScreenPlugin>,
}

unsafe impl Send for ScreenPluginProxy {}
unsafe impl Sync for ScreenPluginProxy {}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageConfig {
    pub name: String,
    #[serde_as(as = "Vec<(_, _)>")]
    pub screen: HashMap<u8, PluginIdentifier>,
}

pub struct ControllerState {
    current_page: Option<Page>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ControllerConfig {
    pub pages: HashMap<String, PageConfig>,
}

pub struct Controller {
    plugin_registry: PluginRegistry,
    state: ControllerState,
    config: ControllerConfig,
    runtime: Option<Runtime>,
}

impl Controller {
    pub fn new() -> Controller {
        return Controller {
            plugin_registry: PluginRegistry::new(),
            state: ControllerState { current_page: None },
            runtime: None,
            config: ControllerConfig {
                pages: HashMap::default(),
            },
        };
    }

    pub fn load_plugin(&mut self, plugin_path: &str) -> Result<()> {
        unsafe { self.plugin_registry.load_from_path(plugin_path.to_string()) }
    }

    pub fn set_current_page(&mut self, mut device: Device, page_name: String) -> Result<()> {
        let page_config = self.config.pages.get(&page_name);
        if page_config.is_none() {
            return Ok(());
        }

        let current_page = self.create_page_instance(page_config.unwrap()).unwrap();

        self.start(device, current_page);

        return Ok(());
    }

    fn create_page_instance(&self, page_config: &PageConfig) -> Result<Page> {
        let mut page_instance = Page::from(page_config.clone());

        for (key, plugin_identifier) in page_config.screen.iter() {
            let plugin_id = &plugin_identifier.plugin_id;

            let plugin = self.plugin_registry.plugins.get(plugin_id);
            if plugin.is_none() {
                continue;
            }

            let plugin = plugin.unwrap();
            let screen = plugin.screens.get(&plugin_identifier.plugin_ref);

            if screen.is_none() {
                continue;
            }

            let screen = screen.unwrap();

            let plugin_context = PluginScreenContext::new();

            let screen_instance = screen(plugin_context);

            page_instance.screen.insert(
                *key,
                ScreenPluginProxy {
                    plugin: screen_instance,
                },
            );
        }

        return Ok(page_instance);
    }

    pub fn set_page(&mut self, page: PageConfig) -> Result<()> {
        self.config.pages.insert(page.name.clone(), page);

        return Ok(());
    }

    fn start(&mut self, mut device: Device, current_page: Page) {
        self.runtime = Some(Runtime::new().unwrap());
        let runtime = self.runtime.as_ref().unwrap();

        runtime.spawn(async move {
            let mut rx_event = device.tx_event.clone().unwrap().subscribe();

            loop {
                let next = rx_event.recv().await;

                match next {
                    Ok(event) => match event {
                        Event::ButtonPress(ButtonPressEvent {
                            tx_id: _,
                            button: _,
                            dir: PressDirection::Down,
                        }) => {
                            device.vibrate(Haptic::ShortLow).await;
                        }

                        Event::TouchEvent(touch_event) => {
                            println!("Touch event: {:?}", touch_event);
                            println!("Current page: {:?}", current_page.screen.keys());
                            current_page.screen.iter().for_each(|(key, screen)| {
                                screen.plugin.on_touch(touch_event.clone());
                            });
                        }

                        _ => {}
                    },
                    Err(e) => {
                        println!("Error {:?}", e);
                    }
                }
            }
        });
    }

    pub fn get_page(&self, page_name: String) -> Option<PageConfig> {
        let page_config = self.config.pages.get(&page_name);
        if page_config.is_none() {
            return None;
        }

        return Some(page_config.unwrap().clone());
    }
}
