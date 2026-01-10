use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// User models
#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]

pub struct User {
    pub id: Option<i32>,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    pub fn is_editor(&self) -> bool {
        self.role == "editor" || self.role == "admin"
    }
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub password_hash: String,
    pub role: String,
}

// Node models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::nodes)]

pub struct Node {
    pub id: Option<i32>,
    pub name: String,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub secret_key: String,
    pub ip_address: Option<String>,
    pub status: String,
    #[serde(with = "ts_seconds_option")]
    pub last_heartbeat: Option<NaiveDateTime>,
    pub available_paths: Option<String>,
    #[serde(with = "ts_seconds")]
    pub created_at: NaiveDateTime,
    #[serde(with = "ts_seconds")]
    pub updated_at: NaiveDateTime,
    pub current_content_id: Option<i32>,
    pub playback_position_secs: Option<f32>,
    pub playback_duration_secs: Option<f32>,
    pub script_context: Option<String>,
}

mod ts_seconds {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &NaiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let dt = DateTime::<Utc>::from_naive_utc_and_offset(*date, Utc);
        serializer.serialize_str(&dt.to_rfc3339())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)?;
        Ok(dt.with_timezone(&Utc).naive_utc())
    }
}

mod ts_seconds_option {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &Option<NaiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(d) => {
                let dt = DateTime::<Utc>::from_naive_utc_and_offset(*d, Utc);
                serializer.serialize_str(&dt.to_rfc3339())
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => {
                let dt = DateTime::parse_from_rfc3339(&s).map_err(serde::de::Error::custom)?;
                Ok(Some(dt.with_timezone(&Utc).naive_utc()))
            }
            None => Ok(None),
        }
    }
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::nodes)]
pub struct NewNode {
    pub name: String,
    pub secret_key: String,
    pub ip_address: Option<String>,
    pub status: String,
}

// Schedule models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::schedules)]

pub struct Schedule {
    pub id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub schedule_type: String,
    pub priority: i32,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub dj_id: Option<i32>,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::schedules)]
pub struct NewSchedule {
    pub name: String,
    pub description: Option<String>,
    pub schedule_type: String,
    pub priority: i32,
    pub is_active: bool,
    pub dj_id: Option<i32>,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::schedules)]
pub struct UpdateSchedule {
    pub name: Option<String>,
    pub description: Option<Option<String>>, // Option<Option> to allow nulling out? Or just Option<String>?
    // Diesel treats Option::None as "do not update". To set to NULL, we need Option<Option<T>>?
    // Wait, description is Nullable<Text>.
    // If we want to set it to NULL, we likely need Option<Option<String>> or just pass explicit null.
    // simpler: Option<String>. But if None, it skips. How to unset description?
    // Usually via explicit null in JSON -> Some(None).
    // Let's stick to simple Option<String> for now and assume simple updates.
    // Actually, strictly speaking `Option<String>` in AsChangeset means "if None, don't update".
    // If the field is nullable, we might want to set it to null.
    // For now, let's just mirror NewSchedule but with Options.
    pub schedule_type: Option<String>,
    pub priority: Option<i32>,
    pub is_active: Option<bool>,
    pub dj_id: Option<Option<i32>>,
}

// Schedule Block models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::schedule_blocks)]

pub struct ScheduleBlock {
    pub id: Option<i32>,
    pub schedule_id: i32,
    pub content_id: Option<i32>,
    pub day_of_week: Option<i32>,
    pub specific_date: Option<NaiveDate>,
    pub start_time: NaiveTime,
    pub duration_minutes: i32,
    pub script_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub dj_id: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub target: String,
    pub timestamp: String,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::schedule_blocks)]
pub struct NewScheduleBlock {
    pub schedule_id: i32,
    pub content_id: Option<i32>,
    pub day_of_week: Option<i32>,
    pub specific_date: Option<NaiveDate>,
    pub start_time: NaiveTime,
    pub duration_minutes: i32,
    pub script_id: Option<i32>,
    pub dj_id: Option<i32>,
}

// Content Item models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::content_items)]

pub struct ContentItem {
    pub id: Option<i32>,
    pub title: String,
    pub description: Option<String>,
    pub content_type: String,
    pub content_path: String,
    pub adapter_id: Option<i32>,
    pub duration_minutes: Option<i32>,
    pub tags: Option<String>,
    pub node_accessibility: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub transformer_scripts: Option<String>,
    pub is_dj_accessible: bool, // New field, default false in DB
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::content_items)]
pub struct NewContentItem {
    pub title: String,
    pub description: Option<String>,
    pub content_type: String,
    pub content_path: String,
    pub adapter_id: Option<i32>,
    pub duration_minutes: Option<i32>,
    pub tags: Option<String>,
    pub node_accessibility: Option<String>,
    pub transformer_scripts: Option<String>,
    pub is_dj_accessible: bool,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::content_items)]
pub struct UpdateContentItem {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub content_type: Option<String>,
    pub content_path: Option<String>,
    pub adapter_id: Option<Option<i32>>,
    pub duration_minutes: Option<Option<i32>>,
    pub tags: Option<Option<String>>,
    pub node_accessibility: Option<Option<String>>,
    pub transformer_scripts: Option<Option<String>>,
    pub is_dj_accessible: Option<bool>,
}

// AI Provider models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::ai_providers)]
pub struct AiProvider {
    pub id: Option<i32>,
    pub name: String,
    pub provider_type: String,
    pub api_key: Option<String>,
    pub endpoint_url: Option<String>,
    pub model_name: Option<String>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub provider_category: String,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::ai_providers)]
pub struct NewAiProvider {
    pub name: String,
    pub provider_type: String,
    pub api_key: Option<String>,
    pub endpoint_url: Option<String>,
    pub model_name: Option<String>,
    pub is_active: bool,
    pub provider_category: String,
}

// DJ Profile models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::dj_profiles)]
pub struct DjProfile {
    pub id: Option<i32>,
    pub name: String,
    pub personality_prompt: String,
    pub voice_config_json: String,
    pub context_depth: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub voice_provider_id: Option<i32>,
    pub llm_provider_id: Option<i32>,
    pub context_script_ids: Option<String>,
    pub talkativeness: f32,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::dj_profiles)]
pub struct NewDjProfile {
    pub name: String,
    pub personality_prompt: String,
    pub voice_config_json: String,
    pub context_depth: i32,
    pub voice_provider_id: Option<i32>,
    pub llm_provider_id: Option<i32>,
    pub context_script_ids: Option<String>,
    pub talkativeness: f32,
}

// DJ Memory models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::dj_memories)]
pub struct DjMemory {
    pub id: Option<i32>,
    pub dj_id: i32,
    pub memory_type: String,
    pub content: String,
    pub importance_score: i32,
    pub happened_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::dj_memories)]
pub struct NewDjMemory {
    pub dj_id: i32,
    pub memory_type: String,
    pub content: String,
    pub importance_score: i32,
    pub happened_at: NaiveDateTime,
}

#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = crate::schema::dj_memories)]
pub struct UpdateDjMemory {
    pub content: Option<String>,
    pub importance_score: Option<i32>,
    pub memory_type: Option<String>,
}

// Script models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::scripts)]

pub struct Script {
    pub id: Option<i32>,
    pub name: String,
    pub description: Option<String>,
    pub script_type: String,
    pub script_content: String,
    pub parameters_schema: Option<String>,
    pub is_builtin: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::scripts)]
pub struct NewScript {
    pub name: String,
    pub description: Option<String>,
    pub script_type: String,
    pub script_content: String,
    pub parameters_schema: Option<String>,
    pub is_builtin: bool,
}

// Node Schedule Assignment models
#[allow(dead_code)]
#[derive(Debug, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::node_schedules)]

pub struct NodeSchedule {
    pub node_id: i32,
    pub schedule_id: i32,
    pub priority: Option<i32>,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::node_schedules)]
pub struct NewNodeSchedule {
    pub node_id: i32,
    pub schedule_id: i32,
    pub priority: Option<i32>,
}

// Permission models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::permissions)]

pub struct Permission {
    pub id: Option<i32>,
    pub user_id: i32,
    pub resource_type: String,
    pub resource_id: i32,
    pub permission_level: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::permissions)]
pub struct NewPermission {
    pub user_id: i32,
    pub resource_type: String,
    pub resource_id: i32,
    pub permission_level: String,
}

// Global Settings models
#[allow(dead_code)]
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::global_settings)]

pub struct GlobalSetting {
    pub id: Option<i32>,
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::global_settings)]
pub struct NewGlobalSetting {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
}
