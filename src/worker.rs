use crate::{
    account::Account,
    engine::EngineCore,
    event::{Event, EventType},
    QUEUE_CAPACITY,
};
use crossbeam::queue::ArrayQueue;
use std::{
    sync::{atomic::Ordering, Arc},
    thread,
    time::Duration,
};

pub struct Worker {
    pub queue: Arc<ArrayQueue<Event>>,
    engine_core: EngineCore,
}

impl Worker {
    pub fn new(engine_core: EngineCore) -> Self {
        Worker {
            engine_core,
            queue: Arc::new(ArrayQueue::new(QUEUE_CAPACITY)),
        }
    }

    pub fn start(self) -> thread::JoinHandle<()> {
        let queue = self.queue.clone();

        thread::spawn(move || {
            while !self.engine_core.shutdown.load(Ordering::Relaxed) || !self.queue.is_empty() {
                match queue.pop() {
                    Some(event) => self.process_event(&event),
                    None => thread::sleep(Duration::from_millis(10)),
                }
            }
        })
    }

    pub fn process_event(&self, event: &Event) {
        let mut client = self
            .engine_core
            .chart
            .entry(event.client)
            .or_insert_with(|| Account::new(event.client));

        match event.tx_type {
            EventType::Deposit => client.available += event.amount.unwrap(),
            EventType::Withdrawal => {
                if client.available > event.amount.unwrap() {
                    client.available -= event.amount.unwrap()
                }
            }
            EventType::Dispute => {
                let tx = self
                    .engine_core
                    .ledger
                    .fetch_transaction(event.tx, event.client);
                let amount = tx.unwrap().amount.unwrap();
                client.held += amount;
            }
            EventType::Resolve => {
                let tx = self
                    .engine_core
                    .ledger
                    .fetch_transaction(event.tx, event.client);
                let amount = tx.unwrap().amount.unwrap();
                client.held -= amount;
                client.available -= amount;
            }
            EventType::Chargeback => {
                let tx = self
                    .engine_core
                    .ledger
                    .fetch_transaction(event.tx, event.client);
                let amount = tx.unwrap().amount.unwrap();
                client.held -= amount;
                client.locked = true;
            }
        }
    }
}
