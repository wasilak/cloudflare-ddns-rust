use once_cell::sync::Lazy;
use rand::prelude::IndexedRandom;
use rand::rng;
use std::sync::{Arc, RwLock};

#[derive(serde::Serialize, Clone)]
pub struct IP {
    pub ip: String,
    pub source: IPSource,
}

#[derive(Clone, Debug, serde::Serialize)]
pub enum IPSource {
    ApifyOrg(ApifyOrg),
    IpApi(IpApi),
    IpinfoIo(IpinfoIo),
    IdentMe(IdentMe),
}

impl IPSource {
    pub fn name(&self) -> &str {
        match self {
            IPSource::ApifyOrg(s) => &s.name,
            IPSource::IpApi(s) => &s.name,
            IPSource::IpinfoIo(s) => &s.name,
            IPSource::IdentMe(s) => &s.name,
        }
    }

    pub fn url(&self) -> &str {
        match self {
            IPSource::ApifyOrg(s) => &s.url,
            IPSource::IpApi(s) => &s.url,
            IPSource::IpinfoIo(s) => &s.url,
            IPSource::IdentMe(s) => &s.url,
        }
    }

    pub async fn get() -> Result<IP, Box<dyn std::error::Error>> {
        let sources = vec![
            IPSource::ApifyOrg(ApifyOrg::new()),
            IPSource::IpApi(IpApi::new()),
            IPSource::IpinfoIo(IpinfoIo::new()),
            IPSource::IdentMe(IdentMe::new()),
        ];

        let mut rng = rng();

        let mut selected = sources
            .choose(&mut rng)
            .expect("No sources configured")
            .clone();

        let res = reqwest::get(selected.url()).await?.text().await?;

        let ip = match &mut selected {
            IPSource::ApifyOrg(_) => {
                let parsed: IpFyResponse = serde_json::from_str(&res)?;
                parsed.ip
            }
            IPSource::IpApi(_) => {
                let parsed: IpApiResponse = serde_json::from_str(&res)?;
                parsed.query
            }
            IPSource::IpinfoIo(_) => {
                let parsed: IpinfoIoResponse = serde_json::from_str(&res)?;
                parsed.ip
            }
            IPSource::IdentMe(_) => {
                let parsed: IdentMeResponse = serde_json::from_str(&res)?;
                parsed.address
            }
        };

        Ok(IP {
            ip,
            source: selected,
        })
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct ApifyOrg {
    pub name: String,
    pub url: String,
}

impl ApifyOrg {
    pub fn new() -> Self {
        Self {
            name: "IpifyOrg".to_string(),
            url: "https://api.ipify.org?format=json".to_string(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct IpApi {
    pub name: String,
    pub url: String,
}

impl IpApi {
    pub fn new() -> Self {
        Self {
            name: "IpApi".to_string(),
            url: "http://ip-api.com/json/".to_string(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct IpinfoIo {
    pub name: String,
    pub url: String,
}

impl IpinfoIo {
    pub fn new() -> Self {
        Self {
            name: "IpinfoIo".to_string(),
            url: "https://ipinfo.io/json".to_string(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct IdentMe {
    pub name: String,
    pub url: String,
}

impl IdentMe {
    pub fn new() -> Self {
        Self {
            name: "IdentMe".to_string(),
            url: "https://ident.me/.json".to_string(),
        }
    }
}

#[derive(serde::Deserialize)]
struct IpFyResponse {
    ip: String,
}

#[derive(serde::Deserialize)]
struct IpApiResponse {
    query: String,
}

#[derive(serde::Deserialize)]
struct IpinfoIoResponse {
    ip: String,
}

#[derive(serde::Deserialize)]
struct IdentMeResponse {
    address: String,
}

pub static EXTERNAL_IP: Lazy<RwLock<Option<Arc<IP>>>> = Lazy::new(|| RwLock::new(None));

pub fn set_external_ip(ip: IP) {
    let mut lock = EXTERNAL_IP.write().unwrap();
    *lock = Some(Arc::new(ip));
}

pub fn get_external_ip() -> Option<IP> {
    EXTERNAL_IP
        .read()
        .unwrap()
        .as_ref()
        .map(|ip| (**ip).clone())
}
