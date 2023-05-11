use crypto::digest::Digest;
use crypto::md5::Md5;
use serde::Deserialize;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tbs {
    pub tbs: String,
    is_login: i32,
}

#[serde_as]
#[derive(Debug, Serialize, thiserror::Error)]
#[serde(tag = "type", content = "message")]
pub enum Error {
    #[error("http error")]
    Http(
        #[serde_as(as = "DisplayFromStr")]
        #[from]
        reqwest::Error,
    ),
    #[error("serde error")]
    Serde(
        #[serde_as(as = "DisplayFromStr")]
        #[from]
        serde_json::Error,
    ),
    #[error("Infallible error")]
    Infallible(
        #[serde_as(as = "DisplayFromStr")]
        #[from]
        std::convert::Infallible,
    ),
    #[error("SystemTimeError")]
    SystemTimeError(
        #[serde_as(as = "DisplayFromStr")]
        #[from]
        std::time::SystemTimeError,
    ),
}

// 自定义错误类型
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize)]
pub struct ClientSignData {
    #[serde(rename = "BDUSS")]
    pub bduss: String,
    #[serde(rename = "FID")]
    pub fid: String,
    #[serde(rename = "KW")]
    pub kw: String,
    #[serde(rename = "TBS")]
    pub tbs: String,
    #[serde(rename = "TIMESTAMP")]
    pub timestamp: u64,
    #[serde(rename = "_client_type")]
    pub client_type: String,
    #[serde(rename = "_client_version")]
    pub client_version: String,
    #[serde(rename = "_phone_imei")]
    pub phone_imei: String,
    #[serde(rename = "model")]
    pub model: String,
    #[serde(rename = "net_type")]
    pub net_type: String,
}

impl ClientSignData {
    pub fn new(
        bduss: &str,
        fid: &str,
        kw: &str,
        tbs: &str,
        now_time: u64,
    ) -> Result<ClientSignData> {
        Ok(ClientSignData {
            bduss: bduss.to_string(),
            fid: fid.to_string(),
            kw: kw.to_string(),
            tbs: tbs.to_string(),
            timestamp: now_time,
            client_type: "2".to_string(),
            client_version: "9.7.8.0".to_string(),
            phone_imei: "000000000000000".to_string(),
            model: "MI+5".to_string(),
            net_type: "1".to_string(),
        })
    }
}

fn get_now_time() -> Result<String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs()
        .to_string())
}

pub fn get_hash_map(bd: String) -> Result<HashMap<&'static str, String>> {
    let now_time = get_now_time()?;
    let mut data = HashMap::new();
    data.insert("BDUSS", bd);
    data.insert("_client_id", "wappc_1593576610335_488".to_string());
    data.insert("_client_type", "2".to_string());
    data.insert("_client_version", "9.7.8.0".to_string());
    data.insert("_phone_imei", "000000000000000".to_string());
    data.insert("from", "1008621y".to_string());
    data.insert("model", "MI+5".to_string());
    data.insert("page_no", "1".to_string());
    data.insert("page_size", "200".to_string());
    data.insert("timestamp", now_time);
    data.insert("vcode_tag", "11".to_string());
    data.insert("net_type", "1".to_string());
    Ok(data)
}

pub fn encode_data<'a>(data: &HashMap<&str, String>) -> Result<String> {
    let mut s = String::new();
    let mut keys: Vec<&str> = data.keys().cloned().collect();
    keys.sort();
    for i in keys {
        s += i;
        s += "=";
        s += &*data[i];
    }
    s += "tiebaclient!!!";
    let mut hasher = Md5::new();
    hasher.input_str(&s);
    let sign = hasher.result_str();
    Ok(sign)
}
