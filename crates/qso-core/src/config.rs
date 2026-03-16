use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub callsign: String,
    pub maidenhead: String,
    pub qrz_upload: bool,
    pub lotw_upload: bool,
    pub eqsl_upload: bool,
    surreal_db: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            callsign: String::new(),
            maidenhead: String::new(),
            qrz_upload: false,
            lotw_upload: false,
            eqsl_upload: false,
            surreal_db: String::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let config: Config = serde_json::from_reader(&mut file)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let mut file = File::create(path)?;
        serde_json::to_writer_pretty(&mut file, &self)?;
        Ok(())
    }
}