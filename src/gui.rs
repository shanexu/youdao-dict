use crate::cmd;
use crate::youdao;
use iced::widget::{button, column, markdown, row, scrollable, text_input, Text};
use iced::Font;
use iced::Theme;
use iced::{Element, Task};
use reqwest::Client;
use std::borrow::Borrow;
use std::sync::Arc;

const ICON_BYTES: &[u8] = include_bytes!("./tabs/fonts/icons.ttf");

pub(crate) fn run_gui(args: cmd::App) -> iced::Result {
    iced::application("Youdao Dict", State::update, State::view)
        .font(iced_fonts::REQUIRED_FONT_BYTES)
        .font(iced_fonts::NERD_FONT_BYTES)
        .font(ICON_BYTES)
        .run_with(|| State::new(args))
}

#[derive(Debug, Clone)]
enum Message {
    InputChange(String),
    SearchWord,
    ResultFetched(Option<youdao::WordResult>),
    LinkClicked(markdown::Url),
}

#[derive(Debug, Default)]
struct State {
    client: Arc<Client>,
    input_value: Option<String>,
    word_result: Option<youdao::WordResult>,
    markdown_items: Vec<markdown::Item>,
    word_result_content: String,
}

impl State {
    fn new(args: cmd::App) -> (Self, Task<Message>) {
        let client = Client::builder().user_agent("curl/8.10.1").build().unwrap();
        (
            Self {
                client: Arc::new(client),
                input_value: args.global_opts.word,
                word_result: None,
                markdown_items: vec![],
                word_result_content: "".to_string(),
            },
            Task::done(Message::SearchWord),
        )
    }
    fn update(self: &mut State, message: Message) -> Task<Message> {
        match message {
            Message::InputChange(value) => {
                self.input_value = Some(value);
                Task::none()
            }
            Message::SearchWord => {
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
                    Message::ResultFetched,
                )
            }
            Message::ResultFetched(result) => {
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
            Message::LinkClicked(url) => {
                println!("The following url was clicked: {url}");
                Task::none()
            }
        }
    }

    fn view(self: &State) -> Element<'_, Message> {
        let ft = Font::with_name("LXGW Neo XiHei Screen Full");
        let input = text_input("", self.input_value.as_deref().unwrap_or_default())
            .id("word")
            .on_input(Message::InputChange)
            .on_submit(Message::SearchWord);
        let preview = markdown(
            &self.markdown_items,
            markdown::Settings::default(),
            markdown::Style::from_palette(Theme::TokyoNightStorm.palette()),
        )
        .map(Message::LinkClicked);
        column![
            row![
                input,
                button(Text::new("查询").font(ft)).on_press(Message::SearchWord),
                button(Text::new("收藏").font(ft)).on_press(Message::SearchWord),
            ],
            scrollable(preview).spacing(10)
        ]
        .into()
    }
}

async fn search_word(client: &Client, word: &str) -> Option<youdao::WordResult> {
    youdao::word_result(client, word).await.ok()
}
