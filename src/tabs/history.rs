use iced::Element;
use iced::Task;
use iced_aw::TabLabel;
use iced_fonts::Nerd;

use super::main::Message;
use super::main::Tab;

#[derive(Debug, Clone)]
pub enum HistoryMessage {
}

pub struct HistoryTab {
}

impl HistoryTab {
    pub fn update(self: &mut Self, _message: HistoryMessage) -> Task<HistoryMessage> {
        Task::none()
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
        "history".into()
    }
}
