#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[global_allocator]
static ALLOCATOR: System = System;

use libloading::Library;
use loupedeck::{
    connect_loupedeck_device, get_loupedeck_ports, ButtonPlugin, KnobPlugin, PluginDeclaration,
    ScreenPlugin,
};
use platform_dirs::AppDirs;
use serde::Serialize;
use std::fs::{self};
use std::io::Result;
use std::sync::Arc;
use std::{alloc::System, collections::HashMap, env, ffi::OsStr, io, sync::Mutex};
use tauri::utils::assets::EmbeddedAssets;
use tauri::{CustomMenuItem, Manager, State, SystemTray, SystemTrayMenu, SystemTrayMenuItem};

pub struct ScreenPluginProxy {
    plugin: Box<dyn ScreenPlugin>,
    _lib: Arc<Library>,
}

pub struct ButtonPluginProxy {
    plugin: Box<dyn ButtonPlugin>,
    _lib: Arc<Library>,
}

unsafe impl Send for ScreenPluginProxy {}
unsafe impl Sync for ScreenPluginProxy {}

pub struct KnobPluginProxy {
    plugin: Box<dyn KnobPlugin>,
    _lib: Arc<Library>,
}

impl KnobPlugin for KnobPluginProxy {
    fn create(&self, position: loupedeck::Knob) -> Result<()> {
        self.plugin.create(position)
    }

    fn destroy(&self, position: loupedeck::Knob) -> Result<()> {
        self.plugin.destroy(position)
    }

    fn on_change(&self, position: loupedeck::KnobRotateEvent) -> Result<&str> {
        self.plugin.on_change(position)
    }
}

impl ButtonPlugin for ButtonPluginProxy {
    fn create(&self, position: loupedeck::Button) -> Result<()> {
        self.plugin.create(position)
    }

    fn destroy(&self, position: loupedeck::Button) -> Result<()> {
        self.plugin.destroy(position)
    }

    fn on_change(&self, position: loupedeck::ButtonPressEvent) -> Result<()> {
        self.plugin.on_change(position)
    }
}

impl ScreenPlugin for ScreenPluginProxy {
    fn create(&self, position: loupedeck::Screen) -> Result<()> {
        self.plugin.create(position)
    }

    fn destroy(&self, position: loupedeck::Screen) -> Result<()> {
        self.plugin.destroy(position)
    }

    fn on_touch(&self, position: loupedeck::TouchEvent) -> Result<()> {
        self.plugin.on_touch(position)
    }
}

struct PluginRegistrar {
    knobs: HashMap<String, KnobPluginProxy>,
    buttons: HashMap<String, ButtonPluginProxy>,
    screens: HashMap<String, ScreenPluginProxy>,
    lib: Arc<Library>,
}

impl PluginRegistrar {
    pub fn new(lib: Arc<Library>) -> PluginRegistrar {
        PluginRegistrar {
            knobs: HashMap::default(),
            buttons: HashMap::default(),
            screens: HashMap::default(),
            lib,
        }
    }
}

impl loupedeck::PluginRegistrar for PluginRegistrar {
    fn register_knob(&mut self, name: &str, plugin: Box<dyn KnobPlugin>) {
        let proxy = KnobPluginProxy {
            plugin: plugin,
            _lib: Arc::clone(&self.lib),
        };

        self.knobs.insert(name.to_string(), proxy);
    }

    fn register_button(&mut self, name: &str, plugin: Box<dyn ButtonPlugin>) {
        let proxy = ButtonPluginProxy {
            plugin: plugin,
            _lib: Arc::clone(&self.lib),
        };

        self.buttons.insert(name.to_string(), proxy);
    }

    fn register_screen(&mut self, name: &str, plugin: Box<dyn ScreenPlugin>) {
        let proxy = ScreenPluginProxy {
            plugin: plugin,
            _lib: Arc::clone(&self.lib),
        };

        self.screens.insert(name.to_string(), proxy);
    }
}

#[derive(Default)]
pub struct ExternalPlugins {
    knobs: HashMap<String, KnobPluginProxy>,
    buttons: HashMap<String, ButtonPluginProxy>,
    screens: HashMap<String, ScreenPluginProxy>,
    libraries: Vec<Arc<Library>>,
}

impl ExternalPlugins {
    pub fn new() -> ExternalPlugins {
        ExternalPlugins::default()
    }

    pub unsafe fn load<P: AsRef<OsStr>>(&mut self, library_path: P) -> Result<()> {
        let library = Arc::new(Library::new(library_path)?);
        let decl = library
            .get::<*mut PluginDeclaration>(b"plugin_declaration\0")?
            .read();

        if decl.rustc_version != loupedeck::RUSTC_VERSION
            || decl.core_version != loupedeck::CORE_VERSION
        {
            return Err(io::Error::new(io::ErrorKind::Other, "Version mismatch"));
        }

        let mut registrar = PluginRegistrar::new(Arc::clone(&library));
        (decl.register)(&mut registrar);

        // add all loaded plugins to the functions map
        self.knobs.extend(registrar.knobs);
        self.buttons.extend(registrar.buttons);
        self.screens.extend(registrar.screens);

        // and make sure ExternalPlugins keeps a reference to the library
        self.libraries.push(library);

        return Ok(());
    }
}

struct ConnectionState {
    connected_ld: Mutex<Option<loupedeck::Device>>,
    // external_plugins: Mutex<ExternalPlugins>,
}

#[tauri::command]
fn list_ld_ports() -> Vec<String> {
    return get_loupedeck_ports();
}

#[derive(Debug, Serialize, Clone, PartialEq)]
enum DeviceConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
struct DeviceConnectionEvent {
    status: DeviceConnectionStatus,
}

#[tauri::command]
fn get_connection_status(state: tauri::State<ConnectionState>) -> DeviceConnectionStatus {
    let connected_ld = state.connected_ld.lock();

    if connected_ld.is_ok() {
        if connected_ld.unwrap().is_some() {
            return DeviceConnectionStatus::Connected;
        }
    }

    return DeviceConnectionStatus::Disconnected;
}

#[tauri::command]
fn vibrate(state: tauri::State<ConnectionState>) {
    let connected_ld = state.connected_ld.lock();

    if connected_ld.is_ok() {
        let ld = connected_ld.unwrap();

        if ld.is_some() {
            let ldu = ld.as_ref().unwrap();
            ldu.vibrate(loupedeck::Haptic::Low);
        }
    }
}

#[tauri::command]
fn connect_ld(port: String, state: tauri::State<ConnectionState>, window: tauri::Window) {
    let app = window.app_handle();

    window.emit_all(
        "device-connection-status",
        DeviceConnectionEvent {
            status: DeviceConnectionStatus::Connecting,
        },
    );

    let instance = connect_loupedeck_device(port, move |evt| {
        app.emit_all("event-update", evt);
    });

    let mut connected_ld = state.connected_ld.lock().unwrap();
    *connected_ld = Some(instance);

    window.emit_all(
        "device-connection-status",
        DeviceConnectionEvent {
            status: DeviceConnectionStatus::Connected,
        },
    );
}

fn load_plugins() -> ExternalPlugins {
    let mut ext_plugin = ExternalPlugins::new();

    let app_dirs = AppDirs::new(Some("loupedeck-rs"), true).unwrap();
    let plugins_dir = app_dirs.config_dir.join("plugins");
    fs::create_dir_all(plugins_dir.clone()).unwrap();

    let plugin_files = fs::read_dir(plugins_dir.clone()).unwrap();

    println!("Loading plugins from {:?}", plugins_dir.as_path().display());

    for plugin_file in plugin_files {
        let plugin_file_path = plugin_file.unwrap().path();
        let display = plugin_file_path.display();

        if plugin_file_path.is_file() {
            unsafe {
                let plugin_load_result = ext_plugin.load(plugin_file_path.clone());

                match plugin_load_result {
                    Ok(_) => {
                        println!("Loaded plugin: {}", display);
                    }
                    Err(e) => {
                        println!("Failed to load plugin: {}", display);
                    }
                }
            }
        }
    }

    return ext_plugin;
}

fn build_window(context: &tauri::Context<EmbeddedAssets>) -> tauri::Builder<tauri::Wry> {
    let connection_state = ConnectionState {
        connected_ld: Mutex::new(None),
    };

    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(hide);
    let tray = SystemTray::new().with_menu(tray_menu);

    return tauri::Builder::default()
        .manage(connection_state)
        .system_tray(tray)
        .on_window_event(|window_event| {
            let event = window_event.event();

            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    println!("Close requested");
                    println!("{:?}", api);

                    let window = window_event.window();

                    let s: State<ConnectionState> = window.state();

                    let mut locked_state = s.connected_ld.lock().unwrap();

                    match &mut *locked_state {
                        Some(ld) => {
                            ld.disconnect();
                        }
                        None => {}
                    }
                }

                _ => {}
            }
        })
        .menu(tauri::Menu::os_default(&context.package_info().name))
        .invoke_handler(tauri::generate_handler![
            list_ld_ports,
            connect_ld,
            get_connection_status,
            vibrate
        ]);
}

mod plugins {
    use loupedeck::ScreenPlugin;
    use tauri::{
        plugin::{Builder as PluginBuilder, TauriPlugin},
        Manager, RunEvent, Runtime,
    };

    pub fn init<R: Runtime>() -> TauriPlugin<R> {
        PluginBuilder::new("window")
            .setup(|app| {
                let plugins = crate::load_plugins();

                app.listen_global("trigger-screen-plugin", move |evt| {
                    println!("{:?}", evt);
                    let name = evt.payload();
                    if name.is_some() {
                        let plugin_name = name.unwrap();
                        println!("plugin_name {:?}", name);
                        println!("plugins {:?}", plugins.screens.keys());

                        let plugin = plugins.screens.get(plugin_name);
                        if plugin.is_some() {
                            let plugin = plugin.unwrap();
                            plugin.create(loupedeck::Screen::Center);
                        }
                    }
                });

                // initialize the plugin here
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![])
            .build()
    }
}

fn main() {
    let context = tauri::generate_context!();

    let app_builder = build_window(&context).plugin(plugins::init());

    app_builder.run(context);
}

mod ld_controller {
    use std::{collections::HashMap, hash::Hash};

    use loupedeck::ButtonPlugin;

    use crate::PluginRegistrar;

    struct PageConfig {
        id: u16,
        name: String,
        buttons: HashMap<loupedeck::Button, String>,
        knobs: HashMap<loupedeck::Knob, String>,
        screens: HashMap<loupedeck::Screen, String>,
    }

    struct Page {
        config: PageConfig,
        name: String,
        id: u16,
        buttons: HashMap<loupedeck::Button, String>,
        knobs: HashMap<loupedeck::Knob, String>,
        screens: HashMap<loupedeck::Screen, String>,
    }

    impl Page {
        fn new(config: PageConfig, registrar: PluginRegistrar) -> Page {
            let buttons = HashMap::new();
            let knobs = HashMap::new();
            let screens = HashMap::new();

            for (button, plugin_name) in config.buttons.iter() {
                let plugin = registrar.buttons.get(plugin_name);
                if plugin.is_some() {
                    let plugin = plugin.unwrap();
                    plugin.create(*button);
                    buttons.insert(*button, plugin);
                }
            }

            return Page {
                name: config.name.clone(),
                id: config.id.clone(),
                buttons,
                knobs,
                screens,
                config,
            };
        }
    }

    struct LDController {
        ld: loupedeck::Device,
        plugin_registrar: PluginRegistrar,
        pages: HashMap<u16, Page>,
    }

    impl LDController {
        fn new(ld: loupedeck::Device, plugin_registrar: PluginRegistrar) -> LDController {
            let pages = HashMap::new();
            return LDController {
                ld,
                plugin_registrar,
                pages,
            };
        }
    }
}
