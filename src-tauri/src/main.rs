// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate log;

use aw_server::endpoints::build_rocket;
use awatcher::{
    config::Config, report_client::ReportClient, watchers, watchers::ConstructorFilter,
};
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use std::{
    error::Error,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};
use tauri::{AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};

fn main() {
    tauri::Builder::default()
        .system_tray(get_system_tray())
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => {
                show_hide_window(app);
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
                if id == "quit" {
                    app.exit(0)
                } else if id == "show_hide" {
                    show_hide_window(app);
                }
            }
            _ => (),
        })
        .setup(|app| {
            setup_logger(LevelFilter::Info)?;

            let resource_path = app
                .path_resolver()
                .resolve_resource("../aw-webui/dist")
                .expect("failed to resolve resource");

            run_server(resource_path);
            run_watchers()?;

            let main_window = app.get_window("main").unwrap();
            main_window.eval(&format!(
                "window.location.replace('http://localhost:{}')",
                "5600"
            ))?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the application");
}

fn show_hide_window(app: &AppHandle) {
    let window = app.get_window("main").unwrap();
    let tray_item = app.tray_handle().get_item("show_hide");
    if let Err(e) = if window.is_visible().unwrap_or(false) {
        tray_item.set_title("Show").and(window.hide())
    } else {
        tray_item.set_title("Hide").and(window.show())
    } {
        println!("Window state cannot be changed: {e}");
    }
}

fn get_system_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let open = CustomMenuItem::new("show_hide".to_string(), "Hide");
    let tray_menu = SystemTrayMenu::new().add_item(open).add_item(quit);

    SystemTray::new().with_menu(tray_menu)
}

fn run_server(asset_path: PathBuf) {
    let db_path = aw_server::dirs::db_path(false)
        .expect("Failed to get db path")
        .to_str()
        .unwrap()
        .to_string();
    let device_id = aw_server::device_id::get_device_id();
    let mut config = aw_server::config::create_config(false);
    config.address = "127.0.0.1".to_string();
    config.port = 5600;

    let legacy_import = false;
    let server_state = aw_server::endpoints::ServerState {
        datastore: Mutex::new(aw_datastore::Datastore::new(db_path, legacy_import)),
        asset_path,
        device_id,
    };

    tauri::async_runtime::spawn(build_rocket(server_state, config).launch());
}

fn run_watchers() -> Result<(), Box<dyn Error>> {
    let mut config = Config::from_cli()?;
    config.port = 5600;
    config.host = "localhost".to_string();

    let client = ReportClient::new(config)?;
    let client = Arc::new(client);

    let mut thread_handlers = Vec::new();

    let idle_watcher = watchers::IDLE.filter_first_supported();
    if let Some(mut watcher) = idle_watcher {
        let thread_client = Arc::clone(&client);
        let idle_handler = thread::spawn(move || watcher.watch(&thread_client));
        thread_handlers.push(idle_handler);
    } else {
        warn!("No supported idle handler is found");
    }

    let window_watcher = watchers::ACTIVE_WINDOW.filter_first_supported();
    if let Some(mut watcher) = window_watcher {
        let thread_client = Arc::clone(&client);
        let active_window_handler = thread::spawn(move || watcher.watch(&thread_client));
        thread_handlers.push(active_window_handler);
    } else {
        warn!("No supported active window handler is found");
    }

    Ok(())
}

fn setup_logger(verbosity: LevelFilter) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            let colors = ColoredLevelConfig::new()
                .info(Color::Green)
                .debug(Color::Blue)
                .trace(Color::Cyan);
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.6f"),
                colors.color(record.level()),
                record.target(),
                message
            ));
        })
        .level(log::LevelFilter::Error)
        .level_for("awatcher", verbosity)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
