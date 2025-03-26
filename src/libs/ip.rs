use rand::Rng;

pub trait IPSource {
    fn name(&self) -> &'static str;
    fn url(&self) -> &str;
    fn parse_response(&self, body: &str) -> Result<String, Box<dyn std::error::Error>>;
}

#[derive(Debug, serde::Deserialize)]
pub struct ApifyOrgResponse {
    pub ip: String,
}

#[derive(Debug)]
pub struct ApifyOrg;
impl IPSource for ApifyOrg {
    fn name(&self) -> &'static str {
        "ApifyOrg"
    }
    fn url(&self) -> &str {
        "https://api.ipify.org?format=json"
    }
    fn parse_response(&self, body: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response: ApifyOrgResponse = serde_json::from_str(body)?;
        Ok(response.ip)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct IpinfoIoResponse {
    pub ip: String,
}

#[derive(Debug)]
pub struct IpinfoIo;
impl IPSource for IpinfoIo {
    fn name(&self) -> &'static str {
        "IpinfoIo"
    }
    fn url(&self) -> &str {
        "https://ipinfo.io/json"
    }
    fn parse_response(&self, body: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response: IpinfoIoResponse = serde_json::from_str(body)?;
        Ok(response.ip)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct IdentMeResponse {
    pub address: String,
}

#[derive(Debug)]
pub struct IdentMe;
impl IPSource for IdentMe {
    fn name(&self) -> &'static str {
        "IdentMe"
    }
    fn url(&self) -> &str {
        "https://ident.me/.json"
    }
    fn parse_response(&self, body: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response: IdentMeResponse = serde_json::from_str(body)?;
        Ok(response.address)
    }
}

pub fn random_ip_source() -> Box<dyn IPSource + Send + Sync> {
    match rand::rng().random_range(0..3) {
        0 => Box::new(ApifyOrg),
        1 => Box::new(IpinfoIo),
        _ => Box::new(IdentMe),
    }
}

pub async fn get_ip()
-> Result<(String, Box<dyn IPSource + Send + Sync>), Box<dyn std::error::Error>> {
    let random_source = random_ip_source();

    let body = reqwest::get(random_source.url()).await?.text().await?;
    let ip = random_source.parse_response(&body)?;

    Ok((ip, random_source))
}
