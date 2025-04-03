use std::fs;

use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Kondo {
    pub default_deadline: String,
    pub editor: String,
}

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub kondo: Kondo,
}

const DEFAULT_CONFIG: &str = r#"
[kondo]
# Default deadline in days from now.
default_deadline=7
editor=vim

"#;

impl Configuration {
    pub fn new() -> Self {
        let home_dir = dirs::home_dir().unwrap();
        let mut config_path = home_dir.join(".config/kondo/kondo.toml");

        if !config_path.exists() {
            fs::create_dir_all(config_path.parent().unwrap())
                .expect("Failed to create config directory");
            fs::write(&config_path, DEFAULT_CONFIG.trim()).expect("Failed to write default config");
        }

        let settings = Config::builder()
            .add_source(File::from(config_path.clone()).required(true))
            .add_source(Environment::with_prefix("KONDO"))
            .build()
            .unwrap();

        match settings.try_deserialize::<Configuration>() {
            Ok(c) => c,
            Err(e) => panic!("Couldn't deserialize {}", e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Configuration;

    #[test]
    fn test_config() {
        let cfg = Configuration::new();
        assert_eq!(cfg.kondo.default_deadline, "7");
    }
}
