use crate::tui::Tui;
use std::ops::{Index, IndexMut};
use std::thread::sleep;
use std::time::Duration;

const PI: &str = "3.14159265358979323846264338327950288419716939937510582097494459230781640628620899862803482534211706";
const MAX_SCORE: usize = PI.len();
const GAME_OVER_PAUSE: Duration = Duration::from_secs(4);
const GAME_WIN_PAUSE: Duration = Duration::from_secs(2);

#[derive(Debug)]
struct PiChars([char; MAX_SCORE]);

impl Default for PiChars {
    fn default() -> Self {
        Self(['_'; MAX_SCORE])
    }
}

impl Index<usize> for PiChars {
    type Output = char;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for PiChars {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[derive(Debug, PartialEq)]
enum State {
    Waiting,
    Playing,
}

impl Default for State {
    fn default() -> Self {
        State::Waiting
    }
}

#[derive(Debug, Default)]
pub struct Game {
    high_score: usize,
    current_score: usize,
    state: State,
    chars_pi: PiChars,
    chars_input: Vec<CharCorrectness>,
    tui: Tui,
}

#[derive(Debug, Default)]
pub struct CharCorrectness {
    pub c: char,
    pub correct: bool,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Self::default();
        for (i, c) in PI.chars().enumerate() {
            game.chars_pi[i] = c;
        }
        game.update_score();
        game
    }

    pub fn play(&mut self, mut words: Vec<String>) {
        // Check for start of game
        if self.state == State::Waiting {
            if let Some(more_words) = self.find_start(&words) {
                words = more_words;
            }
        }
        // Play game
        if self.state == State::Playing {
            self.play_words(words);
        }
    }

    // Searches for the word "start" and returns remaining words that could be playable if start was found
    fn find_start(&mut self, words: &Vec<String>) -> Option<Vec<String>> {
        for (i, word) in words.iter().enumerate() {
            if word == "start" {
                self.state = State::Playing;
                self.update_score();
                if i < words.len() {
                    return Some(words[i + 1..].to_vec());
                }
            }
        }
        None
    }

    fn play_words(&mut self, words: Vec<String>) {
        let mut i = self.current_score;
        let mut game_over = false;
        for word in words {
            let c = self.chars_pi[i];
            if let Some(char_said) = word_to_char(&word) {
                if char_said == c {
                    self.current_score += 1;
                    self.chars_input.push(CharCorrectness { c, correct: true });
                    self.update_score();
                    if self.current_score == MAX_SCORE {
                        // handle winning case so we don't go past max digits stored for PI
                        game_over = true;
                        break;
                    }
                } else {
                    self.chars_input.push(CharCorrectness {
                        c: char_said,
                        correct: false,
                    });
                    game_over = true;
                    self.update_score();
                }
            }
            // else: ignore unrecognized words
            i += 1;
        }
        if game_over {
            sleep(GAME_OVER_PAUSE);
            self.end_game();
            self.update_score(); // reset to waiting for user input
        }
    }

    // Reset game
    fn end_game(&mut self) {
        self.state = State::Waiting;
        self.current_score = 0;
        self.chars_input.clear();
    }

    fn update_score(&mut self) {
        if self.current_score > self.high_score {
            self.high_score = self.current_score;
        }
        match self.state {
            State::Playing => {
                if self.chars_input.len() == 0 {
                    self.tui.update_info(
                        self.high_score,
                        self.current_score,
                        "start reciting pi...",
                    )
                } else if self.current_score == MAX_SCORE {
                    self.tui.update_user_input(
                        self.high_score,
                        self.current_score,
                        &self.chars_input,
                    );
                    sleep(GAME_WIN_PAUSE);
                    self.tui.update_info(
                        self.high_score,
                        self.current_score,
                        "Congratulations, you won!",
                    )
                } else {
                    self.tui.update_user_input(
                        self.high_score,
                        self.current_score,
                        &self.chars_input,
                    )
                }
            }
            State::Waiting => self.tui.update_info(
                self.high_score,
                self.current_score,
                "say \"start\" to begin",
            ),
        }
    }
}

// Returns whether the current word is correct in the expected sequence of PI
fn word_to_char(word: &str) -> Option<char> {
    match word {
        "oh" => Some('0'),
        "zero" => Some('0'),
        "one" => Some('1'),
        "two" => Some('2'),
        "three" => Some('3'),
        "four" => Some('4'),
        "five" => Some('5'),
        "six" => Some('6'),
        "seven" => Some('7'),
        "eight" => Some('8'),
        "nine" => Some('9'),
        "point" => Some('.'),
        _ => return None,
    }
}
