// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod monitor;
mod state;
mod tray;
mod window;

use std::sync::Mutex;
use std::time::Duration;

use monitor::{Monitor, MonitorConfig};
use tauri::{Emitter, Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_store::StoreBuilder;

use crate::commands::{
    get_layout, get_monitor_visibility, get_system_info, get_text_color, greet, snap_window,
    toggle_layout,
};
use crate::state::{
    layout_from_str, layout_to_str, position_from_str, position_to_str, primary_monitor_target,
    visibility_from_state, UiState, KEY_LAYOUT, KEY_MONITOR_CPU, KEY_MONITOR_MEM, KEY_MONITOR_NET,
    KEY_MONITOR_TARGET, KEY_POSITION, KEY_TEXT_COLOR, SETTINGS_PATH,
};
use crate::tray::setup_tray;
use crate::window::apply_layout_and_position;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                let _ = app.handle().set_dock_visibility(false);
            }

            let store = StoreBuilder::new(app, SETTINGS_PATH).build()?;
            let mut ui_state = UiState::default();
            if let Some(value) = store.get(KEY_POSITION) {
                if let Some(value) = value.as_str() {
                    if let Some(position) = position_from_str(value) {
                        ui_state.position = position;
                    }
                }
            }
            if let Some(value) = store.get(KEY_LAYOUT) {
                if let Some(value) = value.as_str() {
                    if let Some(layout) = layout_from_str(value) {
                        ui_state.layout = layout;
                    }
                }
            }
            if let Some(value) = store.get(KEY_TEXT_COLOR) {
                if let Some(value) = value.as_str() {
                    ui_state.text_color = value.to_string();
                }
            }
            if let Some(value) = store.get(KEY_MONITOR_TARGET) {
                if let Some(value) = value.as_str() {
                    ui_state.monitor_target = crate::state::monitor_target_from_str(value);
                }
            }
            if ui_state.monitor_target.is_none() {
                ui_state.monitor_target = primary_monitor_target(&app.handle());
            }
            if let Some(value) = store.get(KEY_MONITOR_CPU) {
                if let Some(value) = value.as_bool() {
                    ui_state.show_cpu = value;
                }
            }
            if let Some(value) = store.get(KEY_MONITOR_MEM) {
                if let Some(value) = value.as_bool() {
                    ui_state.show_mem = value;
                }
            }
            if let Some(value) = store.get(KEY_MONITOR_NET) {
                if let Some(value) = value.as_bool() {
                    ui_state.show_net = value;
                }
            }
            if !(ui_state.show_cpu || ui_state.show_mem || ui_state.show_net) {
                ui_state.show_cpu = true;
            }
            store.set(KEY_POSITION, position_to_str(ui_state.position).to_string());
            store.set(KEY_LAYOUT, layout_to_str(ui_state.layout).to_string());
            store.set(KEY_TEXT_COLOR, ui_state.text_color.clone());
            if let Some(target) = &ui_state.monitor_target {
                store.set(KEY_MONITOR_TARGET, crate::state::monitor_target_to_str(target));
            }
            store.set(KEY_MONITOR_CPU, ui_state.show_cpu);
            store.set(KEY_MONITOR_MEM, ui_state.show_mem);
            store.set(KEY_MONITOR_NET, ui_state.show_net);
            app.manage(store);
            app.manage(Mutex::new(ui_state.clone()));

            let monitor = Monitor::new(
                MonitorConfig::new()
                    .cpu_interval(Duration::from_secs(1))
                    .memory_interval(Duration::from_secs(1))
                    .disk_interval(Duration::from_secs(30))
                    .network_interval(Duration::from_secs(1)),
            );
            monitor.refresh_all();
            monitor.start();
            app.manage(Mutex::new(monitor));

            if let Some(window) = app.get_webview_window("main") {
                let handle = app.handle();
                apply_layout_and_position(&handle, &window);
                let _ = window.set_shadow(true);
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }

            let tray_items = setup_tray(&app.handle(), &ui_state)?;
            app.manage(tray_items.clone());

            let _ = app.emit("layout-changed", layout_to_str(ui_state.layout));
            let _ = app.emit("text-color-changed", ui_state.text_color.clone());
            let _ = app.emit(
                "monitor-visibility-changed",
                visibility_from_state(&ui_state),
            );
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_system_info,
            get_layout,
            get_monitor_visibility,
            get_text_color,
            snap_window,
            toggle_layout
        ])
        .on_window_event(|window, event| match event {
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                let app = window.app_handle().clone();
                if let Some(webview) = app.get_webview_window("main") {
                    apply_layout_and_position(&app, &webview);
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
