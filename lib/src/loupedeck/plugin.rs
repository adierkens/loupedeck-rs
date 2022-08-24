use std::io::Result;

#[macro_export]
macro_rules! export_plugin {
    ($register:expr) => {
        #[doc(hidden)]
        #[no_mangle]
        pub static plugin_declaration: $crate::PluginDeclaration = $crate::PluginDeclaration {
            rustc_version: $crate::RUSTC_VERSION,
            core_version: $crate::CORE_VERSION,
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

pub trait KnobPlugin {
    fn create(&self) -> Result<LDPluginRequirement>;
    fn destroy(&self) -> Result<()>;
    fn on_change(&self, position: crate::KnobRotateEvent) -> Result<()>;
}

pub trait ButtonPlugin {
    fn create(&self, position: crate::Button) -> Result<()>;
    fn destroy(&self, position: crate::Button) -> Result<()>;
    fn on_change(&self, position: crate::ButtonPressEvent) -> Result<()>;
}

pub trait ScreenPlugin {
    fn create(&self, position: crate::Screen) -> Result<()>;
    fn destroy(&self, position: crate::Screen) -> Result<()>;
    fn on_touch(&self, position: crate::TouchEvent) -> Result<()>;
}

pub trait PluginRegistrar {
    fn register_knob(&mut self, name: &str, plugin: Box<dyn KnobPlugin>);
    fn register_button(&mut self, name: &str, plugin: Box<dyn ButtonPlugin>);
    fn register_screen(&mut self, name: &str, plugin: Box<dyn ScreenPlugin>);
}

#[derive(Copy, Clone)]
pub struct PluginDeclaration {
    pub rustc_version: &'static str,
    pub core_version: &'static str,
    pub register: unsafe extern "C" fn(&mut dyn PluginRegistrar),
}
