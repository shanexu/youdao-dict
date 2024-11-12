use std::error::Error;

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
pub struct ApiResponse {
    result: ResultData,
    data: Data,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WordResult {
    pub word_head: String,
    pub phone_con: String,
    pub simple_dict: String,
    pub catalogue_sentence: String,
    pub not_found: bool,
    pub maybe: String,
}

#[allow(dead_code)]
pub async fn suggest(client: &Client, q: &str) -> Result<ApiResponse, Box<dyn Error>> {
    let mut url =
        Url::parse("https://dict.youdao.com/suggest?num=5&ver=3.0&doctype=json&cache=false&le=en")?;
    url.query_pairs_mut().append_pair("q", q);
    let response = client.get(url).send().await?.json::<ApiResponse>().await?;
    Ok(response)
}

pub async fn word_result(client: &Client, word: &str) -> Result<WordResult, Box<dyn Error>> {
    let mut url = Url::parse("https://dict.youdao.com/result?lang=en")?;
    url.query_pairs_mut().append_pair("word", word);
    let body = client.get(url).send().await?.text().await?;
    parse_word_result_body(word, &body)
}

fn parse_word_result_body(word: &str, body: &str) -> Result<WordResult, Box<dyn Error>> {
    let dom = Html::parse_document(body);

    let word_head_selector = Selector::parse(".word-head .title")?;
    let (word_head, not_found) = match dom.select(&word_head_selector).next() {
        Some(el) => (el.text().next().unwrap().to_string(), false),
        None => (word.to_string(), true),
    };

    let maybe_selector = Selector::parse(".maybe")?;
    let maybe = dom
        .select(&maybe_selector)
        .into_iter()
        .next()
        .map(|t| t.html())
        .map(|h| html2text::from_read(h.as_bytes(), 200).unwrap())
        .unwrap_or_default();

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

    let catalogue_sentence = parse_catalogue_sentence(&dom).unwrap_or_default();

    Ok(WordResult {
        word_head,
        phone_con,
        simple_dict,
        catalogue_sentence,
        not_found,
        maybe,
    })
}

fn parse_catalogue_sentence(dom: &Html) -> Option<String> {
    let catalogue_sentence_selector =
        Selector::parse("#catalogue_sentence .dict-book ul > li").unwrap();
    let els = dom
        .select(&catalogue_sentence_selector)
        .into_iter()
        .enumerate();
    let sen_eng_selector = Selector::parse(".sen-eng").unwrap();
    let sen_ch_selector = Selector::parse(".sen-ch").unwrap();
    let secondary_selector = Selector::parse(".secondary").unwrap();
    Some(
        els.flat_map(|(index, el)| {
            let eng = el
                .select(&sen_eng_selector)
                .into_iter()
                .next()
                .map(|x| x.inner_html())
                .map(|x| x.replace("<b>", "**").replace("</b>", "**"))?;
            let cn = el
                .select(&sen_ch_selector)
                .into_iter()
                .next()
                .map(|x| x.inner_html())?;
            let dict = el
                .select(&secondary_selector)
                .into_iter()
                .next()
                .map(|x| x.inner_html())?;
            let idx = index + 1;
            let idx_str = format!("{}. ", idx);
            let indent = " ".repeat(idx_str.len());
            Some(format!(
                "{}{}\n{}{}\n{}{}",
                idx_str, eng, indent, cn, indent, dict
            ))
        })
        .collect::<Vec<String>>()
        .join("\n"),
    )
}

#[cfg(test)]
#[test]
fn test_parse_catalogue_sentence() -> Result<(), Box<dyn Error>> {
    use std::fs;
    let body = fs::read_to_string("fire.html").unwrap();
    let dom = Html::parse_document(&body);
    println!("{}", parse_catalogue_sentence(&dom).unwrap());
    Ok(())
}
