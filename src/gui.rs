use crate::cmd;
use crate::youdao;
use iced::widget::{button, column, row, text_input, Text};
use iced::Font;
use iced::{Element, Task};
use reqwest::Client;
use std::borrow::Borrow;
use std::sync::Arc;

pub fn run_gui(args: cmd::App) -> iced::Result {
    let client = Client::builder().user_agent("curl/8.10.1").build().unwrap();
    iced::application("Youdao Dict", State::update, view)
        .run_with(|| {
            (
                State {
                    client: Arc::new(client),
                    input_value: args.global_opts.word,
                    word_result: None,
                },
                Task::done(Message::SearchWord),
            )
        })
}

#[derive(Debug, Clone)]
enum Message {
    InputChange(String),
    SearchWord,
    ResultFetched(Option<youdao::WordResult>),
}

#[derive(Debug, Default)]
struct State {
    client: Arc<Client>,
    input_value: Option<String>,
    word_result: Option<youdao::WordResult>,
}

impl State {
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
                Task::none()
            }
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    // button(text(state.count)).on_press(Message::Increment).into()
    let content = match &state.word_result {
        Some(youdao::WordResult {
            word_head,
            phone_con,
            simple_dict,
            catalogue_sentence,
            not_found,
            maybe,
        }) => {
            if *not_found {
                format!("{}:\n\n{}\n", word_head, maybe)
            } else {
                format!(
                    "{}:\n\n{}\n{}\n{}\n",
                    word_head, phone_con, simple_dict, catalogue_sentence
                )
            }
        }
        None => "".to_string(),
    };
    let input = text_input("", state.input_value.as_deref().unwrap_or_default())
        .id("word")
        .on_input(Message::InputChange)
        .on_submit(Message::SearchWord);
    let f = Font::with_name("LXGW Neo XiHei Screen Full");
    column![
        row![input, button("Search").on_press(Message::SearchWord),],
        row![Text::new(content).font(f),],
    ]
    .into()
}

async fn search_word(client: &Client, word: &str) -> Option<youdao::WordResult> {
    youdao::word_result(client, word).await.ok()
}
