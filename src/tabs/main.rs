// This example demonstrates how to use the tabs widget
//
// This was written by Kaiden42 <gitlab@tinysn.com>

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{Column, Container, Text},
    Element, Length, Task,
};
use iced_aw::{TabLabel, Tabs};

use crate::{cmd, tabs::settings};
use settings::{style_from_index, SettingsMessage, SettingsTab, TabBarPosition};

use super::home::{HomeMessage, HomeTab};

const HEADER_SIZE: u16 = 32;
const TAB_PADDING: u16 = 16;

pub(crate) fn run_tabs(args: cmd::App) -> iced::Result {
    iced::application("Tabs example", TabBarExample::update, TabBarExample::view)
        .font(iced_fonts::NERD_FONT_BYTES)
        .run_with(|| {
            let (home_tab, home_tab_task) = HomeTab::new(args);
            let (settings_tab, settings_tab_task) = SettingsTab::new();
            let tasks = Task::batch(vec![
                home_tab_task.map(Message::Home),
                settings_tab_task.map(Message::Settings),
            ]);
            (
                TabBarExample {
                    active_tab: TabId::Home,
                    settings_tab,
                    home_tab,
                },
                tasks,
            )
        })
}

#[derive(Default)]
struct TabBarExample {
    active_tab: TabId,
    settings_tab: SettingsTab,
    home_tab: HomeTab,
}
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub enum TabId {
    #[default]
    Home,
    Settings,
}

#[derive(Clone, Debug)]
pub enum Message {
    TabSelected(TabId),
    Settings(SettingsMessage),
    Home(HomeMessage),
    TabClosed(TabId),
}

impl TabBarExample {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(selected) => {
                self.active_tab = selected;
                Task::none()
            }
            Message::Settings(message) => self.settings_tab.update(message).map(Message::Settings),
            Message::Home(message) => self.home_tab.update(message).map(Message::Home),
            Message::TabClosed(id) => {
                println!("Tab {:?} event hit", id);
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let position = self
            .settings_tab
            .settings()
            .tab_bar_position
            .unwrap_or_default();
        let theme = self
            .settings_tab
            .settings()
            .tab_bar_theme
            .unwrap_or_default();

        Tabs::new(Message::TabSelected)
            .tab_icon_position(iced_aw::tabs::Position::Bottom)
            .on_close(Message::TabClosed)
            .push(TabId::Home, self.home_tab.tab_label(), self.home_tab.view())
            .push(
                TabId::Settings,
                self.settings_tab.tab_label(),
                self.settings_tab.view(),
            )
            .set_active_tab(&self.active_tab)
            .tab_bar_style(style_from_index(theme))
            .tab_bar_position(match position {
                TabBarPosition::Top => iced_aw::TabBarPosition::Top,
                TabBarPosition::Bottom => iced_aw::TabBarPosition::Bottom,
            })
            .into()
    }
}

pub trait Tab {
    type Message;

    fn title(&self) -> String;

    fn tab_label(&self) -> TabLabel;

    fn view(&self) -> Element<'_, Self::Message> {
        let column = Column::new()
            .spacing(20)
            .push(Text::new(self.title()).size(HEADER_SIZE))
            .push(self.content())
            .align_x(iced::Alignment::Center);

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(TAB_PADDING)
            .into()
    }

    fn content(&self) -> Element<'_, Self::Message>;
}
