use crate::libs::api::upsert_record;
use crate::libs::config::CONFIG;
use crate::libs::ip::{IPSource, get_external_ip, set_external_ip};
use tokio::time::{Duration, interval};

pub async fn refresh_dns_loop(refresh_interval_secs: u64) {
    let mut interval_timer = interval(Duration::from_secs(refresh_interval_secs));

    loop {
        interval_timer.tick().await;

        match IPSource::get().await {
            Ok(current_ip) => {
                tracing::info!(
                    "Current IP: {} from {}",
                    current_ip.ip,
                    current_ip.source.name()
                );

                let ip_has_changed = {
                    match get_external_ip() {
                        Some(last_known_ip) if last_known_ip.ip == current_ip.ip => {
                            tracing::info!("IP hasn't changed.");
                            false
                        }
                        _ => {
                            tracing::info!(
                                "IP changed, updating stored IP to: {} from {}",
                                current_ip.ip,
                                current_ip.source.name()
                            );
                            set_external_ip(current_ip.clone());
                            true
                        }
                    }
                };

                if ip_has_changed {
                    let config_snapshot = {
                        let config = CONFIG.read().unwrap();
                        config.records.clone()
                    };

                    for (zone_name, records) in config_snapshot {
                        for mut record in records {
                            record.content = Some(current_ip.ip.clone());

                            if let Err(e) = upsert_record(&zone_name, record).await {
                                tracing::error!("Error updating record: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => tracing::error!("Failed to get IP: {}", e),
        }
    }
}
