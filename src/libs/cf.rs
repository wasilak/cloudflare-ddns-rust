use std::vec;

use cloudflare::endpoints::dns::dns::{self};
use cloudflare::endpoints::zones::zone;
use cloudflare::framework::client::async_api::Client as AsyncClient;
use cloudflare::framework::{OrderDirection, response::ApiFailure};

fn map_cloudflare_error(e: ApiFailure) -> Box<dyn std::error::Error> {
    match e {
        ApiFailure::Error(status, errors) => {
            println!("HTTP {status}:");
            for err in errors.errors {
                println!("Error {}: {}", err.code, err.message);
                for (k, v) in err.other {
                    println!("{k}: {v}");
                }
            }
            for (k, v) in errors.other {
                println!("{k}: {v}");
            }
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "API request failed",
            ))
        }
        ApiFailure::Invalid(reqwest_err) => {
            println!("Error: {reqwest_err}");
            Box::new(reqwest_err)
        }
    }
}

pub async fn get_zone(
    api_client: &AsyncClient,
    zone_name: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let zone_list_params = zone::ListZones {
        params: zone::ListZonesParams {
            name: Some(zone_name),
            ..Default::default()
        },
    };

    let response = api_client.request(&zone_list_params);
    match response.await {
        Ok(success) => {
            return {
                if success.result.is_empty() {
                    Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Zone not found",
                    )))
                } else {
                    Ok(success.result.first().unwrap().id.clone())
                }
            };
        }
        Err(e) => Err(map_cloudflare_error(e)),
    }
}

pub async fn list_records(
    api_client: &AsyncClient,
    zone_id: String,
) -> Result<Vec<crate::libs::api::DnsRecord>, Box<dyn std::error::Error>> {
    let endpoint = dns::ListDnsRecords {
        zone_identifier: &zone_id,
        params: dns::ListDnsRecordsParams {
            direction: Some(OrderDirection::Ascending),
            ..Default::default()
        },
    };

    match api_client.request(&endpoint).await {
        Ok(success) => {
            let mut records: Vec<crate::libs::api::DnsRecord> = vec![];
            success.result.iter().for_each(|record| {
                match record.content {
                    cloudflare::endpoints::dns::dns::DnsContent::A { content } => {
                        records.push(crate::libs::api::DnsRecord {
                            id: Some(record.id.clone()),
                            name: Some(record.name.clone()),
                            content: Some(content.to_string()),
                            ttl: Some(record.ttl),
                            proxied: Some(record.proxied),
                        });
                    }
                    _ => (),
                };
            });
            return Ok(records);
        }
        Err(e) => Err(map_cloudflare_error(e)),
    }
}

pub async fn create_record(
    api_client: &AsyncClient,
    zone_id: String,
    record: crate::libs::api::DnsRecord,
) -> Result<crate::libs::api::DnsRecord, Box<dyn std::error::Error>> {
    let endpoint = dns::CreateDnsRecord {
        zone_identifier: &zone_id,
        params: dns::CreateDnsRecordParams {
            name: &record.name.unwrap(),
            content: dns::DnsContent::A {
                content: record.content.unwrap().parse().unwrap(),
            },
            ttl: record.ttl,
            proxied: record.proxied,
            priority: None,
        },
    };

    match api_client.request(&endpoint).await {
        Ok(success) => {
            return {
                let content_str = match success.result.content {
                    dns::DnsContent::A { content } => content.to_string(),
                    _ => "".to_string(),
                };
                Ok(crate::libs::api::DnsRecord {
                    id: Some(success.result.id),
                    name: Some(success.result.name),
                    content: Some(content_str),
                    ttl: Some(success.result.ttl),
                    proxied: Some(success.result.proxied),
                })
            };
        }
        Err(e) => Err(map_cloudflare_error(e)),
    }
}

pub async fn update_record(
    api_client: &AsyncClient,
    zone_id: String,
    record: crate::libs::api::DnsRecord,
) -> Result<crate::libs::api::DnsRecord, Box<dyn std::error::Error>> {
    let endpoint = dns::UpdateDnsRecord {
        zone_identifier: &zone_id,
        identifier: &record.id.unwrap(),
        params: dns::UpdateDnsRecordParams {
            name: &record.name.unwrap(),
            content: dns::DnsContent::A {
                content: record.content.unwrap().parse().unwrap(),
            },
            ttl: record.ttl,
            proxied: record.proxied,
        },
    };

    match api_client.request(&endpoint).await {
        Ok(success) => {
            return {
                let content_str = match success.result.content {
                    dns::DnsContent::A { content } => content.to_string(),
                    _ => "".to_string(),
                };
                Ok(crate::libs::api::DnsRecord {
                    id: Some(success.result.id),
                    name: Some(success.result.name),
                    content: Some(content_str),
                    ttl: Some(success.result.ttl),
                    proxied: Some(success.result.proxied),
                })
            };
        }
        Err(e) => Err(map_cloudflare_error(e)),
    }
}

pub async fn delete_record(
    api_client: &AsyncClient,
    zone_id: String,
    record: crate::libs::api::DnsRecord,
) -> Result<crate::libs::api::DnsRecord, Box<dyn std::error::Error>> {
    let endpoint = dns::DeleteDnsRecord {
        zone_identifier: &zone_id,
        identifier: &record.clone().id.unwrap(),
    };

    match api_client.request(&endpoint).await {
        Ok(_) => Ok(record),
        Err(e) => Err(map_cloudflare_error(e)),
    }
}
