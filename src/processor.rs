use crate::{
    account::Account,
    engine::EngineCore,
    event::{Event, EventType},
};

pub struct EventProcessor {
    engine_core: EngineCore,
}

impl EventProcessor {
    pub fn new(engine_core: EngineCore) -> Self {
        EventProcessor { engine_core }
    }

    pub fn process(&self, event: &Event) {
        match event.tx_type {
            EventType::Deposit => self.handle_deposit(event),
            EventType::Withdrawal => self.handle_withdrawal(event),
            EventType::Dispute => self.handle_dispute(event),
            EventType::Resolve => self.handle_resolve(event),
            EventType::Chargeback => self.handle_chargeback(event),
        }
    }

    pub fn is_shutdown(&self) -> bool {
        self.engine_core
            .shutdown
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    fn handle_deposit(&self, event: &Event) {
        self.with_account(event.client, |account| {
            account.available += event.amount.unwrap();
        });
    }

    fn handle_withdrawal(&self, event: &Event) {
        self.with_account(event.client, |account| {
            if account.available >= event.amount.unwrap() {
                account.available -= event.amount.unwrap();
            }
        });
    }

    fn handle_dispute(&self, event: &Event) {
        if let Some(amount) = self.get_transaction_amount(event) {
            self.with_account(event.client, |account| {
                account.held += amount;
            });
        }
    }

    fn handle_resolve(&self, event: &Event) {
        if let Some(amount) = self.get_transaction_amount(event) {
            self.with_account(event.client, |account| {
                account.held -= amount;
                account.available += amount;
            });
        }
    }

    fn handle_chargeback(&self, event: &Event) {
        if let Some(amount) = self.get_transaction_amount(event) {
            self.with_account(event.client, |account| {
                account.held -= amount;
                account.locked = true;
            });
        }
    }

    fn with_account<F>(&self, client_id: u16, action: F)
    where
        F: FnOnce(&mut Account),
    {
        let mut account = self
            .engine_core
            .chart
            .entry(client_id)
            .or_insert_with(|| Account::new(client_id));
        action(&mut account);
    }

    fn get_transaction_amount(&self, event: &Event) -> Option<f64> {
        self.engine_core
            .ledger
            .fetch_transaction(event.tx, event.client)
            .and_then(|tx| tx.amount)
    }
}
