use loupedeck::{PluginRegistrar, ScreenPlugin, ScreenPluginOptions};
use std::io::Result;

loupedeck::export_plugin!("time-plugin", register);

extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar
        .register_screen(
            "current-time",
            ScreenPluginOptions { exclusive: false },
            create_plugin,
        )
        .expect("registered");
}

fn create_plugin(ctx: loupedeck::PluginScreenContext) -> Box<dyn ScreenPlugin> {
    Box::new(ScreenPluginImpl { ctx })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScreenPluginImpl {
    ctx: loupedeck::PluginScreenContext,
}

impl ScreenPlugin for ScreenPluginImpl {
    fn on_touch(&self, _position: loupedeck::TouchEvent) -> Result<()> {
        println!("ScreenPluginImpl::on_touch");
        Ok(())
    }
}
