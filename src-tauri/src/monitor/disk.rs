//! 磁盘信息采集模块

use crate::monitor::types::{DiskDetail, DiskInfo};
use sysinfo::Disks;

/// 磁盘采集器
pub struct DiskCollector {
    disks: Disks,
}

impl DiskCollector {
    /// 创建新的磁盘采集器
    pub fn new() -> Self {
        let disks = Disks::new_with_refreshed_list();
        Self { disks }
    }

    /// 采集磁盘信息
    pub fn collect(&mut self) -> DiskInfo {
        self.disks.refresh(true);

        let mut disk_details: Vec<DiskDetail> = Vec::new();
        let mut total: u64 = 0;
        let mut total_used: u64 = 0;
        let mut total_available: u64 = 0;

        for disk in self.disks.iter() {
            let disk_total = disk.total_space();
            let disk_available = disk.available_space();
            let disk_used = disk_total.saturating_sub(disk_available);

            let usage_percent = if disk_total > 0 {
                (disk_used as f32 / disk_total as f32) * 100.0
            } else {
                0.0
            };

            let file_system = disk.file_system()
                .to_string_lossy()
                .to_string();

            let detail = DiskDetail {
                name: disk.name().to_string_lossy().to_string(),
                mount_point: disk.mount_point().to_string_lossy().to_string(),
                file_system,
                total: disk_total,
                used: disk_used,
                available: disk_available,
                usage_percent,
                is_removable: disk.is_removable(),
            };

            // 累加总量（只计算非可移除磁盘或有意义的磁盘）
            total += disk_total;
            total_used += disk_used;
            total_available += disk_available;

            disk_details.push(detail);
        }

        let total_usage_percent = if total > 0 {
            (total_used as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        DiskInfo {
            disks: disk_details,
            total,
            total_used,
            total_available,
            total_usage_percent,
        }
    }
}

impl Default for DiskCollector {
    fn default() -> Self {
        Self::new()
    }
}
