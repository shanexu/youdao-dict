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
    phone_con: String,
    simple_dict: String,
    catalogue_sentence: String,
}

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
    let phone_con_selector = Selector::parse(".phone_con")?;
    let phone_con_content = dom
        .select(&phone_con_selector)
        .into_iter()
        .next()
        .unwrap()
        .html();
    let phone_con_content = html2text::from_read(phone_con_content.as_bytes(), 200).unwrap();

    let simple_dict_selector = Selector::parse(".simple.dict-module")?;
    let simple_dict_content = dom
        .select(&simple_dict_selector)
        .into_iter()
        .next()
        .unwrap()
        .html();
    let simple_dict_content = html2text::from_read(simple_dict_content.as_bytes(), 200).unwrap();

    let catalogue_sentence_selector = Selector::parse("#catalogue_sentence .dict-book")?;
    let catalogue_sentence_content = dom
        .select(&catalogue_sentence_selector)
        .into_iter()
        .next()
        .unwrap()
        .html();
    let catalogue_sentence_content =
        html2text::from_read(catalogue_sentence_content.as_bytes(), 200).unwrap();

    Ok(WordPage {
        phone_con: phone_con_content,
        simple_dict: simple_dict_content,
        catalogue_sentence: catalogue_sentence_content,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::builder().user_agent("curl/8.10.1").build()?;
    // let resp = suggest(&client, "test").await?;
    // println!("{resp:#?}");
    let args: Vec<String> = args().collect();
    let mut args_iter = args.into_iter();
    args_iter.next();
    let w = args_iter.next().unwrap();

    let word = word_result(&client, &w).await?;
    println!(
        "{}\n{}\n{}\n",
        word.phone_con, word.simple_dict, word.catalogue_sentence
    );

    Ok(())
}
