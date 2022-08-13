pub static CORE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static RUSTC_VERSION: &str = env!("RUSTC_VERSION");

pub mod loupedeck;
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

pub trait KnobPlugin {
    fn create(&self, position: loupedeck::Knob) -> Result<()>;
    fn destroy(&self, position: loupedeck::Knob) -> Result<()>;
    fn on_change(&self, position: loupedeck::KnobRotateEvent) -> Result<&str>;
}

pub trait ButtonPlugin {
    fn create(&self, position: loupedeck::Button) -> Result<()>;
    fn destroy(&self, position: loupedeck::Button) -> Result<()>;
    fn on_change(&self, position: loupedeck::ButtonPressEvent) -> Result<()>;
}

pub trait ScreenPlugin {
    fn create(&self, position: loupedeck::Screen) -> Result<()>;
    fn destroy(&self, position: loupedeck::Screen) -> Result<()>;
    fn on_touch(&self, position: loupedeck::TouchEvent) -> Result<()>;
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
