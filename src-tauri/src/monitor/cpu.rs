//! CPU 信息采集模块

use crate::monitor::types::{CpuCoreInfo, CpuInfo};
use sysinfo::{Components, CpuRefreshKind, RefreshKind, System};

/// CPU 采集器
pub struct CpuCollector {
    system: System,
    components: Components,
}

impl CpuCollector {
    /// 创建新的 CPU 采集器
    pub fn new() -> Self {
        // 只刷新 CPU 相关信息，减少不必要的开销
        let mut system = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
        );
        system.refresh_cpu_all();
        let components = Components::new_with_refreshed_list();
        
        Self { system, components }
    }

    /// 采集 CPU 信息
    pub fn collect(&mut self) -> CpuInfo {
        // 刷新 CPU 数据
        self.system.refresh_cpu_all();
        
        let cpus = self.system.cpus();
        
        // 获取品牌名称
        let brand = cpus.first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_default();

        // 计算总体使用率
        let total_usage = if cpus.is_empty() {
            0.0
        } else {
            cpus.iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / cpus.len() as f32
        };

        // 收集各核心信息
        let cores: Vec<CpuCoreInfo> = cpus.iter()
            .map(|cpu| CpuCoreInfo {
                name: cpu.name().to_string(),
                usage: cpu.cpu_usage(),
                frequency: cpu.frequency(),
            })
            .collect();

        // 获取 CPU 温度
        let temperature = self.get_cpu_temperature();

        // 获取物理核心数
        let physical_core_count = System::physical_core_count();

        CpuInfo {
            brand,
            total_usage,
            cores,
            temperature,
            physical_core_count,
        }
    }

    /// 获取 CPU 温度
    fn get_cpu_temperature(&mut self) -> Option<f32> {
        self.components.refresh(true);
        
        // 尝试从组件中找到 CPU 温度
        for component in self.components.iter() {
            let label = component.label().to_lowercase();
            // 不同系统的 CPU 温度标签可能不同
            if label.contains("cpu") || label.contains("core") || label.contains("package") {
                return component.temperature();
            }
        }

        // 如果没找到明确的 CPU 温度，尝试获取第一个温度传感器
        self.components.iter().next().and_then(|c| c.temperature())
    }
}

impl Default for CpuCollector {
    fn default() -> Self {
        Self::new()
    }
}
