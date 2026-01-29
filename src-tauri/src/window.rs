use std::sync::Mutex;
use tauri::{Manager, PhysicalPosition, PhysicalSize};

use crate::state::{
    monitor_target_from_monitor, monitor_target_to_str, Layout, SettingsStore, UiState,
    WindowPosition, KEY_MONITOR_TARGET, SIZE_HORIZONTAL, SIZE_VERTICAL,
};

fn desired_position(
    monitor_pos: PhysicalPosition<i32>,
    monitor_size: PhysicalSize<u32>,
    window_size: PhysicalSize<u32>,
    position: WindowPosition,
) -> PhysicalPosition<i32> {
    let min_x = monitor_pos.x;
    let min_y = monitor_pos.y;
    let max_x = monitor_pos.x + monitor_size.width as i32 - window_size.width as i32;
    let max_y = monitor_pos.y + monitor_size.height as i32 - window_size.height as i32;

    let x = match position {
        WindowPosition::TopLeft | WindowPosition::BottomLeft => min_x,
        WindowPosition::TopRight | WindowPosition::BottomRight => max_x,
    };
    let y = match position {
        WindowPosition::TopLeft | WindowPosition::TopRight => min_y,
        WindowPosition::BottomLeft | WindowPosition::BottomRight => max_y,
    };

    let final_x = if max_x < min_x { min_x } else { x };
    let final_y = if max_y < min_y { min_y } else { y };

    PhysicalPosition::new(final_x, final_y)
}

fn layout_window_size(
    app: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
) -> tauri::Result<PhysicalSize<u32>> {
    let layout = app
        .state::<Mutex<UiState>>()
        .lock()
        .map(|state| state.layout)
        .unwrap_or(Layout::Vertical);
    let logical = match layout {
        Layout::Horizontal => SIZE_HORIZONTAL,
        Layout::Vertical => SIZE_VERTICAL,
    };
    let scale = window.scale_factor()?;
    let width = (logical.width * scale).round() as u32;
    let height = (logical.height * scale).round() as u32;
    Ok(PhysicalSize::new(width, height))
}

pub fn calculate_window_position_on_monitor(
    app: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    position: WindowPosition,
    monitor: &tauri::Monitor,
) -> tauri::Result<PhysicalPosition<i32>> {
    let monitor_pos = *monitor.position();
    let monitor_size = *monitor.size();
    let window_size = match layout_window_size(app, window) {
        Ok(size) => size,
        Err(_) => window.outer_size()?,
    };
    Ok(desired_position(
        monitor_pos,
        monitor_size,
        window_size,
        position,
    ))
}

pub fn selected_monitor(app: &tauri::AppHandle) -> Option<tauri::Monitor> {
    let target = app
        .state::<Mutex<UiState>>()
        .lock()
        .ok()
        .and_then(|state| state.monitor_target.clone())?;
    let monitors = app.available_monitors().ok()?;
    if let Some(monitor) = monitors.get(target.index) {
        return Some(monitor.clone());
    }
    if let Some(name) = &target.name {
        return monitors
            .iter()
            .find(|monitor| monitor.name().map(|value| value == name).unwrap_or(false))
            .cloned();
    }
    None
}

pub fn monitor_for_window(
    app: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
) -> Option<tauri::Monitor> {
    if let (Ok(position), Ok(size)) = (window.outer_position(), window.outer_size()) {
        if let Ok(monitors) = app.available_monitors() {
            let mut best: Option<tauri::Monitor> = None;
            let mut best_area: i64 = -1;
            for monitor in &monitors {
                let area_pos = *monitor.position();
                let area_size = *monitor.size();
                let window_right = position.x + size.width as i32;
                let window_bottom = position.y + size.height as i32;
                let area_right = area_pos.x + area_size.width as i32;
                let area_bottom = area_pos.y + area_size.height as i32;
                let overlap_x = (window_right.min(area_right) - position.x.max(area_pos.x)).max(0);
                let overlap_y =
                    (window_bottom.min(area_bottom) - position.y.max(area_pos.y)).max(0);
                let overlap_area = overlap_x as i64 * overlap_y as i64;
                if overlap_area > best_area {
                    best_area = overlap_area;
                    best = Some(monitor.clone());
                }
            }
            if let Some(best) = best {
                if best_area > 0 {
                    return Some(best);
                }
            }
        }

        let center_x = position.x as f64 + size.width as f64 / 2.0;
        let center_y = position.y as f64 + size.height as f64 / 2.0;
        if let Ok(monitor) = app.monitor_from_point(center_x, center_y) {
            if let Some(monitor) = monitor {
                return Some(monitor);
            }
        }
    }
    window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten())
}

pub fn nearest_corner(
    monitor_pos: PhysicalPosition<i32>,
    monitor_size: PhysicalSize<u32>,
    window_size: PhysicalSize<u32>,
    current_pos: PhysicalPosition<i32>,
) -> (WindowPosition, PhysicalPosition<i32>) {
    let candidates = [
        (
            WindowPosition::TopLeft,
            desired_position(monitor_pos, monitor_size, window_size, WindowPosition::TopLeft),
        ),
        (
            WindowPosition::TopRight,
            desired_position(monitor_pos, monitor_size, window_size, WindowPosition::TopRight),
        ),
        (
            WindowPosition::BottomLeft,
            desired_position(monitor_pos, monitor_size, window_size, WindowPosition::BottomLeft),
        ),
        (
            WindowPosition::BottomRight,
            desired_position(
                monitor_pos,
                monitor_size,
                window_size,
                WindowPosition::BottomRight,
            ),
        ),
    ];

    let mut best = candidates[0];
    let mut best_distance = i64::MAX;
    for candidate in candidates {
        let dx = current_pos.x as i64 - candidate.1.x as i64;
        let dy = current_pos.y as i64 - candidate.1.y as i64;
        let distance = dx * dx + dy * dy;
        if distance < best_distance {
            best_distance = distance;
            best = candidate;
        }
    }
    best
}

pub fn calculate_window_position(
    app: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    position: WindowPosition,
) -> tauri::Result<PhysicalPosition<i32>> {
    let monitor = selected_monitor(app)
        .or_else(|| window.current_monitor().ok().flatten())
        .or_else(|| window.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else {
        return Ok(PhysicalPosition::new(0, 0));
    };
    calculate_window_position_on_monitor(app, window, position, &monitor)
}

pub fn apply_window_position(
    app: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    position: WindowPosition,
) -> tauri::Result<()> {
    let target = calculate_window_position(app, window, position)?;
    if let Ok(current) = window.outer_position() {
        if current.x == target.x && current.y == target.y {
            return Ok(());
        }
    }
    window.set_position(target)
}

pub fn apply_layout_and_position(app: &tauri::AppHandle, window: &tauri::WebviewWindow) {
    let (layout, position) = match app.state::<Mutex<UiState>>().lock() {
        Ok(state) => (state.layout, state.position),
        Err(_) => (Layout::Vertical, WindowPosition::TopLeft),
    };
    let target = match layout {
        Layout::Horizontal => SIZE_HORIZONTAL,
        Layout::Vertical => SIZE_VERTICAL,
    };
    let _ = window.set_size(target);
    if let Some(monitor) = monitor_for_window(app, window) {
        if let Ok(target_pos) = calculate_window_position_on_monitor(app, window, position, &monitor)
        {
            let _ = window.set_position(target_pos);
        }
        let monitor_target = monitor_target_from_monitor(app, &monitor);
        if let Ok(mut state) = app.state::<Mutex<UiState>>().lock() {
            state.monitor_target = monitor_target.clone();
        }
        if let Some(target) = monitor_target {
            let store = app.state::<SettingsStore>();
            store.set(KEY_MONITOR_TARGET, monitor_target_to_str(&target));
        }
    } else {
        let _ = apply_window_position(app, window, position);
    }
}
