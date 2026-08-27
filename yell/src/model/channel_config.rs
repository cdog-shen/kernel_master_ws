use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::schema::channel_configs::{self, dsl::*};

static UNKNOWN_ERROR_CODE: u8 = 0;

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = channel_configs, check_for_backend(diesel::pg::Pg))]
pub struct ChannelConfig {
    pub id: i32,
    pub channel_type: String,
    pub name: String,
    pub config_json: Value,
    pub is_enabled: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = channel_configs)]
pub struct NewChannelConfig {
    pub channel_type: String,
    pub name: String,
    pub config_json: Value,
    pub is_enabled: Option<bool>,
}

#[derive(AsChangeset, Debug, Serialize, Deserialize)]
#[diesel(table_name = channel_configs)]
pub struct UpdateChannelConfig {
    pub config_json: Option<Value>,
    pub is_enabled: Option<bool>,
    pub updated_at: Option<NaiveDateTime>,
}

/// SMTP 配置结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub use_tls: bool,
}

/// Bark 配置结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BarkConfig {
    pub server_url: String,
    pub device_key: Option<String>,
}

/// Gotify 配置结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GotifyConfig {
    pub server_url: String,
    pub app_token: String,
}

/// Teams 配置结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TeamsConfig {
    pub webhook_url: String,
}

/// 通用 Webhook 配置结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebhookConfig {
    pub webhook_url: String,
}

impl ChannelConfig {
    /// 获取所有配置
    pub fn get_all(conn: &mut PgConnection) -> Result<Vec<Value>, (u8, String)> {
        match channel_configs
            .select(ChannelConfig::as_select())
            .load(conn)
        {
            Ok(configs) => configs
                .into_iter()
                .map(|c| serde_json::to_value(&c).map_err(|e| (UNKNOWN_ERROR_CODE, e.to_string())))
                .collect(),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 创建或更新配置（按 channel_type + name 联合键）
    pub fn upsert(
        channel: &str,
        instance_name: &str,
        config_value: &Value,
        enabled: bool,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        // 尝试更新现有配置
        let update_result = diesel::update(
            channel_configs
                .filter(channel_type.eq(channel))
                .filter(name.eq(instance_name)),
        )
        .set((config_json.eq(config_value), is_enabled.eq(enabled)))
        .execute(conn);

        match update_result {
            Ok(0) => {
                // 没有更新到记录，插入新记录
                let new_config = NewChannelConfig {
                    channel_type: channel.to_string(),
                    name: instance_name.to_string(),
                    config_json: config_value.clone(),
                    is_enabled: Some(enabled),
                };
                match diesel::insert_into(channel_configs)
                    .values(&new_config)
                    .execute(conn)
                {
                    Ok(_) => Ok(1),
                    Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
                }
            }
            Ok(n) => Ok(n),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }
}
