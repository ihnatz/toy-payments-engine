mod event;
mod ledger;
mod resources;

use event::Event;
use ledger::Ledger;

use std::sync::mpsc;
use std::{process, thread};

#[derive(Debug, PartialEq)]
enum StreamEvent {
    Value(Event),
    EndOfStream,
}

fn main() {
    let ledger = Ledger::default();
    let (tx, rx) = mpsc::channel::<StreamEvent>();
    thread::spawn(move || {
        let resource = resources::CsvResource::new(tx);

        if let Err(e) = resource.parse("fixtures/sample.csv") {
            eprintln!("Error {:?}", e);
            process::exit(1);
        }
    });

    for event in rx {
        match event {
            StreamEvent::Value(event) => {
                match ledger.add_event(event) {
                    Ok(_) => (),
                    Err(message) => eprintln!("{}", message),
                };
            }
            StreamEvent::EndOfStream => {
                break;
            }
        }
    }

    println!("{:?}", ledger.count());
}
