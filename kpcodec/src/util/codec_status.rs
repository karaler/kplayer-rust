#[derive(Default, Eq, PartialEq, Debug)]
pub enum KPCodecStatus {
    #[default]
    None,
    Created,
    Opened,
    Started,
    Paused,
    Flushed,
    Stopped,
    Ended,
}

#[derive(Default, Eq, PartialEq, Debug)]
pub enum KPEncodeMode {
    #[default]
    File,
    Live,
}