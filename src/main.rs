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

    fn take_a_guess(&mut self, word: String) -> Result<(bool, Vec<(char, LetterStatus)>), String> {
        if word.len() != 5 {
            return Err("The word must be 5 characters long".to_string());
        }

        if self.attempts.len() == 6 {
            return Err("Max attempts achieved!".to_string());
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
}

fn display_game_state(game: &Game, changelog: Vec<(char, LetterStatus)>) {
    for change in changelog {
        if matches!(change.1, LetterStatus::Correct) {
            let change_str = format!("{}", change.0);
            print!(" {} ", change_str.blue());
        }

        if matches!(change.1, LetterStatus::WrongPosition) {
            let change_str = format!("{}", change.0);
            print!(" {} ", change_str.yellow());
        }

        if matches!(change.1, LetterStatus::Wrong) {
            let change_str = format!("{}", change.0);
            print!(" {} ", change_str.red());
        }
    }

    println!();
}

fn main() {
    let mut game = Game::new();
    
    let mut running = true;

    while running {
        println!("Type your guess");
        let guess: String = read!("{}\n");

        match game.take_a_guess(guess.to_lowercase()) {
            Err(message) => {
                running = false;
            },
            Ok(is_over) => {
                if is_over.0 {
                    println!("The word was {}", guess);
                    return;
                }

                display_game_state(&game, is_over.1);
            }
        }
    }
}

