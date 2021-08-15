use std::collections::HashSet;
use std::fs;
use std::io::{self, BufRead};

// TODO: make `words` private
// Stores lowercased and trimmed words.
#[derive(Default)]
pub struct WordStorage {
    pub words: HashSet<String>,
}

impl WordStorage {
    pub fn load_file(&mut self, file_path: &str) -> Result<(), io::Error> {
        let f = fs::File::open(file_path)?;
        let reader = io::BufReader::new(f);
        let mut n = 0;
        for line in reader.lines() {
            let l = line.unwrap();
            match line_to_word(&l) {
                Some(w) => {
                    self.words.insert(w);
                    n += 1;
                }
                None => {
                    println!("Failed to parse line: \"{}\"", l);
                }
            }
        }
        println!("`load_file` finished. Total words parsed: {}.", n);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }
}

fn line_to_word(l: &str) -> Option<String> {
    let w = l.trim().to_lowercase();
    if w.len() == 0 || w.contains(char::is_whitespace) {
        None
    } else {
        Some(w)
    }
}
