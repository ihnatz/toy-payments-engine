use crate::event::{Event, EventType};
use dashmap::DashMap;
use rust_decimal::{dec, prelude::FromPrimitive, Decimal};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum Transaction {
    Deposit { amount: Decimal, client: u16 },
    Withdrawal { amount: Decimal, client: u16 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum DisputeEvent {
    Dispute,
    Chargeback,
    Resolve,
}

#[derive(Default, Debug, Clone)]
pub struct Ledger {
    transactions: Arc<DashMap<u32, Transaction>>,
    disputes: Arc<DashMap<u32, Vec<DisputeEvent>>>,
}

impl Ledger {
    pub fn add_event(&self, event: Event) -> Result<(), String> {
        let id = event.tx;
        let client = event.client;

        match event.tx_type {
            EventType::Deposit | EventType::Withdrawal => match self.transactions.entry(id) {
                dashmap::mapref::entry::Entry::Occupied(_) => {
                    Err(format!("Transaction with ID {} already exists", id))
                }
                dashmap::mapref::entry::Entry::Vacant(entry) => {
                    if let Some(amount) = event.amount.and_then(Decimal::from_f64) {
                        if amount <= dec!(0) {
                            return Err(format!("Negative amount for ID {}", id));
                        }
                        let transaction = match event.tx_type {
                            EventType::Deposit => Transaction::Deposit { amount, client },
                            EventType::Withdrawal => Transaction::Withdrawal { amount, client },
                            _ => return Err(format!("Unknown transaction format for ID {}", id)),
                        };
                        entry.insert(transaction);
                        Ok(())
                    } else {
                        Err(format!("No amount for a transaction with ID {}", id))
                    }
                }
            },
            EventType::Chargeback | EventType::Dispute | EventType::Resolve => {
                if self.fetch_transaction(id, client).is_none() {
                    return Err(format!(
                        "Can't find a transaction with ID {} for client {}",
                        id, client
                    ));
                }

                let dispute_event = match event.tx_type {
                    EventType::Dispute => DisputeEvent::Dispute,
                    EventType::Chargeback => DisputeEvent::Chargeback,
                    EventType::Resolve => DisputeEvent::Resolve,
                    _ => return Err(format!("Unknown dispute format: {:?}", event.tx_type)),
                };

                let previous_event = self
                    .disputes
                    .get(&id)
                    .and_then(|f| f.clone().into_iter().last());

                if !self.is_valid_dispute_transition(previous_event.clone(), dispute_event.clone())
                {
                    return Err(String::from("Undefined transition"));
                }

                self.disputes
                    .entry(id)
                    .and_modify(|v| {
                        v.push(dispute_event.clone());
                    })
                    .or_insert_with(|| vec![dispute_event.clone()]);
                Ok(())
            }
        }
    }

    pub fn fetch_transaction(&self, id: u32, client: u16) -> Option<Transaction> {
        self.transactions.get(&id).and_then(|tx| {
            let transaction = tx.value();
            match transaction {
                Transaction::Deposit {
                    client: tx_client, ..
                }
                | Transaction::Withdrawal {
                    client: tx_client, ..
                } => {
                    if *tx_client == client {
                        Some(transaction.clone())
                    } else {
                        None
                    }
                }
            }
        })
    }

    fn is_valid_dispute_transition(
        &self,
        previous: Option<DisputeEvent>,
        next: DisputeEvent,
    ) -> bool {
        match (previous, next) {
            (Some(DisputeEvent::Dispute), DisputeEvent::Chargeback | DisputeEvent::Resolve) => true,
            (None, DisputeEvent::Dispute) => true,
            (_, _) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::dec;

    use super::*;

    impl Ledger {
        pub fn count(&self) -> (usize, usize) {
            (self.transactions.len(), self.disputes.len())
        }
    }

    impl Transaction {
        pub fn deposit(amount: Decimal, client: u16) -> Self {
            Transaction::Deposit { amount, client }
        }

        pub fn withdrawal(amount: Decimal, client: u16) -> Self {
            Transaction::Withdrawal { amount, client }
        }
    }

    #[test]
    fn test_initial_state() {
        let ledger = Ledger::default();
        let (transactions, disputes) = ledger.count();
        assert_eq!(transactions, 0);
        assert_eq!(disputes, 0);
    }

    #[test]
    fn test_add_deposit() {
        let ledger = Ledger::default();
        let event = Event::deposit(1, 1, 10.0);

        assert!(ledger.add_event(event.clone()).is_ok());

        let (transactions, disputes) = ledger.count();
        assert_eq!(transactions, 1);
        assert_eq!(disputes, 0);

        assert_eq!(
            ledger.transactions.get(&1).unwrap().value(),
            &Transaction::deposit(dec!(10.0), 1)
        );
    }

    #[test]
    fn test_add_withdrawal() {
        let ledger = Ledger::default();
        let event = Event::withdrawal(1, 1, 10.0);

        assert!(ledger.add_event(event.clone()).is_ok());

        let (transactions, disputes) = ledger.count();
        assert_eq!(transactions, 1);
        assert_eq!(disputes, 0);

        assert_eq!(
            ledger.transactions.get(&1).unwrap().value(),
            &Transaction::withdrawal(dec!(10.0), 1)
        );
    }

    #[test]
    fn test_add_negative_withdrawal() {
        let ledger = Ledger::default();
        let event = Event::withdrawal(1, 1, -10.0);

        assert!(ledger.add_event(event.clone()).is_err());
    }

    #[test]
    fn test_duplicate_transaction() {
        let ledger = Ledger::default();
        let event1 = Event::deposit(1, 1, 10.0);
        let event2 = Event::deposit(1, 1, 20.0);

        assert!(ledger.add_event(event1).is_ok());
        let result = ledger.add_event(event2);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Transaction with ID 1 already exists");
    }

    #[test]
    fn test_add_dispute() {
        let ledger = Ledger::default();
        let deposit = Event::deposit(1, 1, 10.0);
        let dispute = Event::dispute(1, 1);

        assert!(ledger.add_event(deposit).is_ok());
        assert!(ledger.add_event(dispute.clone()).is_ok());

        let (transactions, disputes) = ledger.count();
        assert_eq!(transactions, 1);
        assert_eq!(disputes, 1);

        let disputes_vec = ledger.disputes.get(&1).unwrap();
        assert_eq!(disputes_vec.value(), &vec![DisputeEvent::Dispute]);
    }

    #[test]
    fn test_fetch_transaction() {
        let ledger = Ledger::default();
        let deposit1 = Event::deposit(1, 1, 10.0);
        let deposit2 = Event::deposit(2, 2, 20.0);

        assert!(ledger.add_event(deposit1).is_ok());
        assert!(ledger.fetch_transaction(1, 1).is_some());
        assert!(ledger.fetch_transaction(1, 2).is_none());
        assert!(ledger.fetch_transaction(2, 1).is_none());

        assert!(ledger.add_event(deposit2).is_ok());
        assert!(ledger.fetch_transaction(2, 2).is_some());
        assert!(ledger.fetch_transaction(2, 1).is_none());
    }

    #[test]
    fn test_multiple_disputes_for_same_tx() {
        let ledger = Ledger::default();
        let deposit = Event::deposit(1, 1, 10.0);
        let dispute1 = Event::dispute(1, 1);
        let dispute2 = Event::dispute(1, 1);

        assert!(ledger.add_event(deposit).is_ok());
        assert!(ledger.add_event(dispute1.clone()).is_ok());
        assert!(ledger.add_event(dispute2.clone()).is_err()); // Dispute can't go to Dispute

        let disputes_vec = ledger.disputes.get(&1).unwrap();
        assert_eq!(disputes_vec.value(), &vec![DisputeEvent::Dispute]);
    }

    #[test]
    fn test_straight_to_resolve() {
        let ledger = Ledger::default();
        let deposit = Event::deposit(1, 1, 10.0);
        let resolve = Event::resolve(1, 1);

        assert!(ledger.add_event(deposit).is_ok());
        assert!(ledger.add_event(resolve).is_err()); // Dispute can't go to Dispute

        assert!(ledger.disputes.get(&1).is_none());
    }

    #[test]
    fn test_resolve_and_chargeback() {
        let ledger = Ledger::default();
        let deposit = Event::deposit(1, 1, 10.0);
        let dispute = Event::dispute(1, 1);
        let resolve = Event::resolve(1, 1);
        let chargeback = Event::chargeback(1, 1);

        assert!(ledger.add_event(deposit).is_ok());
        assert!(ledger.add_event(dispute).is_ok());
        assert!(ledger.add_event(resolve).is_ok());
        assert!(ledger.add_event(chargeback).is_err());

        let disputes_vec = ledger.disputes.get(&1).unwrap();
        assert_eq!(disputes_vec.value().len(), 2);
        assert_eq!(disputes_vec.value()[1], DisputeEvent::Resolve);
    }

    #[test]
    fn test_dispute_for_nonexistent_transaction() {
        let ledger = Ledger::default();
        let dispute = Event::dispute(1, 1);

        assert!(ledger.add_event(dispute).is_err());

        let (transactions, disputes) = ledger.count();
        assert_eq!(transactions, 0);
        assert_eq!(disputes, 0);
    }

    #[test]
    fn test_mixed_operations() {
        let ledger = Ledger::default();

        assert!(ledger.add_event(Event::deposit(1, 1, 10.0)).is_ok());
        assert!(ledger.add_event(Event::deposit(1, 2, 20.0)).is_ok());
        assert!(ledger.add_event(Event::deposit(2, 3, 30.0)).is_ok());

        assert!(ledger.add_event(Event::withdrawal(1, 4, 5.0)).is_ok());
        assert!(ledger.add_event(Event::withdrawal(2, 5, 10.0)).is_ok());

        assert!(ledger.add_event(Event::dispute(1, 1)).is_ok());
        assert!(ledger.add_event(Event::dispute(1, 2)).is_ok());
        assert!(ledger.add_event(Event::dispute(2, 3)).is_ok());

        assert!(ledger.add_event(Event::resolve(1, 1)).is_ok());
        assert!(ledger.add_event(Event::chargeback(1, 2)).is_ok());

        let (transactions, disputes) = ledger.count();
        assert_eq!(transactions, 5);
        assert_eq!(disputes, 3);

        assert_eq!(ledger.disputes.get(&1).unwrap().value().len(), 2);
        assert_eq!(ledger.disputes.get(&2).unwrap().value().len(), 2);
        assert_eq!(ledger.disputes.get(&3).unwrap().value().len(), 1);
    }
}
