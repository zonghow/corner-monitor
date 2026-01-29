use std::sync::Mutex;

use tauri::{
    menu::{CheckMenuItem, MenuBuilder, MenuItem, SubmenuBuilder},
    tray::TrayIconBuilder,
    Emitter, Manager, Wry,
};
use tauri_plugin_autostart::ManagerExt as AutoLaunchManagerExt;

use crate::state::{
    layout_to_str, monitor_target_from_monitor, monitor_target_to_str, position_to_str,
    visibility_from_state, Layout, MonitorItem, MonitorVisibility, SettingsStore, UiState,
    WindowPosition, COLOR_OPTIONS, KEY_LAYOUT, KEY_MONITOR_CPU, KEY_MONITOR_MEM,
    KEY_MONITOR_NET, KEY_MONITOR_TARGET, KEY_POSITION, KEY_TEXT_COLOR, SIZE_HORIZONTAL,
    SIZE_VERTICAL,
};
use crate::window::{
    apply_window_position, calculate_window_position_on_monitor, monitor_for_window, nearest_corner,
};

#[derive(Clone)]
pub struct TrayMenuItems {
    autostart: CheckMenuItem<Wry>,
    pos_top_left: CheckMenuItem<Wry>,
    pos_bottom_left: CheckMenuItem<Wry>,
    pos_top_right: CheckMenuItem<Wry>,
    pos_bottom_right: CheckMenuItem<Wry>,
    layout_horizontal: CheckMenuItem<Wry>,
    layout_vertical: CheckMenuItem<Wry>,
    color_items: Vec<ColorMenuItem>,
    monitor_cpu: CheckMenuItem<Wry>,
    monitor_mem: CheckMenuItem<Wry>,
    monitor_net: CheckMenuItem<Wry>,
}

#[derive(Clone)]
struct ColorMenuItem {
    value: &'static str,
    item: CheckMenuItem<Wry>,
}

impl TrayMenuItems {
    pub fn set_autostart(&self, enabled: bool) {
        let _ = self.autostart.set_checked(enabled);
    }

    pub fn set_position(&self, position: WindowPosition) {
        let _ = self
            .pos_top_left
            .set_checked(position == WindowPosition::TopLeft);
        let _ = self
            .pos_bottom_left
            .set_checked(position == WindowPosition::BottomLeft);
        let _ = self
            .pos_top_right
            .set_checked(position == WindowPosition::TopRight);
        let _ = self
            .pos_bottom_right
            .set_checked(position == WindowPosition::BottomRight);
    }

    pub fn set_layout(&self, layout: Layout) {
        let _ = self
            .layout_horizontal
            .set_checked(layout == Layout::Horizontal);
        let _ = self.layout_vertical.set_checked(layout == Layout::Vertical);
    }

    pub fn set_text_color(&self, color: &str) {
        for item in &self.color_items {
            let checked = item.value.eq_ignore_ascii_case(color);
            let _ = item.item.set_checked(checked);
        }
    }

    pub fn set_monitor_visibility(&self, visibility: MonitorVisibility) {
        let _ = self.monitor_cpu.set_checked(visibility.cpu);
        let _ = self.monitor_mem.set_checked(visibility.mem);
        let _ = self.monitor_net.set_checked(visibility.net);
    }
}

pub fn update_position(app: &tauri::AppHandle, position: WindowPosition, tray: &TrayMenuItems) {
    if let Ok(mut state) = app.state::<Mutex<UiState>>().lock() {
        state.position = position;
    }
    tray.set_position(position);
    let store = app.state::<SettingsStore>();
    store.set(KEY_POSITION, position_to_str(position).to_string());
    if let Some(window) = app.get_webview_window("main") {
        let _ = apply_window_position(app, &window, position);
    }
}

pub fn update_layout(app: &tauri::AppHandle, layout: Layout, tray: &TrayMenuItems) {
    let mut changed = true;
    if let Ok(mut state) = app.state::<Mutex<UiState>>().lock() {
        changed = state.layout != layout;
        state.layout = layout;
    }
    tray.set_layout(layout);
    let store = app.state::<SettingsStore>();
    store.set(KEY_LAYOUT, layout_to_str(layout).to_string());
    let payload = layout_to_str(layout);
    let _ = app.emit("layout-changed", payload);

    if !changed {
        return;
    }

    if let Some(window) = app.get_webview_window("main") {
        let target = match layout {
            Layout::Horizontal => SIZE_HORIZONTAL,
            Layout::Vertical => SIZE_VERTICAL,
        };
        let _ = window.set_size(target);

        let position = match app.state::<Mutex<UiState>>().lock() {
            Ok(state) => state.position,
            Err(_) => WindowPosition::TopLeft,
        };
        if let Some(monitor) = monitor_for_window(app, &window) {
            if let Ok(target_pos) =
                calculate_window_position_on_monitor(app, &window, position, &monitor)
            {
                let _ = window.set_position(target_pos);
            }
            let monitor_target = monitor_target_from_monitor(app, &monitor);
            if let Ok(mut state) = app.state::<Mutex<UiState>>().lock() {
                state.monitor_target = monitor_target.clone();
            }
            if let Some(target) = monitor_target {
                store.set(KEY_MONITOR_TARGET, monitor_target_to_str(&target));
            }
        } else {
            let _ = apply_window_position(app, &window, position);
        }
    }
}

pub fn update_text_color(app: &tauri::AppHandle, color: &str, tray: &TrayMenuItems) {
    if let Ok(mut state) = app.state::<Mutex<UiState>>().lock() {
        state.text_color = color.to_string();
    }
    tray.set_text_color(color);
    let store = app.state::<SettingsStore>();
    store.set(KEY_TEXT_COLOR, color.to_string());
    let _ = app.emit("text-color-changed", color);
}

pub fn update_monitor_visibility(app: &tauri::AppHandle, item: MonitorItem, tray: &TrayMenuItems) {
    let mut next = None;
    if let Ok(mut state) = app.state::<Mutex<UiState>>().lock() {
        let mut cpu = state.show_cpu;
        let mut mem = state.show_mem;
        let mut net = state.show_net;
        match item {
            MonitorItem::Cpu => cpu = !cpu,
            MonitorItem::Mem => mem = !mem,
            MonitorItem::Net => net = !net,
        }

        if !(cpu || mem || net) {
            tray.set_monitor_visibility(visibility_from_state(&state));
            return;
        }

        state.show_cpu = cpu;
        state.show_mem = mem;
        state.show_net = net;
        next = Some(visibility_from_state(&state));
    }

    if let Some(visibility) = next {
        tray.set_monitor_visibility(visibility);
        let store = app.state::<SettingsStore>();
        store.set(KEY_MONITOR_CPU, visibility.cpu);
        store.set(KEY_MONITOR_MEM, visibility.mem);
        store.set(KEY_MONITOR_NET, visibility.net);
        let _ = app.emit("monitor-visibility-changed", visibility);
    }
}

pub fn snap_window_to_nearest_corner(
    app: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
) -> tauri::Result<()> {
    let current_pos = window.outer_position()?;
    let current_size = window.outer_size()?;
    let Some(monitor) = monitor_for_window(app, window) else {
        return Ok(());
    };
    let monitor_pos = *monitor.position();
    let monitor_size = *monitor.size();
    let (corner, target_pos) =
        nearest_corner(monitor_pos, monitor_size, current_size, current_pos);

    if current_pos.x != target_pos.x || current_pos.y != target_pos.y {
        window.set_position(target_pos)?;
    }

    let target_monitor = monitor_target_from_monitor(app, &monitor);
    if let Ok(mut state) = app.state::<Mutex<UiState>>().lock() {
        state.position = corner;
        state.monitor_target = target_monitor.clone();
    }
    let store = app.state::<SettingsStore>();
    store.set(KEY_POSITION, position_to_str(corner).to_string());
    if let Some(target) = target_monitor {
        store.set(KEY_MONITOR_TARGET, monitor_target_to_str(&target));
    }

    if let Some(tray) = app.try_state::<TrayMenuItems>() {
        tray.set_position(corner);
    }
    Ok(())
}

pub fn setup_tray(app: &tauri::AppHandle, ui_state: &UiState) -> tauri::Result<TrayMenuItems> {
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    let autostart_item = CheckMenuItem::with_id(
        app,
        "autostart",
        "开机启动",
        true,
        autostart_enabled,
        None::<&str>,
    )?;

    let pos_top_left = CheckMenuItem::with_id(
        app,
        "pos_top_left",
        "左上",
        true,
        ui_state.position == WindowPosition::TopLeft,
        None::<&str>,
    )?;
    let pos_bottom_left = CheckMenuItem::with_id(
        app,
        "pos_bottom_left",
        "左下",
        true,
        ui_state.position == WindowPosition::BottomLeft,
        None::<&str>,
    )?;
    let pos_top_right = CheckMenuItem::with_id(
        app,
        "pos_top_right",
        "右上",
        true,
        ui_state.position == WindowPosition::TopRight,
        None::<&str>,
    )?;
    let pos_bottom_right = CheckMenuItem::with_id(
        app,
        "pos_bottom_right",
        "右下",
        true,
        ui_state.position == WindowPosition::BottomRight,
        None::<&str>,
    )?;

    let layout_horizontal = CheckMenuItem::with_id(
        app,
        "layout_horizontal",
        "水平",
        true,
        ui_state.layout == Layout::Horizontal,
        None::<&str>,
    )?;
    let layout_vertical = CheckMenuItem::with_id(
        app,
        "layout_vertical",
        "垂直",
        true,
        ui_state.layout == Layout::Vertical,
        None::<&str>,
    )?;

    let mut color_items = Vec::new();
    for option in COLOR_OPTIONS {
        let checked = option.value.eq_ignore_ascii_case(&ui_state.text_color);
        let item = CheckMenuItem::with_id(
            app,
            option.id,
            option.label,
            true,
            checked,
            None::<&str>,
        )?;
        color_items.push(ColorMenuItem {
            value: option.value,
            item,
        });
    }

    let monitor_cpu = CheckMenuItem::with_id(
        app,
        "monitor_cpu",
        "CPU",
        true,
        ui_state.show_cpu,
        None::<&str>,
    )?;
    let monitor_mem = CheckMenuItem::with_id(
        app,
        "monitor_mem",
        "Mem",
        true,
        ui_state.show_mem,
        None::<&str>,
    )?;
    let monitor_net = CheckMenuItem::with_id(
        app,
        "monitor_net",
        "Net",
        true,
        ui_state.show_net,
        None::<&str>,
    )?;

    let tray_items = TrayMenuItems {
        autostart: autostart_item.clone(),
        pos_top_left: pos_top_left.clone(),
        pos_bottom_left: pos_bottom_left.clone(),
        pos_top_right: pos_top_right.clone(),
        pos_bottom_right: pos_bottom_right.clone(),
        layout_horizontal: layout_horizontal.clone(),
        layout_vertical: layout_vertical.clone(),
        color_items: color_items.clone(),
        monitor_cpu: monitor_cpu.clone(),
        monitor_mem: monitor_mem.clone(),
        monitor_net: monitor_net.clone(),
    };

    let position_menu = SubmenuBuilder::new(app, "位置")
        .item(&pos_top_left)
        .item(&pos_bottom_left)
        .item(&pos_top_right)
        .item(&pos_bottom_right)
        .build()?;

    let layout_menu = SubmenuBuilder::new(app, "布局")
        .item(&layout_horizontal)
        .item(&layout_vertical)
        .build()?;

    let mut color_menu_builder = SubmenuBuilder::new(app, "颜色");
    for color_item in &color_items {
        color_menu_builder = color_menu_builder.item(&color_item.item);
    }
    let color_menu = color_menu_builder.build()?;

    let monitor_menu = SubmenuBuilder::new(app, "监控")
        .item(&monitor_cpu)
        .item(&monitor_mem)
        .item(&monitor_net)
        .build()?;

    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let tray_menu = MenuBuilder::new(app)
        .item(&position_menu)
        .item(&layout_menu)
        .item(&color_menu)
        .item(&monitor_menu)
        .separator()
        .item(&autostart_item)
        .separator()
        .item(&quit_item)
        .build()?;

    let mut tray_builder = TrayIconBuilder::new()
        .menu(&tray_menu)
        .show_menu_on_left_click(true)
        .on_menu_event({
            let tray_items = tray_items.clone();
            move |app, event| {
                let id = event.id().as_ref();
                match id {
                    "autostart" => {
                        let enabled = app.autolaunch().is_enabled().unwrap_or(false);
                        let result = if enabled {
                            app.autolaunch().disable()
                        } else {
                            app.autolaunch().enable()
                        };
                        if result.is_ok() {
                            tray_items.set_autostart(!enabled);
                        }
                    }
                    "pos_top_left" => {
                        update_position(app, WindowPosition::TopLeft, &tray_items);
                    }
                    "pos_bottom_left" => {
                        update_position(app, WindowPosition::BottomLeft, &tray_items);
                    }
                    "pos_top_right" => {
                        update_position(app, WindowPosition::TopRight, &tray_items);
                    }
                    "pos_bottom_right" => {
                        update_position(app, WindowPosition::BottomRight, &tray_items);
                    }
                    "layout_horizontal" => {
                        update_layout(app, Layout::Horizontal, &tray_items);
                    }
                    "layout_vertical" => {
                        update_layout(app, Layout::Vertical, &tray_items);
                    }
                    "color_white" => {
                        update_text_color(app, "#ffffff", &tray_items);
                    }
                    "color_black" => {
                        update_text_color(app, "#000000", &tray_items);
                    }
                    "color_cyan" => {
                        update_text_color(app, "#8fe9ff", &tray_items);
                    }
                    "color_green" => {
                        update_text_color(app, "#7cff6b", &tray_items);
                    }
                    "color_orange" => {
                        update_text_color(app, "#ffb454", &tray_items);
                    }
                    "color_pink" => {
                        update_text_color(app, "#ff6fae", &tray_items);
                    }
                    "color_yellow" => {
                        update_text_color(app, "#ffd56a", &tray_items);
                    }
                    "monitor_cpu" => {
                        update_monitor_visibility(app, MonitorItem::Cpu, &tray_items);
                    }
                    "monitor_mem" => {
                        update_monitor_visibility(app, MonitorItem::Mem, &tray_items);
                    }
                    "monitor_net" => {
                        update_monitor_visibility(app, MonitorItem::Net, &tray_items);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                }
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray_builder = tray_builder.icon(icon);
    }

    tray_builder.tooltip("corner-monitor").build(app)?;
    Ok(tray_items)
}
