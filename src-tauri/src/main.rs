#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use std::sync::Mutex;
use tauri::SystemTray;
use tauri::{CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem};
use tauri::{Manager, Runtime, State, Window};

struct App {
    connected_ld: Mutex<Vec<Option<loupedeck::loupedeck::Device>>>,
}

#[tauri::command]
fn list_ld_ports() -> Vec<String> {
    return loupedeck::loupedeck::get_loupedeck_ports();
}

#[tauri::command]
fn connect_ld(port: String, state: tauri::State<App>, window: tauri::Window) {
    let instance = loupedeck::loupedeck::connect_loupedeck_device(port, move |evt| {
        window.emit("event-update", evt);
    });

    let mut connected_ld = state.connected_ld.lock().unwrap();
    connected_ld.push(Some(instance));
}

#[tauri::command]
fn test_state(state: tauri::State<App>, window: tauri::Window) {
    window.emit("state-update", state.connected_ld.lock().unwrap().len());

    let connected_ld = state.connected_ld.lock().unwrap();
    println!("{:?}", connected_ld.len());
}

fn main() {
    let context = tauri::generate_context!();

    let app = App {
        connected_ld: Mutex::new(Vec::new()),
    };

    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(hide);
    let tray = SystemTray::new().with_menu(tray_menu);

    let appResult = tauri::Builder::default()
        .manage(app)
        .system_tray(tray)
        .on_window_event(|windowEvent| {
            let event = windowEvent.event();

            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    println!("Close requested");
                    println!("{:?}", api);

                    let mut window = windowEvent.window();

                    let s: State<App> = window.state();

                    let lockedState = &mut s.connected_ld.lock().unwrap();

                    for ld in lockedState.iter_mut() {
                        if let Some(ld) = ld {
                            ld.disconnect();
                        }
                    }
                }

                _ => {}
            }
        })
        .menu(tauri::Menu::os_default(&context.package_info().name))
        .invoke_handler(tauri::generate_handler![
            list_ld_ports,
            connect_ld,
            test_state
        ])
        .run(context);

    if let Err(e) = appResult {}
}
