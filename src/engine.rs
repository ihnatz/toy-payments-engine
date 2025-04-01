use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam::queue::ArrayQueue;
use dashmap::DashMap;

use crate::{account::Account, event::Event, ledger::Ledger, worker::Worker, WORKERS_COUNT};

#[derive(Clone)]
pub struct EngineCore {
    pub ledger: Ledger,
    pub chart: Arc<DashMap<u16, Account>>,
    pub shutdown: Arc<AtomicBool>,
}

pub struct Engine {
    pub core: EngineCore,
    pub queues: Vec<Arc<ArrayQueue<Event>>>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            core: EngineCore {
                ledger: Ledger::default(),
                chart: Arc::new(DashMap::new()),
                shutdown: Arc::new(AtomicBool::new(false)),
            },
            queues: Vec::with_capacity(WORKERS_COUNT),
        }
    }

    pub fn start_workers(&mut self) -> Vec<JoinHandle<()>> {
        let mut workers = Vec::with_capacity(WORKERS_COUNT);
        for _ in 0..WORKERS_COUNT {
            let worker = Worker::new(self.core.clone());
            self.queues.push(worker.queue.clone());
            workers.push(worker);
        }
        workers.into_iter().map(|worker| worker.start()).collect()
    }

    pub fn submit_event(&self, event: Event) -> Result<(), &str> {
        match self.core.ledger.add_event(event.clone()) {
            Ok(_) => (),
            Err(message) => eprintln!("{}", message),
        };

        let worker_idx = (event.client as usize) % self.queues.len();
        self.queues[worker_idx]
            .push(event)
            .map_err(|_| "Queue is full")
    }

    pub fn shutdown(&self) {
        self.core.shutdown.store(true, Ordering::Relaxed);
        self.wait_for_workers();
    }

    fn wait_for_workers(&self) {
        while self.queues.iter().any(|q| !q.is_empty()) {
            thread::sleep(Duration::from_millis(10));
        }
    }
}
