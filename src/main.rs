use std::{env::args, error::Error};

use reqwest::Client;

mod youdao;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::builder().user_agent("curl/8.10.1").build()?;
    // let resp = suggest(&client, "test").await?;
    // println!("{resp:#?}");
    let args: Vec<String> = args().collect();
    let mut args_iter = args.iter();
    args_iter.next();
    let word = args_iter.next().unwrap();

    let result = youdao::word_result(&client, &word).await?;
    println!(
        "{}:\n{}\n{}\n{}\n",
        result.word_head, result.phone_con, result.simple_dict, result.catalogue_sentence
    );

    Ok(())
}

