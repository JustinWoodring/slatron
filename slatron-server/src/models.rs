use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

// User models
#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: i32,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
    pub id: i32,
    pub name: String,
    #[serde(skip_serializing)]
    pub secret_key: String,
    pub ip_address: Option<String>,
    pub status: String,
    pub last_heartbeat: Option<NaiveDateTime>,
    pub available_paths: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub schedule_type: String,
    pub priority: i32,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::schedules)]
pub struct NewSchedule {
    pub name: String,
    pub description: Option<String>,
    pub schedule_type: String,
    pub priority: i32,
    pub is_active: bool,
}

// Schedule Block models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::schedule_blocks)]
pub struct ScheduleBlock {
    pub id: i32,
    pub schedule_id: i32,
    pub content_id: Option<i32>,
    pub day_of_week: Option<i32>,
    pub specific_date: Option<NaiveDate>,
    pub start_time: NaiveTime,
    pub duration_minutes: i32,
    pub script_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
}

// Content Item models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::content_items)]
pub struct ContentItem {
    pub id: i32,
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
}

// Script models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::scripts)]
pub struct Script {
    pub id: i32,
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
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::node_schedules)]
pub struct NodeSchedule {
    pub id: i32,
    pub node_id: i32,
    pub schedule_id: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::schema::node_schedules)]
pub struct NewNodeSchedule {
    pub node_id: i32,
    pub schedule_id: i32,
}

// Permission models
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::permissions)]
pub struct Permission {
    pub id: i32,
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
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::global_settings)]
pub struct GlobalSetting {
    pub id: i32,
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
