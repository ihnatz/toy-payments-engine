use csv::{ReaderBuilder, Trim};
use std::fs::File;
use anyhow::{Context, Result};
use std::sync::mpsc;

use crate::transaction::Transaction;


pub struct CsvResource {
    sender: mpsc::Sender<Transaction>,
}

impl CsvResource {
    pub fn new(sender: mpsc::Sender<Transaction>) -> Self {
        CsvResource { sender }
    }

    pub fn parse(self, path: &str) -> Result<()> {
        let file =
            File::open(&path).with_context(|| format!("Failed to read from {}", path))?;
        let mut rdr = ReaderBuilder::new().trim(Trim::All).from_reader(file);
        for result in rdr.deserialize() {
            match result {
                Ok(record) => self.sender.send(record)?,
                Err(e) => {
                    eprintln!("Can't parse: {:?}", e);
                    continue;
                }
            }
        }
        Ok(())
    }
}
