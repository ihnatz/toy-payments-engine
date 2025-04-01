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
