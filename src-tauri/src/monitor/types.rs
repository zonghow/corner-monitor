//! 系统监控数据类型定义

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// CPU 核心信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCoreInfo {
    /// 核心名称
    pub name: String,
    /// 使用率 (0.0 - 100.0)
    pub usage: f32,
    /// 频率 (MHz)
    pub frequency: u64,
}

/// CPU 整体信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// 品牌名称
    pub brand: String,
    /// 总体使用率 (0.0 - 100.0)
    pub total_usage: f32,
    /// 各核心信息
    pub cores: Vec<CpuCoreInfo>,
    /// CPU 温度 (摄氏度)，可能在某些系统上不可用
    pub temperature: Option<f32>,
    /// 物理核心数
    pub physical_core_count: Option<usize>,
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self {
            brand: String::new(),
            total_usage: 0.0,
            cores: Vec::new(),
            temperature: None,
            physical_core_count: None,
        }
    }
}

/// 内存信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// 总内存 (字节)
    pub total: u64,
    /// 已使用内存 (字节)
    pub used: u64,
    /// 可用内存 (字节)
    pub available: u64,
    /// 使用率 (0.0 - 100.0)
    pub usage_percent: f32,
    /// 交换分区总量 (字节)
    pub swap_total: u64,
    /// 交换分区已使用 (字节)
    pub swap_used: u64,
    /// 交换分区使用率 (0.0 - 100.0)
    pub swap_usage_percent: f32,
}

impl Default for MemoryInfo {
    fn default() -> Self {
        Self {
            total: 0,
            used: 0,
            available: 0,
            usage_percent: 0.0,
            swap_total: 0,
            swap_used: 0,
            swap_usage_percent: 0.0,
        }
    }
}

/// 单个磁盘信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskDetail {
    /// 磁盘名称
    pub name: String,
    /// 挂载点
    pub mount_point: String,
    /// 文件系统类型
    pub file_system: String,
    /// 总容量 (字节)
    pub total: u64,
    /// 已使用 (字节)
    pub used: u64,
    /// 可用 (字节)
    pub available: u64,
    /// 使用率 (0.0 - 100.0)
    pub usage_percent: f32,
    /// 是否可移除
    pub is_removable: bool,
}

/// 磁盘整体信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    /// 各磁盘详情
    pub disks: Vec<DiskDetail>,
    /// 总容量 (字节)
    pub total: u64,
    /// 总已使用 (字节)
    pub total_used: u64,
    /// 总可用 (字节)
    pub total_available: u64,
    /// 总体使用率 (0.0 - 100.0)
    pub total_usage_percent: f32,
}

impl Default for DiskInfo {
    fn default() -> Self {
        Self {
            disks: Vec::new(),
            total: 0,
            total_used: 0,
            total_available: 0,
            total_usage_percent: 0.0,
        }
    }
}

/// 网络接口信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterfaceInfo {
    /// 接口名称
    pub name: String,
    /// 上传速率 (字节/秒)
    pub upload_speed: u64,
    /// 下载速率 (字节/秒)
    pub download_speed: u64,
    /// 累计上传字节数
    pub total_uploaded: u64,
    /// 累计下载字节数
    pub total_downloaded: u64,
}

/// 网络整体信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// 各网络接口信息
    pub interfaces: Vec<NetworkInterfaceInfo>,
    /// 总上传速率 (字节/秒)
    pub total_upload_speed: u64,
    /// 总下载速率 (字节/秒)
    pub total_download_speed: u64,
    /// 总累计上传字节数
    pub total_uploaded: u64,
    /// 总累计下载字节数
    pub total_downloaded: u64,
}

impl Default for NetworkInfo {
    fn default() -> Self {
        Self {
            interfaces: Vec::new(),
            total_upload_speed: 0,
            total_download_speed: 0,
            total_uploaded: 0,
            total_downloaded: 0,
        }
    }
}

/// 系统完整信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// CPU 信息
    pub cpu: CpuInfo,
    /// 内存信息
    pub memory: MemoryInfo,
    /// 磁盘信息
    pub disk: DiskInfo,
    /// 网络信息
    pub network: NetworkInfo,
    /// 采集时间戳 (毫秒)
    pub timestamp: u64,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self {
            cpu: CpuInfo::default(),
            memory: MemoryInfo::default(),
            disk: DiskInfo::default(),
            network: NetworkInfo::default(),
            timestamp: 0,
        }
    }
}

/// 监控配置
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// CPU 采集间隔
    pub cpu_interval: Duration,
    /// 内存采集间隔
    pub memory_interval: Duration,
    /// 磁盘采集间隔
    pub disk_interval: Duration,
    /// 网络采集间隔
    pub network_interval: Duration,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            cpu_interval: Duration::from_secs(5),
            memory_interval: Duration::from_secs(10),
            disk_interval: Duration::from_secs(60 * 5),
            network_interval: Duration::from_secs(3),
        }
    }
}

impl MonitorConfig {
    /// 创建新配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 CPU 采集间隔
    pub fn cpu_interval(mut self, interval: Duration) -> Self {
        self.cpu_interval = interval;
        self
    }

    /// 设置内存采集间隔
    pub fn memory_interval(mut self, interval: Duration) -> Self {
        self.memory_interval = interval;
        self
    }

    /// 设置磁盘采集间隔
    pub fn disk_interval(mut self, interval: Duration) -> Self {
        self.disk_interval = interval;
        self
    }

    /// 设置网络采集间隔
    pub fn network_interval(mut self, interval: Duration) -> Self {
        self.network_interval = interval;
        self
    }
}
