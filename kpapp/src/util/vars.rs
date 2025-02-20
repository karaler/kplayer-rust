#[derive(Eq, PartialEq, Clone, Debug)]
pub enum KPAppStatus {
    None,
    Initialized,
    Starting,
    Ended,
    Closed,
}