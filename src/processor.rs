use rust_decimal::Decimal;

use crate::{
    account::Account,
    engine::EngineCore,
    event::{Event, EventType},
    ledger::Transaction,
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
            if let Some(amount) = self.get_transaction_amount(event) {
                account.available += amount;
            }
        });
    }

    fn handle_withdrawal(&self, event: &Event) {
        self.with_account(event.client, |account| {
            if let Some(amount) = self.get_transaction_amount(event) {
                if account.available >= amount {
                    account.available -= amount;
                }
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

    fn get_transaction_amount(&self, event: &Event) -> Option<Decimal> {
        let tx = self
            .engine_core
            .ledger
            .fetch_transaction(event.tx, event.client);

        match tx {
            Some(Transaction::Deposit { amount, .. })
            | Some(Transaction::Withdrawal { amount, .. }) => Some(amount),
            None => None,
        }
    }
}
