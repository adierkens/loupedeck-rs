use font_kit::family_name::FamilyName;
use font_kit::font::Font;
use font_kit::properties::{Properties, Weight};
use font_kit::source::SystemSource;
use loupedeck::{
    convert_draw_target_to_rgb565, PluginRegistrar, ScreenPlugin, ScreenPluginOptions,
};
use pathfinder_geometry::vector::{vec2f, vec2i};
use raqote::{DrawOptions, DrawTarget, Point, SolidSource, Source};
use std::io::Result;
use std::time::SystemTime;
use time::format_description::FormatItem;
use time::macros::{datetime, format_description, offset};
use time::OffsetDateTime;
use tokio::runtime::Runtime;
use tokio::time::{sleep, Duration};

loupedeck::export_plugin!("time-plugin", register);

const TIME_FORMAT: &[FormatItem] = format_description!("[hour repr:12 padding:none]:[minute]");

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
    let time_plugin = TimeDisplayPlugin {
        ctx,
        runtime: Runtime::new().unwrap(),
    };

    time_plugin.start();

    Box::new(time_plugin)
}

fn create_date_plugin(ctx: loupedeck::PluginScreenContext) -> Box<dyn ScreenPlugin> {
    let date_plugin = DateDisplayPlugin {
        ctx,
        runtime: Runtime::new().unwrap(),
    };

    date_plugin.start();

    Box::new(date_plugin)
}

#[derive(Debug)]
pub struct TimeDisplayPlugin {
    runtime: Runtime,
    ctx: loupedeck::PluginScreenContext,
}

fn measure_text_width(font: &Font, text: String, pt_size: f32) -> f32 {
    let mut start = vec2f(0., 0.);

    for c in text.chars() {
        let id = font.glyph_for_char(c).unwrap();
        start += font.advance(id).unwrap() * pt_size / 24. / 96.;
    }

    start.x()
}

impl TimeDisplayPlugin {
    fn start(&self) {
        let ctx = self.ctx.clone();

        self.runtime.spawn(async move {
            loop {
                let mut current_time: OffsetDateTime = SystemTime::now().into();
                current_time = current_time.to_offset(offset!(-7));
                let time_str = current_time.format(TIME_FORMAT).unwrap();

                let key = convert_draw_target_to_rgb565(draw_text_key(time_str));
                ctx.draw_rgb565(key).await;
                sleep(Duration::from_secs(1)).await;
            }
        });
    }
}

impl ScreenPlugin for TimeDisplayPlugin {
    fn on_touch(&self, _position: loupedeck::TouchEvent) -> Result<()> {
        let ctx = self.ctx.clone();
        let key = convert_draw_target_to_rgb565(draw_text_key("AAABBBCCC".to_string()));

        self.runtime.spawn(async move {
            ctx.vibrate(loupedeck::Haptic::Medium).await;
            ctx.draw_rgb565(key).await;
        });

        Ok(())
    }
}

#[derive(Debug)]
pub struct DateDisplayPlugin {
    runtime: Runtime,
    ctx: loupedeck::PluginScreenContext,
}

impl DateDisplayPlugin {
    fn start(&self) {
        let ctx = self.ctx.clone();

        self.runtime.spawn(async move {
            let key = convert_draw_target_to_rgb565(draw_text_key("Date".to_string()));
            ctx.draw_rgb565(key).await;
        });
    }
}

impl ScreenPlugin for DateDisplayPlugin {
    fn on_touch(&self, _position: loupedeck::TouchEvent) -> Result<()> {
        Ok(())
    }
}

fn draw_text_key(text: String) -> DrawTarget {
    let mut dt = DrawTarget::new(90, 90);

    let font_size: f32 = 14.;

    let solidWhite: SolidSource = SolidSource::from_unpremultiplied_argb(255, 0xFF, 0xFF, 0xFF);
    let solidBlack: SolidSource = SolidSource::from_unpremultiplied_argb(255, 0x00, 0x00, 0x00);

    dt.fill_rect(
        0.0,
        0.0,
        90.0,
        90.0,
        &Source::Solid(solidBlack),
        &DrawOptions::new(),
    );

    let font = SystemSource::new()
        .select_best_match(
            &[FamilyName::Monospace],
            &Properties::new().weight(Weight::BOLD),
        )
        .unwrap()
        .load()
        .unwrap();

    let font_width: f32 = measure_text_width(&font, text.clone(), font_size);
    let mut start_x: f32 = 0.;

    if font_width < 90. {
        let diff: f32 = 90. - font_width;
        let half = diff / 2.;
        start_x = half;
    }

    dt.draw_text(
        &font,
        font_size,
        text.as_str(),
        Point::new(start_x, 45.0),
        &Source::Solid(solidWhite),
        &DrawOptions::new(),
    );

    return dt;
}
