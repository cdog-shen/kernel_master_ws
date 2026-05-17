use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::schema::channel_configs::{self, dsl::*};

static UNKNOWN_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

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

impl ChannelConfig {
    /// 根据通道类型 + 实例名获取配置
    pub fn get_by_name(channel: &str, instance_name: &str, conn: &mut PgConnection) -> Result<Option<Self>, (u8, String)> {
        match channel_configs
            .filter(channel_type.eq(channel))
            .filter(name.eq(instance_name))
            .filter(is_enabled.eq(Some(true)))
            .select(ChannelConfig::as_select())
            .first(conn)
        {
            Ok(config) => Ok(Some(config)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 获取默认配置（name 为空字符串）
    pub fn get_default_by_type(channel: &str, conn: &mut PgConnection) -> Result<Option<Self>, (u8, String)> {
        Self::get_by_name(channel, "", conn)
    }

    /// 获取所有配置
    pub fn get_all(conn: &mut PgConnection) -> Result<Vec<Value>, (u8, String)> {
        match channel_configs.select(ChannelConfig::as_select()).load(conn) {
            Ok(configs) => Ok(configs
                .into_iter()
                .map(|c| serde_json::to_value(&c).unwrap())
                .collect()),
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
        .set((
            config_json.eq(config_value),
            is_enabled.eq(enabled),
        ))
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

    /// 获取 SMTP 配置（默认实例）
    pub fn get_smtp_config(conn: &mut PgConnection) -> Result<Option<SmtpConfig>, (u8, String)> {
        Self::get_smtp_config_by_name("", conn)
    }

    /// 获取指定实例的 SMTP 配置
    pub fn get_smtp_config_by_name(
        instance_name: &str,
        conn: &mut PgConnection,
    ) -> Result<Option<SmtpConfig>, (u8, String)> {
        match Self::get_by_name("smtp", instance_name, conn)? {
            Some(config) => {
                match serde_json::from_value::<SmtpConfig>(config.config_json) {
                    Ok(smtp_config) => Ok(Some(smtp_config)),
                    Err(e) => Err((BAD_REQUEST_CODE, format!("Invalid SMTP config: {}", e))),
                }
            }
            None => Ok(None),
        }
    }

    /// 获取 Bark 配置（默认实例）
    pub fn get_bark_config(conn: &mut PgConnection) -> Result<Option<BarkConfig>, (u8, String)> {
        Self::get_bark_config_by_name("", conn)
    }

    /// 获取指定实例的 Bark 配置
    pub fn get_bark_config_by_name(
        instance_name: &str,
        conn: &mut PgConnection,
    ) -> Result<Option<BarkConfig>, (u8, String)> {
        match Self::get_by_name("bark", instance_name, conn)? {
            Some(config) => match serde_json::from_value::<BarkConfig>(config.config_json) {
                Ok(bark_config) => Ok(Some(bark_config)),
                Err(e) => Err((BAD_REQUEST_CODE, format!("Invalid Bark config: {}", e))),
            },
            None => Ok(None),
        }
    }

    /// 获取 Gotify 配置（默认实例）
    pub fn get_gotify_config(conn: &mut PgConnection) -> Result<Option<GotifyConfig>, (u8, String)> {
        Self::get_gotify_config_by_name("", conn)
    }

    /// 获取指定实例的 Gotify 配置
    pub fn get_gotify_config_by_name(
        instance_name: &str,
        conn: &mut PgConnection,
    ) -> Result<Option<GotifyConfig>, (u8, String)> {
        match Self::get_by_name("gotify", instance_name, conn)? {
            Some(config) => match serde_json::from_value::<GotifyConfig>(config.config_json) {
                Ok(gotify_config) => Ok(Some(gotify_config)),
                Err(e) => Err((BAD_REQUEST_CODE, format!("Invalid Gotify config: {}", e))),
            },
            None => Ok(None),
        }
    }
}
