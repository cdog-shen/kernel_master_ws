//! actix-web 中间件集合（feature `web`）
//!
//! - `user_auth`：子系统统一鉴权中间件（Bearer 回源鉴权 + uuid 旧链路，
//!   feature `individual` 下仅编译 uuid 比对分支）

pub mod user_auth;
