use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CustomVoice {
    pub category_id: u64,
    pub voice_channel_id: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Starboard {
    pub forward_channel_id: u64,
    pub channels_whitelist: Vec<u64>,
    pub reaction_threshold: usize,
}
