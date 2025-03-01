use std::ops::{Index, IndexMut};

const PI: &str = "3.14159265358979323846264338327950288419716939937510582097494459230781640628620899862803482534211706";
const MAX_SCORE: usize = PI.len();

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
    Ending,
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
    pi_chars: PiChars,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Self::default();
        for (i, c) in PI.chars().enumerate() {
            game.pi_chars[i] = c;
        }
        game
    }

    pub fn play(&mut self, mut words: Vec<String>) {
        // If ending, wait for external timer to call end_game
        if self.state == State::Ending {
            return;
        }
        // Check for start of game
        if self.state == State::Waiting {
            if let Some(more_words) = self.find_start(&words) {
                words = more_words;
                println!("starting game: words remaining: {:?}", words);
            }
        }
        // Play game
        if self.state == State::Playing {
            self.play_words(words);
        }

        // Print:
        // - print current progress is Playing
        // - print high score if waiting
    }

    // Searches for the word "start" and returns remaining words that could be playable if start was found
    fn find_start(&mut self, words: &Vec<String>) -> Option<Vec<String>> {
        for (i, word) in words.iter().enumerate() {
            if word == "start" {
                self.state = State::Playing;
                if i < words.len() {
                    return Some(words[i + 1..].to_vec());
                }
            }
        }
        None
    }

    fn play_words(&mut self, words: Vec<String>) {
        let mut i = self.current_score;
        for word in words {
            let c = self.pi_chars[i];
            if let Some(char_said) = word_to_char(&word) {
                if char_said == c {
                    println!("Correct!: {c}");
                    self.current_score += 1;
                    if self.current_score == MAX_SCORE {
                        // handing winning case so we don't go past max digits stored for PI
                    }
                } else {
                    println!("Incorrect char: '{char_said}'");
                }
            } else {
                println!("Unrecognized word : '{word}'");
            }
            i += 1;
        }
    }

    // Reset game
    fn end_game(&mut self) {
        self.state = State::Waiting;
        self.current_score = 0;
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
