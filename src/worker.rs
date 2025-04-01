use crate::{engine::EngineCore, event::Event, processor::EventProcessor, QUEUE_CAPACITY};
use crossbeam::queue::ArrayQueue;
use std::{sync::Arc, thread, time::Duration};

pub struct Worker {
    pub queue: Arc<ArrayQueue<Event>>,
    processor: EventProcessor,
}

impl Worker {
    pub fn new(engine_core: EngineCore) -> Self {
        Worker {
            processor: EventProcessor::new(engine_core),
            queue: Arc::new(ArrayQueue::new(QUEUE_CAPACITY)),
        }
    }

    pub fn start(self) -> thread::JoinHandle<()> {
        let queue = self.queue.clone();

        thread::spawn(move || {
            while !self.processor.is_shutdown() || !self.queue.is_empty() {
                match queue.pop() {
                    Some(event) => self.processor.process(&event),
                    None => thread::sleep(Duration::from_millis(10)),
                }
            }
        })
    }
}
