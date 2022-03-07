use std::{
    fs::File,
    io::{BufRead, BufReader},
    collections::HashMap
};

use text_io::read;
use colored::*;

struct Game {
    attempts: Vec<String>,
    word: String,
    words_to_guess: Vec<String>,
    available_words: Vec<String>,
    word_index: usize,
    state: HashMap<char, LetterStatus>
}

enum LetterStatus {
    Correct,
    Wrong,
    WrongPosition,
}

enum WordErrorStatus {
    WordTooLong,
    WordDoesNotExist,
    GameOver,
}

impl ToString for WordErrorStatus {
    fn to_string(&self) -> String {
        match *self {
            WordErrorStatus::GameOver => "Game over!",
            WordErrorStatus::WordDoesNotExist => "This word does not exist!",
            WordErrorStatus::WordTooLong => "Only 5 letter words are accepted"
        }.to_string()
    }
}

fn fetch_words_from_file(filename: &str) -> Vec<String> {
    let mut words = Vec::new();

    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let vector = line.split_whitespace().collect::<Vec<&str>>();

        words.push(vector[0].to_string());
    }

    words
}

impl Game {
    fn new() -> Self {
        let mut game = Game {
            attempts: Vec::new(),
            word: String::new(),
            words_to_guess: fetch_words_from_file("public/words_to_guess.txt"),
            available_words: fetch_words_from_file("public/available_words.txt"),
            word_index: 0,
            state: HashMap::new()
        };

        game.start();
        game
    }

    fn start(&mut self) {
        self.word = self.pick_word();
        self.word_index += 1;
    }

    fn pick_word(&self) -> String {
        let word = self.words_to_guess[self.word_index].clone();
        word
    }

    fn is_valid_word(&self, word: String) -> bool {
        let mut valid_words = self.available_words.clone();
        valid_words.append(&mut self.words_to_guess.clone());

        valid_words.contains(&word)
    }

    fn take_a_guess(&mut self, word: String) -> Result<(bool, Vec<(char, LetterStatus)>), WordErrorStatus> {
        if word.len() != 5 {
            return Err(WordErrorStatus::WordTooLong);
        }

        if self.attempts.len() == 6 {
            return Err(WordErrorStatus::GameOver);
        }

        if !self.is_valid_word(word.clone()) {
            return Err(WordErrorStatus::WordDoesNotExist);
        }
        
        self.attempts.push(word.clone());

        let chars: Vec<char> = word.chars().collect();
        let answer_chars: Vec<char> = self.word.clone().chars().collect();
        let mut changes: Vec<(char, LetterStatus)> = Vec::new();

        for i in 0..5 {
            if chars[i] == answer_chars[i] {
               self.state.insert(answer_chars[i], LetterStatus::Correct); 
               changes.push((chars[i], LetterStatus::Correct));
               continue;
            }

            if answer_chars.contains(&chars[i]) {
                self.state.insert(answer_chars[i], LetterStatus::WrongPosition); 
                changes.push((chars[i], LetterStatus::WrongPosition));
                continue;
            }


            self.state.insert(answer_chars[i], LetterStatus::Wrong);
            changes.push((chars[i], LetterStatus::Wrong));
        }

        if self.is_game_over() {
            return Ok((true, changes)); 
        }

        Ok((false, changes))
    }

    fn is_game_over(&self) -> bool {
        let mut result = true;

        for key in self.state.keys() {
            result = result && matches!(self.state.get(key).unwrap(), LetterStatus::Correct);
        }

        result
    }

    fn main_game_loop(&mut self, client: &dyn GameClient) {
        let mut running = true;

        while running {
            let guess = client.get_new_guess();
            match self.take_a_guess(guess.to_lowercase()) {
                Err(message) => {
                    client.display_error_message(&message);

                    if matches!(message, WordErrorStatus::GameOver) {
                        running = false;
                    }
                },
                Ok(round) => {
                    // The second element in the tuple is all the changes in the hash that happend in
                    // the given round
                    client.display_round_changelog(round.1);

                    // the first element on the tuple is wheter the word was guessed successfully
                    if round.0 {
                        running = false;
                    }
                }
            }

        }
    }
}

trait GameClient {
    fn get_new_guess(&self) -> String;
    fn display_round_changelog(&self, changelog: Vec<(char, LetterStatus)>);
    fn display_error_message(&self, error: &WordErrorStatus);
}

struct TerminalGameClient;

impl GameClient for TerminalGameClient {
    fn get_new_guess(&self) -> String {
        println!("Type your guess");
        read!("{}\n")
    }

    fn display_round_changelog(&self, changelog: Vec<(char, LetterStatus)>) {
        for change in changelog {
            let change_str = format!("{}", change.0);

            if matches!(change.1, LetterStatus::Correct) {
                print!(" {} ", change_str.blue());
            }

            if matches!(change.1, LetterStatus::WrongPosition) {
                print!(" {} ", change_str.yellow());
            }

            if matches!(change.1, LetterStatus::Wrong) {
                print!(" {} ", change_str.red());
            }
        }
        println!();
    }

    fn display_error_message(&self, error: &WordErrorStatus) {
        println!("{}", error.to_string());
    }
}

fn main() {
    let mut game = Game::new();
    let client = TerminalGameClient {};
    game.main_game_loop(&client);
}

