use std::vec;

use cloudflare::endpoints::dns::dns::{self};
use cloudflare::endpoints::zones::zone;
use cloudflare::framework::client::async_api::Client as AsyncClient;
use cloudflare::framework::{OrderDirection, response::ApiFailure};

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
        Err(e) => {
            println!("Error: {e}");
            return Err(Box::new(e));
        }
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
                            name: Some(record.name.clone()),
                            content: Some(content.to_string()),
                        });
                    }
                    _ => (),
                };
            });
            return Ok(records);
        }
        Err(e) => match e {
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
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "API request failed",
                )));
            }
            ApiFailure::Invalid(reqwest_err) => {
                println!("Error: {reqwest_err}");
                return Err(Box::new(reqwest_err));
            }
        },
    }
}
