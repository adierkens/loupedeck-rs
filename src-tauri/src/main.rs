#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[global_allocator]
static ALLOCATOR: System = System;

use futures::executor::block_on;
use loupedeck::{
    get_loupedeck_ports, Controller, DeviceConnectionStatus, PageConfig, PluginIdentifier,
};
use platform_dirs::AppDirs;
use serde::Serialize;
use std::fs;
use std::{alloc::System, env, sync::Mutex};
use tauri::utils::assets::EmbeddedAssets;
use tauri::{CustomMenuItem, Manager, State, SystemTray, SystemTrayMenu, SystemTrayMenuItem};

struct ConnectionState {
    controller: Mutex<Controller>,
}
#[derive(Debug, Serialize, Clone, PartialEq)]
struct DeviceConnectionEvent {
    status: DeviceConnectionStatus,
}

#[tauri::command]
fn list_ld_ports() -> Vec<String> {
    return get_loupedeck_ports();
}

#[tauri::command]
fn get_connection_status(state: tauri::State<ConnectionState>) -> DeviceConnectionStatus {
    let controller = state.controller.lock().unwrap();

    return controller.get_connection_status().unwrap();
}

#[tauri::command]
fn list_plugins(state: tauri::State<ConnectionState>) -> Vec<PluginIdentifier> {
    let controller = state.controller.lock().unwrap();
    let plugin = controller.list_plugins();

    if plugin.is_ok() {
        return plugin.unwrap();
    }

    return Vec::default();
}

#[tauri::command]
fn get_page_names(state: tauri::State<ConnectionState>) -> Vec<String> {
    let controller = state.controller.lock().unwrap();
    return controller.get_page_names();
}

#[tauri::command]
fn get_page_config(state: tauri::State<ConnectionState>, page_name: String) -> Option<PageConfig> {
    let controller = state.controller.lock().unwrap();
    return controller.get_page(page_name);
}

#[tauri::command]
fn set_page_config(state: tauri::State<ConnectionState>, page_config: PageConfig) {
    let mut controller = state.controller.lock().unwrap();
    controller.set_page(page_config);
}

#[tauri::command]
fn set_active_page(state: tauri::State<ConnectionState>, page_name: String) {
    let mut controller = state.controller.lock().unwrap();
    block_on(controller.set_current_page(page_name));
}

fn get_default_plugin() -> Vec<String> {
    let app_dirs = AppDirs::new(Some("loupedeck-rs"), true).unwrap();
    let plugins_dir = app_dirs.config_dir.join("plugins");
    fs::create_dir_all(plugins_dir.clone()).unwrap();

    let mut default_plugins = Vec::new();

    let plugin_files = fs::read_dir(plugins_dir.clone()).unwrap();

    println!("Loading plugins from {:?}", plugins_dir.as_path().display());

    for plugin_file in plugin_files {
        let plugin_file_path = plugin_file.unwrap().path();

        if plugin_file_path.is_file() {
            default_plugins.push(plugin_file_path.into_os_string().into_string().unwrap());
        }
    }

    return default_plugins;
}

#[tauri::command]
fn connect_ld(port: String, window: tauri::Window) {
    let app = window.app_handle();

    window.emit_all(
        "device-connection-status",
        DeviceConnectionEvent {
            status: DeviceConnectionStatus::Connecting,
        },
    );

    tauri::async_runtime::spawn(async move {
        let mut device = loupedeck::Device::new(port);

        let state: tauri::State<ConnectionState> = app.state();
        block_on(device.connect());

        let mut controller = state.controller.lock().unwrap();
        controller.start(device);

        for plugin in get_default_plugin() {
            let res = controller.load_plugin(&plugin);

            if res.is_err() {
                println!("Error loading plugin: {:?}", res.err());
            }
        }
    });
}

fn build_window(context: &tauri::Context<EmbeddedAssets>) -> tauri::Builder<tauri::Wry> {
    let connection_state = ConnectionState {
        controller: Mutex::new(Controller::new()),
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
                    let window = window_event.window();
                    let s: State<ConnectionState> = window.state();
                }
                _ => {}
            }
        })
        .menu(tauri::Menu::os_default(&context.package_info().name))
        .invoke_handler(tauri::generate_handler![
            list_ld_ports,
            get_connection_status,
            connect_ld,
            list_plugins,
            get_page_names,
            get_page_config,
            set_page_config,
            set_active_page
        ]);
}

fn main() {
    let context = tauri::generate_context!();

    let app_builder = build_window(&context);

    app_builder.run(context);
}
