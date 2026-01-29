//! 内存信息采集模块

use crate::monitor::types::MemoryInfo;
use sysinfo::{MemoryRefreshKind, RefreshKind, System};

/// 内存采集器
pub struct MemoryCollector {
    system: System,
}

impl MemoryCollector {
    /// 创建新的内存采集器
    pub fn new() -> Self {
        // 只刷新内存相关信息，减少不必要的开销
        let system = System::new_with_specifics(
            RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
        );
        Self { system }
    }

    /// 采集内存信息
    pub fn collect(&mut self) -> MemoryInfo {
        self.system.refresh_memory();

        let total = self.system.total_memory();
        let used = self.system.used_memory();
        let available = self.system.available_memory();
        
        let usage_percent = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        let swap_total = self.system.total_swap();
        let swap_used = self.system.used_swap();
        
        let swap_usage_percent = if swap_total > 0 {
            (swap_used as f32 / swap_total as f32) * 100.0
        } else {
            0.0
        };

        MemoryInfo {
            total,
            used,
            available,
            usage_percent,
            swap_total,
            swap_used,
            swap_usage_percent,
        }
    }
}

impl Default for MemoryCollector {
    fn default() -> Self {
        Self::new()
    }
}
