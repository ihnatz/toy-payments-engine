use rust_decimal::dec;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    id: u16,
    available: Decimal,
    held: Decimal,
    locked: bool,
}

impl Account {
    pub fn new(id: u16) -> Self {
        Account {
            id,
            available: dec!(0.0),
            held: dec!(0.0),
            locked: false,
        }
    }

    pub fn hold(&mut self, amount: Decimal) {
        self.held += amount;
    }

    pub fn resolve(&mut self, amount: Decimal) {
        self.held -= amount;
        self.available += amount;
    }

    pub fn reject(&mut self, amount: Decimal) {
        self.held -= amount;
        self.lock();
    }

    pub fn deposit(&mut self, amount: Decimal) {
        self.available += amount
    }

    pub fn withdraw(&mut self, amount: Decimal) {
        self.available -= amount
    }

    pub fn available(&self) -> Decimal {
        self.available
    }

    pub fn total(&self) -> Decimal {
        self.held + self.available
    }

    pub fn held(&self) -> Decimal {
        self.held
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    fn lock(&mut self) {
        self.locked = true;
    }
}
