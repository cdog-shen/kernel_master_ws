use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

use crate::model::schema::notification_templates::{self, dsl::*};

static UNKNOWN_ERROR_CODE: u8 = 0;
static BAD_REQUEST_CODE: u8 = 1;

/// 渠道 JSONB 字段名列表
const CHANNEL_COLUMNS: &[&str] = &["smtp", "bark", "gotify", "ntfy", "teams", "webhook"];

#[derive(Queryable, Selectable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = notification_templates, check_for_backend(diesel::pg::Pg))]
pub struct NotificationTemplate {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub is_enabled: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub params_template: Option<Value>,
    pub smtp: Option<Value>,
    pub bark: Option<Value>,
    pub gotify: Option<Value>,
    pub ntfy: Option<Value>,
    pub teams: Option<Value>,
    pub webhook: Option<Value>,
}

#[derive(Insertable, Debug, Serialize, Deserialize)]
#[diesel(table_name = notification_templates)]
pub struct NewNotificationTemplate {
    pub name: String,
    pub description: Option<String>,
    pub is_enabled: Option<bool>,
    pub params_template: Option<Value>,
    pub smtp: Option<Value>,
    pub bark: Option<Value>,
    pub gotify: Option<Value>,
    pub ntfy: Option<Value>,
    pub teams: Option<Value>,
    pub webhook: Option<Value>,
}

#[derive(AsChangeset, Debug, Serialize, Deserialize)]
#[diesel(table_name = notification_templates)]
pub struct UpdateNotificationTemplate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_enabled: Option<bool>,
    pub params_template: Option<Value>,
    pub smtp: Option<Value>,
    pub bark: Option<Value>,
    pub gotify: Option<Value>,
    pub ntfy: Option<Value>,
    pub teams: Option<Value>,
    pub webhook: Option<Value>,
    pub updated_at: Option<NaiveDateTime>,
}

impl NotificationTemplate {
    /// 根据 ID 获取模板
    pub fn get_by_id(template_id: i32, conn: &mut PgConnection) -> Result<Option<Self>, (u8, String)> {
        match notification_templates
            .filter(id.eq(template_id))
            .filter(is_enabled.eq(Some(true)))
            .select(NotificationTemplate::as_select())
            .first(conn)
        {
            Ok(template) => Ok(Some(template)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 根据名称获取模板
    pub fn get_by_name(template_name: &str, conn: &mut PgConnection) -> Result<Option<Self>, (u8, String)> {
        match notification_templates
            .filter(name.eq(template_name))
            .filter(is_enabled.eq(Some(true)))
            .select(NotificationTemplate::as_select())
            .first(conn)
        {
            Ok(template) => Ok(Some(template)),
            Err(diesel::result::Error::NotFound) => Ok(None),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 根据过滤器获取模板列表
    pub fn get_with_filter(
        filter: &Map<String, Value>,
        conn: &mut PgConnection,
    ) -> Result<Vec<Value>, (u8, String)> {
        let mut query = notification_templates
            .into_boxed()
            .select(NotificationTemplate::as_select());

        for (q_k, q_v) in filter.iter() {
            match q_k.as_str() {
                "is_enabled" => {
                    if let Some(value) = q_v.as_bool() {
                        query = query.filter(is_enabled.eq(value));
                    }
                }
                "name" => {
                    if let Some(value) = q_v.as_str() {
                        query = query.filter(name.eq(value));
                    }
                }
                _ => continue,
            }
        }

        match query.load::<NotificationTemplate>(conn) {
            Ok(templates) => Ok(templates
                .into_iter()
                .map(|t| serde_json::to_value(&t).unwrap())
                .collect()),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 创建新模板
    pub fn create(
        new_template: &NewNotificationTemplate,
        conn: &mut PgConnection,
    ) -> Result<i32, (u8, String)> {
        match diesel::insert_into(notification_templates)
            .values(new_template)
            .returning(id)
            .get_result::<i32>(conn)
        {
            Ok(new_id) => Ok(new_id),
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 更新模板
    pub fn update(
        template_id: i32,
        update: &UpdateNotificationTemplate,
        conn: &mut PgConnection,
    ) -> Result<usize, (u8, String)> {
        match diesel::update(notification_templates.filter(id.eq(template_id)))
            .set(update)
            .execute(conn)
        {
            Ok(num) => match num {
                0 => Err((BAD_REQUEST_CODE, format!("Template id: {} not found", template_id))),
                _ => Ok(num),
            },
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 删除模板
    pub fn delete(template_id: i32, conn: &mut PgConnection) -> Result<usize, (u8, String)> {
        match diesel::delete(notification_templates.filter(id.eq(template_id))).execute(conn) {
            Ok(num) => match num {
                0 => Err((BAD_REQUEST_CODE, format!("Template id: {} not found", template_id))),
                _ => Ok(num),
            },
            Err(e) => Err((UNKNOWN_ERROR_CODE, e.to_string())),
        }
    }

    /// 渲染模板，返回 HashMap<channel_type, rendered_payload_json>
    /// 只渲染模板中已配置的渠道字段
    pub fn render(&self, variables: &Map<String, Value>) -> HashMap<String, Value> {
        let mut result = HashMap::new();
        for col in CHANNEL_COLUMNS {
            if let Some(json_val) = self.get_channel_json(col) {
                let rendered = self.render_value(json_val, variables);
                result.insert(col.to_string(), rendered);
            }
        }
        result
    }

    /// 获取指定渠道的 JSONB 字段
    fn get_channel_json(&self, col: &str) -> Option<&Value> {
        match col {
            "smtp" => self.smtp.as_ref(),
            "bark" => self.bark.as_ref(),
            "gotify" => self.gotify.as_ref(),
            "ntfy" => self.ntfy.as_ref(),
            "teams" => self.teams.as_ref(),
            "webhook" => self.webhook.as_ref(),
            _ => None,
        }
    }

    fn replace_vars(&self, template: &str, variables: &Map<String, Value>) -> String {
        let mut result = template.to_string();
        for (key, value) in variables.iter() {
            let placeholder = format!("{{{{{}}}}}", key);
            let value_str = value.as_str().unwrap_or("").to_string();
            result = result.replace(&placeholder, &value_str);
        }
        result
    }

    /// 递归渲染 JSON Value 中的 {{var}} 占位符
    fn render_value(&self, value: &Value, variables: &Map<String, Value>) -> Value {
        match value {
            Value::String(s) => Value::String(self.replace_vars(s, variables)),
            Value::Object(map) => {
                let mut rendered = serde_json::Map::new();
                for (k, v) in map {
                    rendered.insert(k.clone(), self.render_value(v, variables));
                }
                Value::Object(rendered)
            }
            Value::Array(arr) => Value::Array(arr.iter().map(|v| self.render_value(v, variables)).collect()),
            other => other.clone(),
        }
    }
}
