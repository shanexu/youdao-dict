use color_eyre::Result;
use crate::youdao::{WordResult, word_result};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph},
    DefaultTerminal, Frame,
};
use reqwest::Client;
use crate::cmd;

// /// A type alias for the terminal type used in this application
// pub type Tui = Terminal<CrosstermBackend<Stdout>>;

// /// Initialize the terminal
// pub fn init() -> io::Result<Tui> {
//     execute!(stdout(), EnterAlternateScreen)?;
//     enable_raw_mode()?;
//     set_panic_hook();
//     Terminal::new(CrosstermBackend::new(stdout()))
// }

// fn set_panic_hook() {
//     let hook = std::panic::take_hook();
//     std::panic::set_hook(Box::new(move |panic_info| {
//         let _ = restore(); // ignore any errors as we are already failing
//         hook(panic_info);
//     }));
// }

// /// Restore the terminal to its original state
// pub fn restore() -> io::Result<()> {
//     execute!(stdout(), LeaveAlternateScreen)?;
//     disable_raw_mode()?;
//     Ok(())
// }

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    input_mode: InputMode,
    /// Word search result
    word_result: Option<WordResult>,
    client: Client,
}

enum InputMode {
    Normal,
    Editing,
}

impl App {
    pub fn new(input: String) -> Self {
        let client = Client::builder().user_agent("curl/8.10.1").build().unwrap();
        Self {
            input,
            input_mode: InputMode::Editing,
            character_index: 0,
            word_result: None,
            client,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    async fn submit_message(&mut self) {
        let word_result = word_result(&self.client, &self.input)
            .await
            .unwrap();
        self.word_result = Some(word_result);
        self.input.clear();
        self.reset_cursor();
    }

    async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            self.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Esc => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => self.submit_message().await,
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Esc => self.input_mode = InputMode::Normal,
                        _ => {}
                    },
                    InputMode::Editing => {}
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, messages_area] = vertical.areas(frame.area());

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " or ".into(),
                    "Esc".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing.".bold(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to search".into(),
                ],
                Style::default(),
            ),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[allow(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                input_area.x + self.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )),
        }

        let text = match &self.word_result {
            Some(WordResult {
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
        let paragraph = Paragraph::new(text).block(Block::bordered().title("Result"));
        frame.render_widget(paragraph, messages_area);
    }
}

pub(crate) async fn run_tui(args: cmd::App) -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = App::new(args.global_opts.word.unwrap_or_default());
    let app_result = app.run(terminal).await;
    ratatui::restore();
    app_result
}

