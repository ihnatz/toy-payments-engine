use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Event {
    #[serde(rename = "type")]
    pub tx_type: EventType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
}

impl Event {
    #[allow(dead_code)]
    fn new(tx_type: EventType, client: u16, tx: u32, amount: Option<f64>) -> Self {
        Event {
            tx_type,
            client,
            tx,
            amount,
        }
    }

    #[allow(dead_code)]
    pub fn deposit(client: u16, tx: u32, amount: f64) -> Self {
        Self::new(EventType::Deposit, client, tx, Some(amount))
    }

    #[allow(dead_code)]
    pub fn withdrawal(client: u16, tx: u32, amount: f64) -> Self {
        Self::new(EventType::Withdrawal, client, tx, Some(amount))
    }

    #[allow(dead_code)]
    pub fn dispute(client: u16, tx: u32) -> Self {
        Self::new(EventType::Dispute, client, tx, None)
    }

    #[allow(dead_code)]
    pub fn resolve(client: u16, tx: u32) -> Self {
        Self::new(EventType::Resolve, client, tx, None)
    }

    #[allow(dead_code)]
    pub fn chargeback(client: u16, tx: u32) -> Self {
        Self::new(EventType::Chargeback, client, tx, None)
    }
}
