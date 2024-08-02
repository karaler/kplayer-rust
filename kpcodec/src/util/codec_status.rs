#[derive(Default, Eq, PartialEq, Debug)]
pub enum KPCodecStatus {
    #[default]
    Created,
    Opened,
    Started,
    Paused,
    Ended,
    Stopped,
}