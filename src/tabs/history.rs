use std::cell::RefCell;
use std::rc::Rc;

use super::common::DEFAULT_FONT;
use super::main::Message;
use super::main::Tab;
use super::main::TabId;
use crate::db;
use crate::models::History;
use diesel::SqliteConnection;
use iced::widget::{column, row, text, button};
use iced::Element;
use iced::Task;
use iced_aw::TabLabel;
use iced_fonts::Nerd;

#[derive(Debug, Clone)]
pub enum HistoryMessage {
    LoadHistory,
}

pub struct HistoryTab {
    conn: Rc<RefCell<SqliteConnection>>,
    history: Vec<History>,
}

impl HistoryTab {
    pub fn new(conn: Rc<RefCell<SqliteConnection>>) -> (Self, Task<HistoryMessage>) {
        (
            Self {
                conn,
                history: vec![],
            },
            Task::none(),
        )
    }
    pub fn update(self: &mut Self, message: HistoryMessage) -> Task<HistoryMessage> {
        match message {
            HistoryMessage::LoadHistory => {
                self.history = db::list_history(&mut self.conn.borrow_mut());
                Task::none()
            }
        }
    }
}

impl Tab for HistoryTab {
    type Message = Message;

    fn title(&self) -> String {
        String::from("History")
    }

    fn tab_label(&self) -> iced_aw::TabLabel {
        TabLabel::IconText(Nerd::History.into(), self.title())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        let content: Element<'_, Message> = column(self.history.iter().map(|i| {
            row![
                text!("{}", i.id),
                text!("{}", i.word),
                text!("{}", i.created_at),
                button(text!("查看").font(DEFAULT_FONT))
                    .on_press(Message::List(vec![
                        Message::TabSelected(TabId::Home),
                        Message::Home(super::home::HomeMessage::InputChange(i.word.clone())),
                        Message::Home(super::home::HomeMessage::SearchWord),
                    ]))
            ]
            .into()
        }))
        .into();
        content
    }
}
