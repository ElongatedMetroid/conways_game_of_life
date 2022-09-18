use std::fs;

use serde_derive::Deserialize;

#[derive(Deserialize, PartialEq, Debug)]
pub struct Config {
    file_name: Option<String>,

    pub dead_cell: Option<char>,
    pub live_cell: Option<char>,

    pub grid_rows: Option<usize>,
    pub grid_cols: Option<usize>,

    pub speed: Option<u64>,
    pub seed: Option<usize>,
}

impl Config {
    pub fn new(file: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let mut config: Config = toml::from_str(&fs::read_to_string(file)?)?;
        
        config.file_name = Some(String::from(file));
        
        Ok(config)
    }
    pub fn refresh(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let mut new: Config = toml::from_str(&fs::read_to_string(self.file_name.as_ref().unwrap())?)?;
        new.file_name = self.file_name.clone();

        if new == *self {
            Ok(false)
        } else {
            *self = new;
            Ok(true)
        }
    }
}