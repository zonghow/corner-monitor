//! 磁盘信息采集模块

use crate::monitor::types::{DiskDetail, DiskInfo};
use sysinfo::Disks;

/// 磁盘采集器
pub struct DiskCollector {
    disks: Disks,
}

fn mount_point_priority(mount_point: &str) -> u8 {
    match mount_point {
        "/System/Volumes/Data" => 2,
        "/" => 1,
        _ => 0,
    }
}

fn select_primary_disk(disks: &[DiskDetail]) -> Option<&DiskDetail> {
    let has_non_removable = disks.iter().any(|disk| !disk.is_removable);

    disks.iter()
        .filter(|disk| !has_non_removable || !disk.is_removable)
        .max_by_key(|disk| (disk.total, mount_point_priority(&disk.mount_point)))
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

            disk_details.push(detail);
        }

        let (total, total_used, total_available, total_usage_percent) = select_primary_disk(&disk_details)
            .map(|disk| {
                (
                    disk.total,
                    disk.used,
                    disk.available,
                    disk.usage_percent,
                )
            })
            .unwrap_or((0, 0, 0, 0.0));

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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_disk_detail(
        name: &str,
        mount_point: &str,
        total: u64,
        available: u64,
        is_removable: bool,
    ) -> DiskDetail {
        let used = total.saturating_sub(available);
        let usage_percent = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };

        DiskDetail {
            name: name.to_string(),
            mount_point: mount_point.to_string(),
            file_system: "apfs".to_string(),
            total,
            used,
            available,
            usage_percent,
            is_removable,
        }
    }

    #[test]
    fn selects_data_volume_instead_of_summing_apfs_volumes() {
        let disks = vec![
            make_disk_detail("Macintosh HD", "/", 494_000_000_000, 120_000_000_000, false),
            make_disk_detail(
                "Macintosh HD - Data",
                "/System/Volumes/Data",
                494_000_000_000,
                120_000_000_000,
                false,
            ),
            make_disk_detail(
                "Preboot",
                "/System/Volumes/Preboot",
                20_000_000_000,
                15_000_000_000,
                false,
            ),
        ];

        let selected = select_primary_disk(&disks).expect("expected primary disk");

        assert_eq!(selected.mount_point, "/System/Volumes/Data");
        assert_eq!(selected.total, 494_000_000_000);
    }
}
