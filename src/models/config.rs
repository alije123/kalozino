use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CustomVoice {
    pub category_id: u64,
    pub voice_channel_id: u64,
}
