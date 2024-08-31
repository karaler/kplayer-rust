#[derive(Default, Eq, PartialEq, Debug)]
pub enum KPCodecStatus {
    #[default]
    None,
    Created,
    Opened,
    Started,
    Paused,
    Ended,
    Stopped,
}