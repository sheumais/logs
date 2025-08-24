pub mod log_edit;
pub mod split_log;
pub mod esologs_format;
pub mod esologs_convert;

pub struct SharedUIProgressObject {
    pub state: SharedUIProgressState,
    pub value: u8,
}

pub enum SharedUIProgressState {
    None,
    Processing,
    Uploading,
}

impl SharedUIProgressObject {
    pub fn new() -> Self {
        return SharedUIProgressObject {
            state: SharedUIProgressState::None,
            value: 0
        }
    }
}