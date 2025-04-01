mod account;
mod engine;
mod event;
mod ledger;
mod processor;
mod resources;
mod worker;

use csv::Writer;
use engine::Engine;
use event::Event;

use std::sync::mpsc;
use std::{env, process};

const QUEUE_CAPACITY: usize = 100;
const WORKERS_COUNT: usize = 4;

#[derive(Debug, PartialEq)]
enum StreamEvent {
    Value(Event),
    EndOfStream,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        eprintln!("No arguments provided");
        process::exit(1);
    }

    let (tx, rx) = mpsc::channel::<StreamEvent>();

    let mut engine = Engine::new();
    let handles = engine.start_workers();

    if let Err(e) = resources::CsvResource::new(tx).parse(&args[1]) {
        eprintln!("Error {:?}", e);
        process::exit(1);
    }

    for event in rx {
        match event {
            StreamEvent::EndOfStream => break,
            StreamEvent::Value(event) => {
                let _ = engine.submit_event(event.clone());
            }
        }
    }

    engine.shutdown();

    for handle in handles {
        handle.join().unwrap();
    }

    let mut wtr = Writer::from_writer(std::io::stdout());
    for account in engine.core.chart.iter() {
        wtr.serialize(account.clone()).unwrap();
    }
    wtr.flush().unwrap();
}
