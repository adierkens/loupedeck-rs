use loupedeck::{PluginRegistrar, ScreenPlugin, ScreenPluginOptions};
use std::io::Result;
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

loupedeck::export_plugin!("time-plugin", register);

extern "C" fn register(registrar: &mut dyn PluginRegistrar) {
    registrar
        .register_screen(
            "current-time",
            ScreenPluginOptions { exclusive: false },
            create_plugin,
        )
        .expect("registered");

    registrar
        .register_screen(
            "current-date",
            ScreenPluginOptions { exclusive: false },
            create_date_plugin,
        )
        .expect("registered");
}

fn create_plugin(ctx: loupedeck::PluginScreenContext) -> Box<dyn ScreenPlugin> {
    Box::new(ScreenPluginImpl {
        ctx,
        runtime: Runtime::new().unwrap(),
    })
}

fn create_date_plugin(ctx: loupedeck::PluginScreenContext) -> Box<dyn ScreenPlugin> {
    Box::new(DatePluginImpl {
        ctx,
        runtime: Runtime::new().unwrap(),
    })
}

#[derive(Debug)]
pub struct ScreenPluginImpl {
    runtime: Runtime,
    ctx: loupedeck::PluginScreenContext,
}

impl ScreenPlugin for ScreenPluginImpl {
    fn on_touch(&self, _position: loupedeck::TouchEvent) -> Result<()> {
        println!("ScreenPluginImpl::on_touch");

        let ctx = self.ctx.clone();

        self.runtime.spawn(async move {
            ctx.vibrate(loupedeck::Haptic::Medium).await;

            sleep(Duration::from_secs(1)).await;
            ctx.vibrate(loupedeck::Haptic::VeryLong).await;
            sleep(Duration::from_secs(1)).await;
            ctx.vibrate(loupedeck::Haptic::VeryLong).await;
        });

        Ok(())
    }
}

#[derive(Debug)]
pub struct DatePluginImpl {
    runtime: Runtime,
    ctx: loupedeck::PluginScreenContext,
}

impl ScreenPlugin for DatePluginImpl {
    fn on_touch(&self, _position: loupedeck::TouchEvent) -> Result<()> {
        println!("DatePlugin::on_touch");
        let ctx = self.ctx.clone();

        self.runtime.spawn(async move {
            println!("Sending vibration in callback");
            ctx.vibrate(loupedeck::Haptic::Medium).await;

            sleep(Duration::from_secs(1)).await;
            ctx.vibrate(loupedeck::Haptic::VeryLong).await;
            sleep(Duration::from_secs(1)).await;
            ctx.vibrate(loupedeck::Haptic::VeryLong).await;
        });

        Ok(())
    }
}
