use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f64>,
}

impl Transaction {
    fn new(tx_type: TransactionType, client: u16, tx: u32, amount: Option<f64>) -> Self {
        Transaction {
            tx_type,
            client,
            tx,
            amount,
        }
    }

    pub fn deposit(client: u16, tx: u32, amount: f64) -> Self {
        Self::new(TransactionType::Deposit, client, tx, Some(amount))
    }

    pub fn withdrawal(client: u16, tx: u32, amount: f64) -> Self {
        Self::new(TransactionType::Withdrawal, client, tx, Some(amount))
    }

    pub fn dispute(client: u16, tx: u32) -> Self {
        Self::new(TransactionType::Dispute, client, tx, None)
    }

    pub fn resolve(client: u16, tx: u32) -> Self {
        Self::new(TransactionType::Resolve, client, tx, None)
    }

    pub fn chargeback(client: u16, tx: u32) -> Self {
        Self::new(TransactionType::Chargeback, client, tx, None)
    }
}
