//! 网络信息采集模块

use crate::monitor::types::{NetworkInfo, NetworkInterfaceInfo};
use sysinfo::Networks;
use std::collections::HashMap;
use std::time::Instant;

/// 网络接口上一次的数据快照
struct NetworkSnapshot {
    received: u64,
    transmitted: u64,
    timestamp: Instant,
}

/// 网络采集器
pub struct NetworkCollector {
    networks: Networks,
    /// 存储上一次各接口的数据，用于计算速率
    last_snapshot: HashMap<String, NetworkSnapshot>,
}

impl NetworkCollector {
    /// 创建新的网络采集器
    pub fn new() -> Self {
        let networks = Networks::new_with_refreshed_list();
        Self {
            networks,
            last_snapshot: HashMap::new(),
        }
    }

    /// 采集网络信息
    pub fn collect(&mut self) -> NetworkInfo {
        self.networks.refresh(true);
        
        let now = Instant::now();
        let mut interfaces: Vec<NetworkInterfaceInfo> = Vec::new();
        let mut total_upload_speed: u64 = 0;
        let mut total_download_speed: u64 = 0;
        let mut total_uploaded: u64 = 0;
        let mut total_downloaded: u64 = 0;

        for (name, network) in self.networks.iter() {
            let current_received = network.total_received();
            let current_transmitted = network.total_transmitted();

            // 计算速率
            let (download_speed, upload_speed) = if let Some(last) = self.last_snapshot.get(name) {
                let elapsed = now.duration_since(last.timestamp).as_secs_f64();
                if elapsed > 0.0 {
                    let download = ((current_received.saturating_sub(last.received)) as f64 / elapsed) as u64;
                    let upload = ((current_transmitted.saturating_sub(last.transmitted)) as f64 / elapsed) as u64;
                    (download, upload)
                } else {
                    (0, 0)
                }
            } else {
                (0, 0)
            };

            // 更新快照
            self.last_snapshot.insert(name.clone(), NetworkSnapshot {
                received: current_received,
                transmitted: current_transmitted,
                timestamp: now,
            });

            let interface_info = NetworkInterfaceInfo {
                name: name.clone(),
                upload_speed,
                download_speed,
                total_uploaded: current_transmitted,
                total_downloaded: current_received,
            };

            total_upload_speed += upload_speed;
            total_download_speed += download_speed;
            total_uploaded += current_transmitted;
            total_downloaded += current_received;

            interfaces.push(interface_info);
        }

        NetworkInfo {
            interfaces,
            total_upload_speed,
            total_download_speed,
            total_uploaded,
            total_downloaded,
        }
    }
}

impl Default for NetworkCollector {
    fn default() -> Self {
        Self::new()
    }
}
