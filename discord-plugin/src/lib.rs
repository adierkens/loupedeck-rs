use loupedeck::{PluginRegistrar, ScreenPlugin};
use std::io::Result;

loupedeck::export_plugin!(register);

extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar.register_screen("time-plugin", Box::new(ScreenPluginImpl));
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScreenPluginImpl;

impl ScreenPlugin for ScreenPluginImpl {
    fn create(&self, _position: loupedeck::loupedeck::Screen) -> Result<()> {
        println!("ScreenPluginImpl::create");
        Ok(())
    }
    fn destroy(&self, _position: loupedeck::loupedeck::Screen) -> Result<()> {
        println!("ScreenPluginImpl::destroy");
        Ok(())
    }
    fn on_touch(&self, _position: loupedeck::loupedeck::TouchEvent) -> Result<()> {
        println!("ScreenPluginImpl::on_touch");
        Ok(())
    }
}
