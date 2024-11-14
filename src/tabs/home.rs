use crate::cmd;
use crate::youdao;
use iced::widget::{button, column, markdown, row, scrollable, text_input, Text};
use iced::Font;
use iced::Theme;
use iced::{Element, Task};
use iced_aw::TabLabel;
use iced_fonts::Nerd;
use reqwest::Client;
use std::borrow::Borrow;
use std::sync::Arc;

use super::main::Message;
use super::main::Tab;

#[derive(Debug, Clone)]
pub enum HomeMessage {
    InputChange(String),
    SearchWord,
    ResultFetched(Option<youdao::WordResult>),
    LinkClicked(markdown::Url),
}

#[derive(Default)]
pub struct HomeTab {
    client: Arc<Client>,
    input_value: Option<String>,
    word_result: Option<youdao::WordResult>,
    markdown_items: Vec<markdown::Item>,
    word_result_content: String,
}

impl HomeTab {
    pub fn new(args: cmd::App) -> (Self, Task<HomeMessage>) {
        let client = Client::builder().user_agent("curl/8.10.1").build().unwrap();
        (
            Self {
                client: Arc::new(client),
                input_value: args.global_opts.word,
                word_result: None,
                markdown_items: vec![],
                word_result_content: "".to_string(),
            },
            Task::done(HomeMessage::SearchWord),
        )
    }

    pub fn update(self: &mut Self, message: HomeMessage) -> Task<HomeMessage> {
        match message {
            HomeMessage::InputChange(value) => {
                self.input_value = Some(value);
                Task::none()
            }
            HomeMessage::SearchWord => {
                let word = self.input_value.take();
                if word.is_none() {
                    return Task::none();
                }
                let cloned_client = Arc::clone(&self.client);
                Task::perform(
                    async move {
                        search_word(cloned_client.borrow(), word.as_deref().unwrap_or_default())
                            .await
                    },
                    HomeMessage::ResultFetched,
                )
            }
            HomeMessage::ResultFetched(result) => {
                self.word_result = result;
                let content = match self.word_result {
                    Some(ref r) => {
                        if r.not_found {
                            format!("{}:\n\n{}\n", r.word_head, r.maybe)
                        } else {
                            format!(
                                "{}:\n\n{}\n\n{}\n\n{}\n",
                                r.word_head, r.phone_con, r.simple_dict, r.catalogue_sentence
                            )
                        }
                    }
                    None => "".to_string(),
                };
                self.word_result_content = content;
                self.markdown_items = markdown::parse(&self.word_result_content).collect();
                Task::none()
            }
            HomeMessage::LinkClicked(url) => {
                println!("The following url was clicked: {url}");
                Task::none()
            }
        }
    }
}

impl Tab for HomeTab {
    type Message = Message;

    fn title(&self) -> String {
        String::from("Home")
    }

    fn tab_label(&self) -> iced_aw::TabLabel {
        TabLabel::IconText(Nerd::Home.into(), self.title())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        let ft = Font::with_name("LXGW Neo XiHei Screen Full");
        let input = text_input("", self.input_value.as_deref().unwrap_or_default())
            .id("word")
            .on_input(HomeMessage::InputChange)
            .on_submit(HomeMessage::SearchWord);
        let preview = markdown(
            &self.markdown_items,
            markdown::Settings::default(),
            markdown::Style::from_palette(Theme::TokyoNightStorm.palette()),
        )
        .map(HomeMessage::LinkClicked);
        let content: Element<'_, HomeMessage> = column![
            row![
                input,
                button(Text::new("查询").font(ft)).on_press(HomeMessage::SearchWord),
                button(Text::new("收藏").font(ft)).on_press(HomeMessage::SearchWord),
            ],
            scrollable(preview).spacing(10)
        ]
        .into();
        content.map(Message::Home)
    }
}

async fn search_word(client: &Client, word: &str) -> Option<youdao::WordResult> {
    youdao::word_result(client, word).await.ok()
}
