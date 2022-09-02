use std::io::Result;

use raqote::DrawTarget;

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

pub struct LDPluginContext {}

pub struct LDPluginRequirement {
    exclusive: bool,
}

pub enum PluginType {
    Screen,
    Button,
    Knob,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PluginScreenContext {}

impl PluginScreenContext {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub fn draw_target(&self, target: DrawTarget) -> Result<()> {
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
