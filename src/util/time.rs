use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct KPDuration {
    duration: Duration,
}

impl KPDuration {
    pub fn new(duration: Duration) -> KPDuration {
        KPDuration { duration }
    }
}

impl Display for KPDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let seconds = self.duration.as_secs();
        let hours = seconds / 3600;
        let minutes = (seconds / 60) % 60;
        let seconds = seconds % 60;

        let formatted = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        write!(f, "{}", formatted)
    }
}

impl Debug for KPDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "KPDuration({}s / {})",
            self.duration.as_secs().clone(),
            self.to_string()
        )
    }
}

impl KPDuration {
    pub fn current_mill_timestamp() -> u128 {
        let current_time = SystemTime::now();
        current_time.duration_since(UNIX_EPOCH).unwrap().as_millis()
    }
}
