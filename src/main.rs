use reqwest::header;
use reqwest::Client as re_client;
use serde::de::Error;
use std::time::SystemTime;
use tiebaSign::{encode_data, get_hash_map, ClientSignData, Result, Tbs};
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    let tbs_url = String::from("http://tieba.baidu.com/dc/common/tbs");
    let bduss = std::env::var("BDUSS").expect("BDUSS not found");
    let url = tbs_url.parse::<reqwest::Url>().unwrap();
    let tbs = get_tbs(url, &bduss).await?;
    let favorite = get_favorite(&bduss).await?;
    let futures = favorite.into_iter().map(|i| {
        let bduss = bduss.to_owned();
        let user_id = i["id"].to_string();
        let user_name = i["name"].to_string();
        let tbs_data = tbs.tbs.to_string();
        task::spawn(async move { client_sign(&bduss, &tbs_data, &user_id, &user_name).await })
    });

    for fut in futures {
        if let Err(e) = fut.await {
            eprintln!("client_sign error: {:?}", e);
        }
    }
    Ok(())
}

async fn get_tbs(url: reqwest::Url, bduss: &str) -> Result<Tbs> {
    let mut headers = header::HeaderMap::new();
    headers.insert("Host", header::HeaderValue::from_static("tieba.baidu.com"));
    headers.insert("User-Agent", header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/39.0.2171.71 Safari/537.36"));
    headers.insert(
        "COOKIE",
        header::HeaderValue::from_str(&("BDUSS=".to_owned() + bduss)).unwrap(),
    );
    let client = re_client::builder().default_headers(headers).build()?;
    let req = client
        .post(url)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await?;
    let body = req.text().await?;
    let tbs: Tbs = serde_json::from_str(&body)?;
    Ok(tbs)
}

async fn get_favorite(bduss: &str) -> Result<Vec<serde_json::Value>> {
    let mut data = get_hash_map(bduss.to_string())?;
    let sign = encode_data(&data)?;
    data.insert("sign", sign);
    let client = re_client::new();
    let req = client
        .post("http://c.tieba.baidu.com/c/f/forum/like")
        .timeout(std::time::Duration::from_secs(60))
        .form(&data)
        .send()
        .await?;
    let body = req.text().await?;
    let mut return_data: serde_json::Value = serde_json::from_str(&body)?;
    let mut t: Vec<serde_json::Value> = Vec::new();
    let forum_list = return_data["forum_list"]
        .as_object_mut()
        .ok_or_else(|| serde_json::Error::custom("forum_list not found"))?;
    forum_list
        .entry("non-gconforum".to_string())
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));
    forum_list
        .entry("gconforum".to_string())
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));

    for forum_type in ["non-gconforum", "gconforum"].iter() {
        let forum_array = forum_list
            .get_mut(*forum_type)
            .unwrap()
            .as_array_mut()
            .ok_or_else(|| serde_json::Error::custom(format!("{} is not an array", forum_type)))?;
        for forum in forum_array.iter_mut() {
            if let Some(forum_array) = forum.as_array_mut() {
                for subforum in forum_array.iter_mut() {
                    t.push(subforum.take());
                }
            } else {
                t.push(forum.take());
            }
        }
    }

    Ok(t)
}

async fn client_sign(bduss: &str, tbs: &str, fid: &str, kw: &str) -> Result<()> {
    let now_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let data = ClientSignData::new(bduss, tbs, fid, kw, now_time);
    let client = re_client::new();
    let _res = client
        .post("http://c.tieba.baidu.com/c/c/forum/sign")
        .form(&data)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await?;
    Ok(())
}
