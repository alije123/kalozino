use chrono::{DateTime, Utc};
use ormlite::{types::Json, Model};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Model, Clone, Debug, Serialize, Deserialize)]
#[ormlite(table = "players")]
pub struct Player {
    pub id: i64,
    pub balance: f64,
    pub timely_last_at: Option<DateTime<Utc>>,
    pub timely_last_value: Option<f64>,
    pub timely_end_at: Option<DateTime<Utc>>,
    pub last_steal_at: Option<DateTime<Utc>>,
}

#[derive(Model, Clone, Debug, Serialize, Deserialize)]
#[ormlite(table = "active_custom_voices")]
pub struct ActiveCustomVoice {
    pub id: i64,
    pub owner_id: i64,
}

#[derive(Model, Clone, Debug, Serialize, Deserialize)]
pub struct HistoryJournal {
    pub id: Uuid,
    pub user_id: i64,
    pub at: DateTime<Utc>,
    pub value: f64,
    pub changed_by: Option<i64>,
    pub reason: String,
}

#[derive(Model, Clone, Debug, Serialize, Deserialize)]
pub struct VoiceConfig {
    pub id: Uuid,
    pub user_id: i64,
    pub parameter: String,
    pub value: String,
}

#[derive(Model, Clone, Debug, Serialize, Deserialize)]
#[ormlite(table = "twinks")]
pub struct Twink {
    pub id: Uuid,
    pub user_id: i64,
    pub twink_id: i64,
}

#[derive(Model, Clone, Debug, Serialize, Deserialize)]
pub struct Shop {
    pub id: Uuid,
    pub name: String,
    pub price: f64,
    pub description: String,
    pub item_type: String,
    pub role_id: i64,
}

#[derive(Model, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[ormlite(primary_key)]
    pub key: String,
    pub server_id: i64,
    pub data: Json<Value>,
}

#[derive(Model, Clone, Debug, Serialize, Deserialize)]
#[ormlite(table = "starboard_messages")]
pub struct StarboardMessage {
    #[ormlite(primary_key)]
    pub message_id: i64,
    pub server_id: i64,
    pub forwarded_message_id: i64,
    pub last_reaction_count: i16,
}
