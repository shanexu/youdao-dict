use std::error::Error;

use reqwest::{Client, Url};
use scraper::{selectable::Selectable, Html, Selector};
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
    let word_head_opt = parse_word_head(&dom);
    let not_found = word_head_opt.is_none();
    let maybe = parse_maybe(&dom).unwrap_or_default();
    let phone_con = parse_phone_con(&dom).unwrap_or_default();
    let simple_dict = parse_simple_dict(&dom).unwrap_or_default();
    let catalogue_sentence = parse_catalogue_sentence(&dom).unwrap_or_default();

    Ok(WordResult {
        word_head: word_head_opt.unwrap_or_else(|| format!("# {}:", word)),
        phone_con,
        simple_dict,
        catalogue_sentence,
        not_found,
        maybe,
    })
}

fn parse_maybe(dom: &Html) -> Option<String> {
    let maybe_selector = Selector::parse(".maybe").unwrap();
    dom.select(&maybe_selector)
        .next()
        .map(|t| t.text().collect::<Vec<_>>().join("\n"))
}

fn parse_word_head(dom: &Html) -> Option<String> {
    let word_head_selector = Selector::parse(".word-head .title").unwrap();
    dom.select(&word_head_selector)
        .next()
        .and_then(|el| el.text().next())
        .map(|h| format!("# {}", h))
}

fn parse_phone_con(dom: &Html) -> Option<String> {
    let per_phone_selector = Selector::parse(".phone_con .per-phone").unwrap();
    Some(
        dom.select(&per_phone_selector)
            .map(|el| format!("- {}", el.text().collect::<Vec<_>>().join(" ")))
            .collect::<Vec<String>>()
            .join("\n"),
    )
}

fn parse_simple_dict(dom: &Html) -> Option<String> {
    let simple_dict_selector = Selector::parse(".simple.dict-module").unwrap();
    let simple_dict_el = dom.select(&simple_dict_selector).into_iter().next()?;
    let word_exp_selector = Selector::parse(".word-exp").unwrap();
    let mut word_exps = simple_dict_el
        .select(&word_exp_selector)
        .map(|el| format!("- {}", el.text().collect::<Vec<_>>().join(" ")))
        .collect::<Vec<String>>();
    let word_wfs_less_selector = Selector::parse(".word-wfs-less").unwrap();
    let word_wfs_less_opt = simple_dict_el
        .select(&word_wfs_less_selector)
        .next()
        .map(|el| format!("- {}", el.text().collect::<Vec<_>>().join(" ")));
    if let Some(word_wfs_less) = word_wfs_less_opt {
        word_exps.push(word_wfs_less)
    }
    Some(word_exps.join("\n"))
}

fn parse_catalogue_sentence(dom: &Html) -> Option<String> {
    let catalogue_sentence_selector =
        Selector::parse("#catalogue_sentence .dict-book ul > li").unwrap();
    let els = dom.select(&catalogue_sentence_selector).enumerate();
    let sen_eng_selector = Selector::parse(".sen-eng").unwrap();
    let sen_ch_selector = Selector::parse(".sen-ch").unwrap();
    let secondary_selector = Selector::parse(".secondary").unwrap();
    Some(
        els.flat_map(|(index, el)| {
            let eng = el
                .select(&sen_eng_selector)
                .next()
                .map(|x| x.inner_html())
                .map(|x| x.replace("<b>", "**").replace("</b>", "**"))?;
            let cn = el.select(&sen_ch_selector).next().map(|x| x.inner_html())?;
            let dict = el
                .select(&secondary_selector)
                .next()
                .map(|x| x.inner_html())?;
            let idx = index + 1;
            let idx_str = format!("{}. ", idx);
            let indent = " ".repeat(idx_str.len());
            Some(format!(
                "{}{}\\\n{}{}\\\n{}{}",
                idx_str, eng, indent, cn, indent, dict
            ))
        })
        .collect::<Vec<String>>()
        .join("\n"),
    )
}

#[cfg(test)]
#[test]
fn test_parse_normal() -> Result<(), Box<dyn Error>> {
    use std::fs;
    let body = fs::read_to_string("fire.html")?;
    let dom = Html::parse_document(&body);

    println!("{:?}", parse_word_head(&dom).unwrap());
    println!("{}", parse_phone_con(&dom).unwrap());
    println!("{}", parse_simple_dict(&dom).unwrap());
    println!("{}", parse_catalogue_sentence(&dom).unwrap());

    Ok(())
}

#[cfg(test)]
#[test]
fn test_parse_maybe() -> Result<(), Box<dyn Error>> {
    use std::fs;
    let body = fs::read_to_string("zlib.html")?;
    let dom = Html::parse_document(&body);

    println!("{}", parse_maybe(&dom).unwrap());
    Ok(())
}
