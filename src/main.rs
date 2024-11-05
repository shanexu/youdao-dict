use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Entry {
    explain: String,
    entry: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Data {
    entries: Vec<Entry>,
    query: String,
    language: String,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    data_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResultData {
    msg: String,
    code: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiResponse {
    result: ResultData,
    data: Data,
}

async fn suggest(q: &str) -> Result<ApiResponse, Box<dyn std::error::Error>> {
    let mut url =
        Url::parse("https://dict.youdao.com/suggest?num=5&ver=3.0&doctype=json&cache=false&le=en")?;
    url.query_pairs_mut().append_pair("q", q);
    let response = reqwest::get(url).await?.json::<ApiResponse>().await?;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let resp = suggest("test").await?;
    println!("{resp:#?}");

    let body = reqwest::get("https://dict.youdao.com/result?word=fire&lang=en")
        .await?
        .text()
        .await?;

    println!("{}", body);

    Ok(())
}
