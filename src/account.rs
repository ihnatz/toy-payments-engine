#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    pub id: u16,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

impl Default for Account {
    fn default() -> Self {
        Account {
            id: 0,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }
}
