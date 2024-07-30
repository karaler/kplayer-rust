#[derive(Default)]
pub enum KPCodecStatus {
    #[default]
    Created,
    Opened,
    Started,
    Paused,
    Stopped,
}