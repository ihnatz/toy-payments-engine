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
                account.deposit(amount);
            }
        });
    }

    fn handle_withdrawal(&self, event: &Event) {
        self.with_account(event.client, |account| {
            if let Some(amount) = self.get_transaction_amount(event) {
                if !account.locked() && account.available() >= amount {
                    account.withdraw(amount);
                }
            }
        });
    }

    fn handle_dispute(&self, event: &Event) {
        if let Some(amount) = self.get_dispute_amount(event) {
            self.with_account(event.client, |account| {
                account.hold(amount);
            });
        }
    }

    fn handle_resolve(&self, event: &Event) {
        if let Some(amount) = self.get_dispute_amount(event) {
            self.with_account(event.client, |account| {
                account.resolve(amount);
            });
        }
    }

    fn handle_chargeback(&self, event: &Event) {
        if let Some(amount) = self.get_dispute_amount(event) {
            self.with_account(event.client, |account| {
                account.reject(amount);
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

    fn get_dispute_amount(&self, event: &Event) -> Option<Decimal> {
        let tx = self
            .engine_core
            .ledger
            .fetch_transaction(event.tx, event.client);

        match tx {
            Some(Transaction::Withdrawal { amount, .. }) => Some(amount),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::dec;

    #[test]
    fn test_adding_amount() {
        let engine_core = EngineCore::default();
        let events = vec![Event {
            tx_type: EventType::Deposit,
            client: 1,
            tx: 1,
            amount: Some(10.0),
        }];

        process_events(engine_core.clone(), events);

        assert_eq!(engine_core.chart.get(&1).unwrap().available(), dec!(10.0));
    }

    #[test]
    fn test_adding_same_amount_twice() {
        let engine_core = EngineCore::default();
        let events = vec![
            Event {
                tx_type: EventType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(10.0),
            },
            Event {
                tx_type: EventType::Deposit,
                client: 1,
                tx: 2,
                amount: Some(20.0),
            },
        ];

        process_events(engine_core.clone(), events);

        assert_eq!(engine_core.chart.get(&1).unwrap().available(), dec!(30.0));
    }

    #[test]
    fn test_withdrawal_amount() {
        let engine_core = EngineCore::default();
        let events = vec![Event {
            tx_type: EventType::Withdrawal,
            client: 1,
            tx: 1,
            amount: Some(10.0),
        }];

        process_events(engine_core.clone(), events);

        assert_eq!(engine_core.chart.get(&1).unwrap().available(), dec!(0));
    }

    #[test]
    fn test_withdrawal_amount_after_adding() {
        let engine_core = EngineCore::default();
        let events = vec![
            Event {
                tx_type: EventType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(20.0),
            },
            Event {
                tx_type: EventType::Withdrawal,
                client: 1,
                tx: 2,
                amount: Some(5.0),
            },
        ];

        process_events(engine_core.clone(), events);

        assert_eq!(engine_core.chart.get(&1).unwrap().available(), dec!(15.0));
    }

    #[test]
    fn test_twice_withdrawal_amount_after_adding() {
        let engine_core = EngineCore::default();
        let events = vec![
            Event {
                tx_type: EventType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(20.0),
            },
            Event {
                tx_type: EventType::Withdrawal,
                client: 1,
                tx: 2,
                amount: Some(5.0),
            },
            Event {
                tx_type: EventType::Withdrawal,
                client: 1,
                tx: 3,
                amount: Some(5.0),
            },
        ];

        process_events(engine_core.clone(), events);

        assert_eq!(engine_core.chart.get(&1).unwrap().available(), dec!(10.0));
    }

    #[test]
    fn test_dispute_on_deposit_is_ignored() {
        let engine_core = EngineCore::default();
        let events = vec![
            Event {
                tx_type: EventType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(30.0),
            },
            Event {
                tx_type: EventType::Withdrawal,
                client: 1,
                tx: 2,
                amount: Some(20.0),
            },
            Event {
                tx_type: EventType::Dispute,
                client: 1,
                tx: 1,
                amount: None,
            },
        ];

        process_events(engine_core.clone(), events);

        assert_eq!(engine_core.chart.get(&1).unwrap().available(), dec!(10.0));
    }

    #[test]
    fn test_dispute_on_withdrawal() {
        let engine_core = EngineCore::default();
        let events = vec![
            Event {
                tx_type: EventType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(30.0),
            },
            Event {
                tx_type: EventType::Withdrawal,
                client: 1,
                tx: 2,
                amount: Some(20.0),
            },
            Event {
                tx_type: EventType::Dispute,
                client: 1,
                tx: 2,
                amount: None,
            },
        ];

        process_events(engine_core.clone(), events);

        assert_eq!(engine_core.chart.get(&1).unwrap().available(), dec!(10.0));
        assert_eq!(engine_core.chart.get(&1).unwrap().held(), dec!(20.0));
        assert_eq!(engine_core.chart.get(&1).unwrap().total(), dec!(30.0));
    }

    #[test]
    fn test_dispute_on_withdrawal_with_resolve() {
        let engine_core = EngineCore::default();
        let events = vec![
            Event {
                tx_type: EventType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(30.0),
            },
            Event {
                tx_type: EventType::Withdrawal,
                client: 1,
                tx: 2,
                amount: Some(20.0),
            },
            Event {
                tx_type: EventType::Dispute,
                client: 1,
                tx: 2,
                amount: None,
            },
            Event {
                tx_type: EventType::Resolve,
                client: 1,
                tx: 2,
                amount: None,
            },
        ];

        process_events(engine_core.clone(), events);

        assert_eq!(engine_core.chart.get(&1).unwrap().available(), dec!(30.0));
        assert_eq!(engine_core.chart.get(&1).unwrap().held(), dec!(0.0));
        assert_eq!(engine_core.chart.get(&1).unwrap().total(), dec!(30.0));
    }

    #[test]
    fn test_dispute_on_withdrawal_with_chargeback() {
        let engine_core = EngineCore::default();
        let events = vec![
            Event {
                tx_type: EventType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(30.0),
            },
            Event {
                tx_type: EventType::Withdrawal,
                client: 1,
                tx: 2,
                amount: Some(20.0),
            },
            Event {
                tx_type: EventType::Dispute,
                client: 1,
                tx: 2,
                amount: None,
            },
            Event {
                tx_type: EventType::Chargeback,
                client: 1,
                tx: 2,
                amount: None,
            },
        ];

        process_events(engine_core.clone(), events);

        assert_eq!(engine_core.chart.get(&1).unwrap().available(), dec!(10.0));
        assert_eq!(engine_core.chart.get(&1).unwrap().held(), dec!(0.0));
        assert_eq!(engine_core.chart.get(&1).unwrap().total(), dec!(10.0));
        assert!(engine_core.chart.get(&1).unwrap().locked());
    }

    #[test]
    fn test_dispute_on_withdrawal_with_chargeback_disables_withdrawal() {
        let engine_core = EngineCore::default();
        let events = vec![
            Event {
                tx_type: EventType::Deposit,
                client: 1,
                tx: 1,
                amount: Some(30.0),
            },
            Event {
                tx_type: EventType::Withdrawal,
                client: 1,
                tx: 2,
                amount: Some(20.0),
            },
            Event {
                tx_type: EventType::Dispute,
                client: 1,
                tx: 2,
                amount: None,
            },
            Event {
                tx_type: EventType::Chargeback,
                client: 1,
                tx: 2,
                amount: None,
            },
            Event {
                tx_type: EventType::Withdrawal,
                client: 1,
                tx: 3,
                amount: Some(10.0),
            },
        ];

        process_events(engine_core.clone(), events);

        assert_eq!(engine_core.chart.get(&1).unwrap().available(), dec!(10.0));
        assert_eq!(engine_core.chart.get(&1).unwrap().held(), dec!(0.0));
        assert_eq!(engine_core.chart.get(&1).unwrap().total(), dec!(10.0));
        assert!(engine_core.chart.get(&1).unwrap().locked());
    }

    fn process_events(engine_core: EngineCore, events: Vec<Event>) {
        let processor = EventProcessor {
            engine_core: engine_core.clone(),
        };
        events
            .iter()
            .for_each(|event| engine_core.ledger.add_event(event.clone()).unwrap());

        for event in events {
            processor.process(&event);
        }
    }
}
