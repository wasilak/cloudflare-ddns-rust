use once_cell::sync::Lazy;
use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub records: HashMap<String, Vec<crate::libs::api::DnsRecord>>,
}

pub static CONFIG: Lazy<RwLock<Config>> = Lazy::new(|| {
    RwLock::new(Config {
        records: HashMap::new(),
    })
});

impl Config {
    pub fn new_empty() {
        *CONFIG.write().unwrap() = Config {
            records: HashMap::new(),
        };
    }

    pub fn load_from_yaml(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let parsed_config: Config = serde_yaml::from_str(&contents)?;
        println!("Loaded config: {:?}", parsed_config);
        *CONFIG.write().unwrap() = parsed_config;
        Ok(())
    }

    pub fn get_zone_records(&self, zone: &String) -> Option<&Vec<crate::libs::api::DnsRecord>> {
        self.records.get(zone)
    }

    pub fn get_zone_record(
        &self,
        zone: &String,
        record: &String,
    ) -> Option<&crate::libs::api::DnsRecord> {
        match self.get_zone_records(zone) {
            Some(records) => records.iter().find(|r| r.name == Some(record.clone())),
            None => None,
        }
    }

    pub fn delete_zone_record(
        &mut self,
        zone: &String,
        record: &String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(records) = self.records.get_mut(zone) {
            records.retain(|r| r.name != Some(record.clone()));
            if records.is_empty() {
                self.records.remove(zone);
            }
        }
        Ok(())
    }

    pub fn upsert_zone_record(
        &mut self,
        zone: &str,
        record: crate::libs::api::DnsRecord,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure the record has a name, or skip
        let record_name = match &record.name {
            Some(name) => name,
            None => {
                tracing::warn!("Skipping record without name for zone {}", zone);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Record name is missing",
                )));
            }
        };

        // Get or insert zone entry
        let records = self
            .records
            .entry(zone.to_string())
            .or_insert_with(Vec::new);

        // Check if the record exists
        if let Some(existing) = records
            .iter_mut()
            .find(|r| r.name.as_ref() == Some(record_name))
        {
            *existing = record;
        } else {
            records.push(record);
        }

        return Ok(());
    }
}
