use rust_decimal::dec;
use rust_decimal::Decimal;
use serde::ser::SerializeStruct;
use serde::ser::{Serialize, Serializer};

#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    id: u16,
    available: Decimal,
    held: Decimal,
    locked: bool,
}

impl Serialize for Account {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Account", 5)?;

        state.serialize_field("client", &self.id)?;
        state.serialize_field("available", &self.available.to_string())?;
        state.serialize_field("held", &self.held.to_string())?;
        state.serialize_field("total", &self.total().to_string())?;
        state.serialize_field("locked", &self.locked.to_string())?;

        state.end()
    }
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

    fn lock(&mut self) {
        self.locked = true;
    }
}
