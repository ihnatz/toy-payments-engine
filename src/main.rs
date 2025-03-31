mod ledger;
mod resources;
mod event;

use event::Event;

use std::process;
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel::<Event>();
    thread::spawn(move || {
        let resource = resources::CsvResource::new(tx);

        if let Err(e) = resource.parse("fixtures/sample.csv") {
            eprintln!("Error {:?}", e);
            process::exit(1);
        }
    });

    for received in rx {
        println!("Got: {:?}", received);
    }
}
