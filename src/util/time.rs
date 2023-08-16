use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;

pub struct KPDuration {
    duration: Duration,
}

impl KPDuration {
    pub fn new(duration: Duration) -> KPDuration {
        KPDuration { duration }
    }
}

impl ToString for KPDuration {
    fn to_string(&self) -> String {
        let seconds = self.duration.as_secs();
        let hours = seconds / 3600;
        let minutes = (seconds / 60) % 60;
        let seconds = seconds % 60;

        let formatted = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        formatted
    }
}

impl Debug for KPDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "KPDuration({}s / {})", self.duration.as_secs().clone(), self.to_string())
    }
}
