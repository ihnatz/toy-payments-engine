mod transaction;

use transaction::Transaction;

use csv::{ReaderBuilder, Trim};
use std::fs::File;
use std::process;
use std::sync::mpsc;
use std::thread;

use anyhow::{Context, Result};

struct CsvResource {
    path: String,
    sender: mpsc::Sender<Transaction>,
}

impl CsvResource {
    fn parse(self) -> Result<()> {
        let file =
            File::open(&self.path).with_context(|| format!("Failed to read from {}", self.path))?;
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

fn main() {
    let (tx, rx) = mpsc::channel::<Transaction>();
    thread::spawn(move || {
        let resource = CsvResource {
            path: String::from("fixtures/sample.csv"),
            sender: tx,
        };

        if let Err(e) = resource.parse() {
            eprintln!("Error {:?}", e);
            process::exit(1);
        }
    });

    for received in rx {
        println!("Got: {:?}", received);
    }
}
