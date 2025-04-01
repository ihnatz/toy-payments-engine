mod account;
mod engine;
mod event;
mod ledger;
mod processor;
mod resources;
mod worker;

use engine::Engine;
use event::Event;

use std::process;
use std::sync::mpsc;

const QUEUE_CAPACITY: usize = 1;
const WORKERS_COUNT: usize = 4;

#[derive(Debug, PartialEq)]
enum StreamEvent {
    Value(Event),
    EndOfStream,
}

fn main() {
    let (tx, rx) = mpsc::channel::<StreamEvent>();

    let mut engine = Engine::new();
    let handles = engine.start_workers();

    if let Err(e) = resources::CsvResource::new(tx).parse("fixtures/many_clients.csv") {
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

    println!("{:?}", engine.core.chart);
}
