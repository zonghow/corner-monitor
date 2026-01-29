use serde::Serialize;
use std::sync::Arc;
use tauri::{LogicalSize, Wry};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowPosition {
    TopLeft,
    BottomLeft,
    TopRight,
    BottomRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layout {
    Horizontal,
    Vertical,
}

pub enum MonitorItem {
    Cpu,
    Mem,
    Net,
}

#[derive(Clone, Debug)]
pub struct UiState {
    pub position: WindowPosition,
    pub layout: Layout,
    pub monitor_target: Option<MonitorTarget>,
    pub text_color: String,
    pub show_cpu: bool,
    pub show_mem: bool,
    pub show_net: bool,
}

pub const SETTINGS_PATH: &str = "ui-settings.json";
pub const KEY_POSITION: &str = "position";
pub const KEY_LAYOUT: &str = "layout";
pub const KEY_MONITOR_TARGET: &str = "monitor_target";
pub const KEY_TEXT_COLOR: &str = "text_color";
pub const KEY_MONITOR_CPU: &str = "monitor_cpu";
pub const KEY_MONITOR_MEM: &str = "monitor_mem";
pub const KEY_MONITOR_NET: &str = "monitor_net";
pub const SIZE_HORIZONTAL: LogicalSize<f64> = LogicalSize::new(190.0, 40.0);
pub const SIZE_VERTICAL: LogicalSize<f64> = LogicalSize::new(75.0, 100.0);
pub type SettingsStore = Arc<tauri_plugin_store::Store<Wry>>;

impl Default for UiState {
    fn default() -> Self {
        Self {
            position: WindowPosition::TopLeft,
            layout: Layout::Vertical,
            monitor_target: None,
            text_color: "#ffffff".to_string(),
            show_cpu: true,
            show_mem: true,
            show_net: true,
        }
    }
}

pub fn layout_to_str(layout: Layout) -> &'static str {
    match layout {
        Layout::Horizontal => "horizontal",
        Layout::Vertical => "vertical",
    }
}

pub fn layout_from_str(value: &str) -> Option<Layout> {
    match value {
        "horizontal" => Some(Layout::Horizontal),
        "vertical" => Some(Layout::Vertical),
        _ => None,
    }
}

pub fn position_to_str(position: WindowPosition) -> &'static str {
    match position {
        WindowPosition::TopLeft => "top-left",
        WindowPosition::BottomLeft => "bottom-left",
        WindowPosition::TopRight => "top-right",
        WindowPosition::BottomRight => "bottom-right",
    }
}

pub fn position_from_str(value: &str) -> Option<WindowPosition> {
    match value {
        "top-left" => Some(WindowPosition::TopLeft),
        "bottom-left" => Some(WindowPosition::BottomLeft),
        "top-right" => Some(WindowPosition::TopRight),
        "bottom-right" => Some(WindowPosition::BottomRight),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct MonitorVisibility {
    pub cpu: bool,
    pub mem: bool,
    pub net: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonitorTarget {
    pub index: usize,
    pub name: Option<String>,
}

#[derive(Clone, Copy)]
pub struct ColorOption {
    pub id: &'static str,
    pub label: &'static str,
    pub value: &'static str,
}

pub const COLOR_OPTIONS: [ColorOption; 7] = [
    ColorOption {
        id: "color_white",
        label: "白色",
        value: "#ffffff",
    },
    ColorOption {
        id: "color_black",
        label: "黑色",
        value: "#000000",
    },
    ColorOption {
        id: "color_cyan",
        label: "青色",
        value: "#8fe9ff",
    },
    ColorOption {
        id: "color_green",
        label: "绿色",
        value: "#7cff6b",
    },
    ColorOption {
        id: "color_orange",
        label: "橙色",
        value: "#ffb454",
    },
    ColorOption {
        id: "color_pink",
        label: "粉色",
        value: "#ff6fae",
    },
    ColorOption {
        id: "color_yellow",
        label: "黄色",
        value: "#ffd56a",
    },
];

pub fn monitor_target_for_monitor(index: usize, monitor: &tauri::Monitor) -> MonitorTarget {
    MonitorTarget {
        index,
        name: monitor.name().cloned(),
    }
}

pub fn monitor_target_to_str(target: &MonitorTarget) -> String {
    match &target.name {
        Some(name) => format!("name:{}|index:{}", name, target.index),
        None => format!("index:{}", target.index),
    }
}

pub fn monitor_target_from_str(value: &str) -> Option<MonitorTarget> {
    let mut name = None;
    let mut index = None;
    for part in value.split('|') {
        if let Some(rest) = part.strip_prefix("name:") {
            name = Some(rest.to_string());
            continue;
        }
        if let Some(rest) = part.strip_prefix("index:") {
            index = rest.parse::<usize>().ok();
        }
    }
    index.map(|index| MonitorTarget { index, name })
}

fn same_monitor(a: &tauri::Monitor, b: &tauri::Monitor) -> bool {
    if let (Some(a_name), Some(b_name)) = (a.name(), b.name()) {
        if a_name == b_name {
            return true;
        }
    }
    *a.position() == *b.position() && *a.size() == *b.size()
}

pub fn monitor_target_from_monitor(
    app: &tauri::AppHandle,
    monitor: &tauri::Monitor,
) -> Option<MonitorTarget> {
    let monitors = app.available_monitors().ok()?;
    let index = monitors
        .iter()
        .enumerate()
        .find(|(_, candidate)| same_monitor(candidate, monitor))
        .map(|(index, _)| index)?;
    monitors
        .get(index)
        .map(|candidate| monitor_target_for_monitor(index, candidate))
}

pub fn primary_monitor_target(app: &tauri::AppHandle) -> Option<MonitorTarget> {
    let primary = app.primary_monitor().ok().flatten()?;
    monitor_target_from_monitor(app, &primary)
}

pub fn visibility_from_state(state: &UiState) -> MonitorVisibility {
    MonitorVisibility {
        cpu: state.show_cpu,
        mem: state.show_mem,
        net: state.show_net,
    }
}
