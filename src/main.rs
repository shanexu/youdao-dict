use reqwest::{Client, Url};
use scraper::{Html, Selector};
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

async fn suggest(client: &Client, q: &str) -> Result<ApiResponse, Box<dyn std::error::Error>> {
    let mut url =
        Url::parse("https://dict.youdao.com/suggest?num=5&ver=3.0&doctype=json&cache=false&le=en")?;
    url.query_pairs_mut().append_pair("q", q);
    let response = client.get(url).send().await?.json::<ApiResponse>().await?;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().user_agent("curl/8.10.1").build()?;
    let resp = suggest(&client, "test").await?;
    println!("{resp:#?}");

    let body = client
        .get("https://dict.youdao.com/result?word=fire&lang=en")
        .send()
        .await?
        .text()
        .await?;
    let dom = Html::parse_document(&body);
    let phone_con_selector = Selector::parse(".phone_con")?;
    let phone_con_content = dom
        .select(&phone_con_selector)
        .into_iter()
        .next()
        .unwrap()
        .html();
    let phone_con_content = html2text::from_read(phone_con_content.as_bytes(), 200).unwrap();
    println!("{}", phone_con_content);
    let simple_dict_selector = Selector::parse(".simple.dict-module")?;
    let simple_dict_content = dom
        .select(&simple_dict_selector)
        .into_iter()
        .next()
        .unwrap()
        .html();
    let simple_dict_content = html2text::from_read(simple_dict_content.as_bytes(), 200).unwrap();
    println!("{}", simple_dict_content);
    let catalogue_sentence_selector = Selector::parse("#catalogue_sentence .dict-book")?;
    let catalogue_sentence_content = dom
        .select(&catalogue_sentence_selector)
        .into_iter()
        .next()
        .unwrap()
        .html();
    let catalogue_sentence_content = html2text::from_read(catalogue_sentence_content.as_bytes(), 200).unwrap();
    println!("{}", catalogue_sentence_content);
    Ok(())
}
