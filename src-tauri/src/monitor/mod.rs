//! 系统监控模块
//!
//! 提供 CPU、内存、磁盘、网络的监控功能，支持多线程后台采集。
//!
//! # 使用示例
//!
//! ```rust
//! use std::time::Duration;
//! use tatsu_monitor_lib::monitor::{Monitor, MonitorConfig};
//!
//! // 创建配置
//! let config = MonitorConfig::new()
//!     .cpu_interval(Duration::from_secs(1))
//!     .memory_interval(Duration::from_secs(2))
//!     .disk_interval(Duration::from_secs(5))
//!     .network_interval(Duration::from_secs(1));
//!
//! // 创建并启动监控器
//! let monitor = Monitor::new(config);
//! monitor.start();
//!
//! // 获取系统信息
//! let system_info = monitor.get_system_info();
//! println!("CPU Usage: {:.2}%", system_info.cpu.total_usage);
//!
//! // 获取单独的信息
//! let cpu_info = monitor.get_cpu_info();
//! let memory_info = monitor.get_memory_info();
//! let disk_info = monitor.get_disk_info();
//! let network_info = monitor.get_network_info();
//!
//! // 停止监控
//! monitor.stop();
//! ```

mod types;
mod cpu;
mod memory;
mod disk;
mod network;

pub use types::*;

use cpu::CpuCollector;
use memory::MemoryCollector;
use disk::DiskCollector;
use network::NetworkCollector;

use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

/// 内部共享状态
struct MonitorState {
    cpu: RwLock<CpuInfo>,
    memory: RwLock<MemoryInfo>,
    disk: RwLock<DiskInfo>,
    network: RwLock<NetworkInfo>,
    running: AtomicBool,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            cpu: RwLock::new(CpuInfo::default()),
            memory: RwLock::new(MemoryInfo::default()),
            disk: RwLock::new(DiskInfo::default()),
            network: RwLock::new(NetworkInfo::default()),
            running: AtomicBool::new(false),
        }
    }
}

/// 系统监控器
///
/// 使用多线程后台采集，各类数据按独立的采集频率更新。
/// 调用 `get_*` 方法可随时获取最新的监控数据。
pub struct Monitor {
    config: MonitorConfig,
    state: Arc<MonitorState>,
    handles: RwLock<Vec<thread::JoinHandle<()>>>,
}

impl Monitor {
    /// 使用指定配置创建监控器
    pub fn new(config: MonitorConfig) -> Self {
        Self {
            config,
            state: Arc::new(MonitorState::default()),
            handles: RwLock::new(Vec::new()),
        }
    }

    /// 使用默认配置创建监控器
    pub fn with_default_config() -> Self {
        Self::new(MonitorConfig::default())
    }

    /// 启动后台采集线程
    pub fn start(&self) {
        if self.state.running.swap(true, Ordering::SeqCst) {
            // 已经在运行
            return;
        }

        let mut handles = self.handles.write();

        // 使用单线程轮询所有采集器，减少线程数量
        let state = Arc::clone(&self.state);
        let cpu_interval = self.config.cpu_interval;
        let memory_interval = self.config.memory_interval;
        let disk_interval = self.config.disk_interval;
        let network_interval = self.config.network_interval;

        let handle = thread::spawn(move || {
            let mut cpu_collector = CpuCollector::new();
            let mut memory_collector = MemoryCollector::new();
            let mut disk_collector = DiskCollector::new();
            let mut network_collector = NetworkCollector::new();

            // 初始采集一次
            thread::sleep(std::time::Duration::from_millis(100));

            // 使用计时器追踪每个采集器的下次执行时间
            let tick_interval = std::time::Duration::from_millis(100); // 基础轮询间隔
            let mut cpu_countdown = std::time::Duration::ZERO;
            let mut memory_countdown = std::time::Duration::ZERO;
            let mut disk_countdown = std::time::Duration::ZERO;
            let mut network_countdown = std::time::Duration::ZERO;

            while state.running.load(Ordering::SeqCst) {
                // CPU 采集
                if cpu_countdown <= std::time::Duration::ZERO {
                    let info = cpu_collector.collect();
                    *state.cpu.write() = info;
                    cpu_countdown = cpu_interval;
                }

                // 内存采集
                if memory_countdown <= std::time::Duration::ZERO {
                    let info = memory_collector.collect();
                    *state.memory.write() = info;
                    memory_countdown = memory_interval;
                }

                // 磁盘采集
                if disk_countdown <= std::time::Duration::ZERO {
                    let info = disk_collector.collect();
                    *state.disk.write() = info;
                    disk_countdown = disk_interval;
                }

                // 网络采集
                if network_countdown <= std::time::Duration::ZERO {
                    let info = network_collector.collect();
                    *state.network.write() = info;
                    network_countdown = network_interval;
                }

                // 等待并更新倒计时
                thread::sleep(tick_interval);
                cpu_countdown = cpu_countdown.saturating_sub(tick_interval);
                memory_countdown = memory_countdown.saturating_sub(tick_interval);
                disk_countdown = disk_countdown.saturating_sub(tick_interval);
                network_countdown = network_countdown.saturating_sub(tick_interval);
            }
        });
        handles.push(handle);
    }

    /// 停止后台采集线程
    pub fn stop(&self) {
        self.state.running.store(false, Ordering::SeqCst);
        
        // 等待所有线程结束
        let mut handles = self.handles.write();
        for handle in handles.drain(..) {
            let _ = handle.join();
        }
    }

    /// 检查监控器是否正在运行
    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.state.running.load(Ordering::SeqCst)
    }

    /// 获取 CPU 信息
    pub fn get_cpu_info(&self) -> CpuInfo {
        self.state.cpu.read().clone()
    }

    /// 获取内存信息
    pub fn get_memory_info(&self) -> MemoryInfo {
        self.state.memory.read().clone()
    }

    /// 获取磁盘信息
    pub fn get_disk_info(&self) -> DiskInfo {
        self.state.disk.read().clone()
    }

    /// 获取网络信息
    pub fn get_network_info(&self) -> NetworkInfo {
        self.state.network.read().clone()
    }

    /// 获取完整的系统信息
    pub fn get_system_info(&self) -> SystemInfo {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        SystemInfo {
            cpu: self.get_cpu_info(),
            memory: self.get_memory_info(),
            disk: self.get_disk_info(),
            network: self.get_network_info(),
            timestamp,
        }
    }

    /// 立即刷新所有数据（同步操作，会阻塞当前线程）
    pub fn refresh_all(&self) {
        // CPU
        {
            let mut collector = CpuCollector::new();
            thread::sleep(std::time::Duration::from_millis(100));
            let info = collector.collect();
            *self.state.cpu.write() = info;
        }

        // Memory
        {
            let mut collector = MemoryCollector::new();
            let info = collector.collect();
            *self.state.memory.write() = info;
        }

        // Disk
        {
            let mut collector = DiskCollector::new();
            let info = collector.collect();
            *self.state.disk.write() = info;
        }

        // Network
        {
            let mut collector = NetworkCollector::new();
            let info = collector.collect();
            *self.state.network.write() = info;
        }
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Default for Monitor {
    fn default() -> Self {
        Self::with_default_config()
    }
}

/// 便捷函数：一次性获取系统信息（不启动后台线程）
#[allow(dead_code)]
pub fn get_system_info_once() -> SystemInfo {
    let monitor = Monitor::with_default_config();
    monitor.refresh_all();
    monitor.get_system_info()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_monitor_basic() {
        let config = MonitorConfig::new()
            .cpu_interval(Duration::from_millis(500))
            .memory_interval(Duration::from_millis(500))
            .disk_interval(Duration::from_secs(1))
            .network_interval(Duration::from_millis(500));

        let monitor = Monitor::new(config);
        monitor.start();
        
        // 等待数据采集
        thread::sleep(Duration::from_secs(1));
        
        let info = monitor.get_system_info();
        
        // 基本验证
        assert!(info.memory.total > 0);
        assert!(!info.disk.disks.is_empty());
        
        monitor.stop();
        assert!(!monitor.is_running());
    }

    #[test]
    fn test_get_system_info_once() {
        let info = get_system_info_once();
        assert!(info.memory.total > 0);
    }
}
