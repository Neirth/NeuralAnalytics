#[derive(Debug, PartialEq, Clone, Copy)] // Added Clone, Copy for convenience
pub enum BulbState {
    BulbOn,
    BulbOff,
}