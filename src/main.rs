use std::{env::args, error::Error};

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

#[derive(Debug, Deserialize, Serialize)]
struct WordPage {
    word_head: String,
    phone_con: String,
    simple_dict: String,
    catalogue_sentence: String,
}

#[allow(dead_code)]
async fn suggest(client: &Client, q: &str) -> Result<ApiResponse, Box<dyn Error>> {
    let mut url =
        Url::parse("https://dict.youdao.com/suggest?num=5&ver=3.0&doctype=json&cache=false&le=en")?;
    url.query_pairs_mut().append_pair("q", q);
    let response = client.get(url).send().await?.json::<ApiResponse>().await?;
    Ok(response)
}

async fn word_result(client: &Client, word: &str) -> Result<WordPage, Box<dyn Error>> {
    let mut url = Url::parse("https://dict.youdao.com/result?lang=en")?;
    url.query_pairs_mut().append_pair("word", word);
    let body = client.get(url).send().await?.text().await?;
    let dom = Html::parse_document(&body);
    let word_head_selector = Selector::parse(".word-head .title")?;
    let word_head: String = dom
        .select(&word_head_selector)
        .next()
        .unwrap()
        .text()
        .into_iter()
        .next()
        .unwrap()
        .to_string();
    let phone_con_selector = Selector::parse(".phone_con")?;
    let phone_con = dom
        .select(&phone_con_selector)
        .into_iter()
        .next()
        .map(|t| t.html())
        .map(|h| html2text::from_read(h.as_bytes(), 200).unwrap())
        .unwrap_or_default();

    let simple_dict_selector = Selector::parse(".simple.dict-module")?;
    let simple_dict = dom
        .select(&simple_dict_selector)
        .into_iter()
        .next()
        .map(|t| t.html())
        .map(|h| html2text::from_read(h.as_bytes(), 200).unwrap())
        .unwrap_or_default();

    let catalogue_sentence_selector = Selector::parse("#catalogue_sentence .dict-book")?;
    let catalogue_sentence = dom
        .select(&catalogue_sentence_selector)
        .into_iter()
        .next()
        .map(|t| t.html())
        .map(|h| html2text::from_read(h.as_bytes(), 200).unwrap())
        .unwrap_or_default();

    Ok(WordPage {
        word_head,
        phone_con,
        simple_dict,
        catalogue_sentence,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::builder().user_agent("curl/8.10.1").build()?;
    // let resp = suggest(&client, "test").await?;
    // println!("{resp:#?}");
    let args: Vec<String> = args().collect();
    let mut args_iter = args.iter();
    args_iter.next();
    let word = args_iter.next().unwrap();

    let result = word_result(&client, &word).await?;
    println!(
        "{}:\n{}\n{}\n{}\n",
        result.word_head, result.phone_con, result.simple_dict, result.catalogue_sentence
    );

    Ok(())
}
