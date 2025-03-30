use anyhow::{Context, Result};
use csv::{ReaderBuilder, Trim};
use std::fs::File;
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
        let file = File::open(&path).with_context(|| format!("Failed to read from {}", path))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_successfull_parse() {
        let (tx, rx) = mpsc::channel();
        let resource = CsvResource::new(tx);
        let result = resource.parse("fixtures/sample.csv");

        assert!(result.is_ok());

        assert_eq!(rx.recv().unwrap(), Transaction::deposit(1, 1, 100.0));
        assert_eq!(rx.recv().unwrap(), Transaction::withdrawal(1, 2, 20.0));
        assert_eq!(rx.recv().unwrap(), Transaction::withdrawal(1, 3, 30.0));
        assert_eq!(rx.recv().unwrap(), Transaction::dispute(1, 2));
        assert_eq!(rx.recv().unwrap(), Transaction::resolve(1, 2));
        assert_eq!(rx.recv().unwrap(), Transaction::dispute(1, 3));
        assert_eq!(rx.recv().unwrap(), Transaction::chargeback(1, 3));
        assert!(rx.recv().is_err());
    }

    #[test]
    fn test_parse_skips_errors() {
        let (tx, rx) = mpsc::channel();
        let resource = CsvResource::new(tx);
        let result = resource.parse("fixtures/with_error.csv");

        assert!(result.is_ok());

        assert_eq!(rx.recv().unwrap(), Transaction::deposit(1, 1, 100.0));
        assert_eq!(rx.recv().unwrap(), Transaction::withdrawal(1, 2, 20.0));
        assert!(rx.recv().is_err());
    }

    #[test]
    fn test_missing_file() {
        let (tx, rx) = mpsc::channel();
        let resource = CsvResource::new(tx);
        let result = resource.parse("fixtures/__not_existing_file__.csv");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("read from"));
        assert!(rx.recv().is_err());
    }
}
