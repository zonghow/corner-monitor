use std::sync::Mutex;

use tauri::{Emitter, Manager};

use crate::monitor::{Monitor, SystemInfo};
use crate::state::{
    layout_to_str, Layout, MonitorVisibility, SettingsStore, UiState, WindowPosition, KEY_LAYOUT,
    KEY_MONITOR_TARGET, SIZE_HORIZONTAL, SIZE_VERTICAL,
};
use crate::tray::{snap_window_to_nearest_corner, update_layout, TrayMenuItems};
use crate::window::{apply_window_position, calculate_window_position_on_monitor, monitor_for_window};

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
pub fn get_system_info(monitor: tauri::State<'_, Mutex<Monitor>>) -> Result<SystemInfo, String> {
    monitor
        .lock()
        .map(|state| state.get_system_info())
        .map_err(|_| "monitor lock poisoned".to_string())
}

#[tauri::command]
pub fn get_layout(state: tauri::State<'_, Mutex<UiState>>) -> String {
    state
        .lock()
        .map(|ui_state| layout_to_str(ui_state.layout).to_string())
        .unwrap_or_else(|_| "vertical".to_string())
}

#[tauri::command]
pub fn get_monitor_visibility(state: tauri::State<'_, Mutex<UiState>>) -> MonitorVisibility {
    state
        .lock()
        .map(|ui_state| crate::state::visibility_from_state(&ui_state))
        .unwrap_or(MonitorVisibility {
            cpu: true,
            mem: true,
            net: true,
        })
}

#[tauri::command]
pub fn get_text_color(state: tauri::State<'_, Mutex<UiState>>) -> String {
    state
        .lock()
        .map(|ui_state| ui_state.text_color.clone())
        .unwrap_or_else(|_| "#ffffff".to_string())
}

#[tauri::command]
pub fn snap_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        snap_window_to_nearest_corner(&app, &window).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_layout(app: tauri::AppHandle) -> Result<(), String> {
    let current_layout = app
        .state::<Mutex<UiState>>()
        .lock()
        .map(|state| state.layout)
        .unwrap_or(Layout::Vertical);
    let next_layout = match current_layout {
        Layout::Horizontal => Layout::Vertical,
        Layout::Vertical => Layout::Horizontal,
    };
    if let Some(tray) = app.try_state::<TrayMenuItems>() {
        update_layout(&app, next_layout, &tray);
        return Ok(());
    }
    let mut changed = true;
    if let Ok(mut state) = app.state::<Mutex<UiState>>().lock() {
        changed = state.layout != next_layout;
        state.layout = next_layout;
    }
    let store = app.state::<SettingsStore>();
    store.set(KEY_LAYOUT, layout_to_str(next_layout).to_string());
    let payload = layout_to_str(next_layout);
    let _ = app.emit("layout-changed", payload);
    if !changed {
        return Ok(());
    }
    if let Some(window) = app.get_webview_window("main") {
        let target = match next_layout {
            Layout::Horizontal => SIZE_HORIZONTAL,
            Layout::Vertical => SIZE_VERTICAL,
        };
        let _ = window.set_size(target);
        let position = match app.state::<Mutex<UiState>>().lock() {
            Ok(state) => state.position,
            Err(_) => WindowPosition::TopLeft,
        };
        if let Some(monitor) = monitor_for_window(&app, &window) {
            if let Ok(target_pos) =
                calculate_window_position_on_monitor(&app, &window, position, &monitor)
            {
                let _ = window.set_position(target_pos);
            }
            let monitor_target = crate::state::monitor_target_from_monitor(&app, &monitor);
            if let Ok(mut state) = app.state::<Mutex<UiState>>().lock() {
                state.monitor_target = monitor_target.clone();
            }
            if let Some(target) = monitor_target {
                store.set(KEY_MONITOR_TARGET, crate::state::monitor_target_to_str(&target));
            }
        } else {
            let _ = apply_window_position(&app, &window, position);
        }
    }
    Ok(())
}
