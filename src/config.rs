use serde_derive::{Deserialize, Serialize};
use std::fs;
use toml;
use uuid::Uuid;

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub id: Option<Uuid>,
    pub name: String,
    pub geohash: String,
    pub postgres: Postgres,
    pub sdr: Sdr,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Postgres {
    pub connection_url: String,
}
#[derive(Deserialize, Serialize, Clone)]
pub struct Sdr {
    pub center_frequency: f32,
    pub sample_rate: f32,
    pub gain: f32,
    pub frequencies: Vec<f32>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
