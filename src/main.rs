#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod assistant;
mod icon;
mod screen;

use crate::screen::boot;
use crate::screen::conversation;
use crate::screen::search;
use crate::screen::Screen;

use iced::system;
use iced::{Element, Subscription, Task, Theme};

pub fn main() -> iced::Result {
    iced::application(Chat::title, Chat::update, Chat::view)
        .font(icon::FONT_BYTES)
        .subscription(Chat::subscription)
        .theme(Chat::theme)
        .run_with(Chat::new)
}

struct Chat {
    screen: Screen,
    system: Option<system::Information>,
}

#[derive(Debug, Clone)]
enum Message {
    Search(search::Message),
    Boot(boot::Message),
    Conversation(conversation::Message),
    SystemFetched(system::Information),
    Escape,
}

impl Chat {
    pub fn new() -> (Self, Task<Message>) {
        let (search, task) = screen::Search::new();

        (
            Self {
                screen: Screen::Search(search),
                system: None,
            },
            Task::batch([
                system::fetch_information().map(Message::SystemFetched),
                task.map(Message::Search),
            ]),
        )
    }

    fn title(&self) -> String {
        match &self.screen {
            Screen::Search(search) => search.title(),
            Screen::Boot(boot) => boot.title(),
            Screen::Conversation(conversation) => conversation.title(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Search(message) => {
                if let Screen::Search(search) = &mut self.screen {
                    let action = search.update(message);

                    match action {
                        search::Action::None => Task::none(),
                        search::Action::Run(task) => task.map(Message::Search),
                        search::Action::Boot(model) => {
                            self.screen =
                                Screen::Boot(screen::Boot::new(model, self.system.as_ref()));

                            Task::none()
                        }
                    }
                } else {
                    Task::none()
                }
            }
            Message::Boot(message) => {
                if let Screen::Boot(search) = &mut self.screen {
                    let action = search.update(message);

                    match action {
                        boot::Action::None => Task::none(),
                        boot::Action::Run(task) => task.map(Message::Boot),
                        boot::Action::Finish(assistant) => {
                            let (conversation, task) = screen::Conversation::new(assistant);

                            self.screen = Screen::Conversation(conversation);

                            task.map(Message::Conversation)
                        }
                        boot::Action::Abort => self.search(),
                    }
                } else {
                    Task::none()
                }
            }
            Message::Conversation(message) => {
                if let Screen::Conversation(conversation) = &mut self.screen {
                    let action = conversation.update(message);

                    match action {
                        conversation::Action::None => Task::none(),
                        conversation::Action::Run(task) => task.map(Message::Conversation),
                        conversation::Action::Back => self.search(),
                    }
                } else {
                    Task::none()
                }
            }
            Message::SystemFetched(system) => {
                self.system = Some(system);

                Task::none()
            }
            Message::Escape => {
                if matches!(self.screen, Screen::Search(_)) {
                    Task::none()
                } else {
                    self.search()
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match &self.screen {
            Screen::Search(search) => search.view().map(Message::Search),
            Screen::Boot(boot) => boot.view().map(Message::Boot),
            Screen::Conversation(conversation) => conversation.view().map(Message::Conversation),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        use iced::keyboard;

        let screen = match &self.screen {
            Screen::Search(search) => search.subscription().map(Message::Search),
            Screen::Boot(boot) => boot.subscription().map(Message::Boot),
            Screen::Conversation(_) => Subscription::none(),
        };

        let hotkeys = keyboard::on_key_press(|key, _modifiers| match key {
            keyboard::Key::Named(keyboard::key::Named::Escape) => Some(Message::Escape),
            _ => None,
        });

        Subscription::batch([screen, hotkeys])
    }

    fn theme(&self) -> Theme {
        Theme::TokyoNight
    }

    fn search(&mut self) -> Task<Message> {
        let (search, task) = screen::Search::new();

        self.screen = Screen::Search(search);

        task.map(Message::Search)
    }
}
