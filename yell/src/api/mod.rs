pub mod alias_manage;
pub mod channel_manage;
pub mod filter;
pub mod hey_hi_hello;
pub mod notify;
pub mod record_manage;
// individual 模式下无 master，主关注册端点编译期裁掉
#[cfg(not(feature = "individual"))]
pub mod system_manage;
pub mod template_manage;
