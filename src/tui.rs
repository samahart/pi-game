use std::io::{Stdout, stdout};

use crate::game::CharCorrectness;
use ratatui::{Terminal, backend::CrosstermBackend, layout::Layout, prelude::*, widgets::*};

const PADDING: usize = 2;

#[derive(Debug)]
pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Default for Tui {
    fn default() -> Self {
        let backend = CrosstermBackend::new(stdout());
        Self {
            terminal: Terminal::new(backend).expect("failed to startup tui"),
        }
    }
}

impl Tui {
    fn update_screen(&mut self, high_score: usize, current_score: usize, info: Line) {
        let current_score = format!("{}Current Score: {current_score}", " ".repeat(PADDING));
        let high_score = format!("High Score: {high_score}");
        self.terminal.clear().expect("error clearing terminal");
        self.terminal
            .draw(|frame| {
                let block = Block::default().borders(Borders::ALL);
                let inner_area = block.inner(frame.area());
                frame.render_widget(block, frame.area());
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(1), Constraint::Min(1)])
                    .split(inner_area);
                let top_line = Line::from(vec![
                    Span::raw(&current_score),
                    // create flexible space that pushes the right text to the edge
                    Span::styled(
                        " ".repeat(
                            chunks[0].width as usize
                                - current_score.len()
                                - high_score.len()
                                - PADDING,
                        ),
                        Style::default(),
                    ),
                    Span::raw(&high_score),
                ]);
                frame.render_widget(top_line, chunks[0]);
                let available_height = chunks[1].height;
                let center_text_height = 2; // allows up to two lines of text
                let vertical_padding = (available_height - center_text_height) / 2;
                let centered_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(vertical_padding),
                        Constraint::Length(center_text_height),
                        Constraint::Length(
                            available_height - vertical_padding - center_text_height,
                        ),
                    ])
                    .split(chunks[1]);
                frame.render_widget(info.centered(), centered_chunks[1]);
            })
            .expect("failed to draw frame");
    }

    pub fn update_user_input(
        &mut self,
        high_score: usize,
        current_score: usize,
        chars: &Vec<CharCorrectness>,
    ) {
        let user_input: Vec<Span> = chars
            .into_iter()
            .map(|i| {
                let color = match i.correct {
                    true => Color::Green,
                    false => Color::Red,
                };
                Span::styled(
                    i.c.to_string(),
                    Style::default().add_modifier(Modifier::BOLD).fg(color),
                )
            })
            .collect();
        self.update_screen(high_score, current_score, Line::from(user_input));
    }

    pub fn update_info(&mut self, high_score: usize, current_score: usize, info: &str) {
        self.update_screen(high_score, current_score, Line::from(info));
    }
}
