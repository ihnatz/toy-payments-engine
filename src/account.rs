use rust_decimal::dec;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    pub id: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Account {
    pub fn new(id: u16) -> Self {
        Account {
            id,
            available: dec!(0.0),
            held: dec!(0.0),
            total: dec!(0.0),
            locked: false,
        }
    }
}
