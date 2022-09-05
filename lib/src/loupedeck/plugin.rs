use std::io::Result;

use raqote::DrawTarget;

use crate::{KeyLocation, Screen, KEY_SIZE};

#[macro_export]
macro_rules! export_plugin {
    ($plugin_id:literal, $register:expr) => {
        #[doc(hidden)]
        #[no_mangle]
        pub static plugin_declaration: $crate::PluginDeclaration = $crate::PluginDeclaration {
            rustc_version: $crate::RUSTC_VERSION,
            core_version: $crate::CORE_VERSION,
            plugin_id: $plugin_id,
            register: $register,
        };
    };
}

pub struct LDPluginRequirement {
    exclusive: bool,
}

pub enum PluginType {
    Screen,
    Button,
    Knob,
}

#[derive(Debug, Clone)]
pub struct PluginScreenContext {
    device_event_emitter: crate::ExternalDeviceEventEmitter,
    position: Screen,
    key_id: KeyLocation,
}

impl PluginScreenContext {
    pub(crate) fn new(
        device_event_emitter: crate::ExternalDeviceEventEmitter,
        position: Screen,
        key_id: KeyLocation,
    ) -> Self {
        println!("PluginScreenContext::new {:?}", key_id);

        Self {
            position,
            key_id,
            device_event_emitter,
        }
    }

    pub async fn draw_target(&self, target: DrawTarget) -> Result<()> {
        let x: u16 = KEY_SIZE * (self.key_id.x as u16);
        let y: u16 = KEY_SIZE * (self.key_id.y as u16);

        self.device_event_emitter
            .draw_target(self.position.clone(), x, y, KEY_SIZE, KEY_SIZE, target)
            .await;

        Ok(())
    }

    pub async fn draw_rgb565(&self, data: Vec<u8>) -> Result<()> {
        let x: u16 = KEY_SIZE * (self.key_id.x as u16);
        let y: u16 = KEY_SIZE * (self.key_id.y as u16);

        self.device_event_emitter
            .draw_rgb565(self.position.clone(), x, y, KEY_SIZE, KEY_SIZE, data)
            .await;

        Ok(())
    }

    pub async fn vibrate(&self, level: crate::Haptic) -> Result<()> {
        // println!("Sending vibration: {:?}", level);
        self.device_event_emitter.vibrate(level).await;
        Ok(())
    }
}

pub trait ScreenPlugin {
    fn on_touch(&self, position: crate::TouchEvent) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScreenPluginOptions {
    pub exclusive: bool,
}

pub type ScreenPluginFactory = fn(ctx: PluginScreenContext) -> Box<dyn ScreenPlugin>;

pub trait PluginRegistrar {
    fn register_screen(
        &mut self,
        name: &str,
        options: ScreenPluginOptions,
        create: ScreenPluginFactory,
    ) -> Result<()>;
}

#[derive(Copy, Clone)]
pub struct PluginDeclaration {
    pub rustc_version: &'static str,
    pub core_version: &'static str,
    pub plugin_id: &'static str,
    pub register: unsafe extern "C" fn(&mut dyn PluginRegistrar),
}
