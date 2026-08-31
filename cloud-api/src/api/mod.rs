pub mod account_manage;
pub mod filter;
pub mod hey_hi_hello;
pub mod script_caller;
// 依赖 master 回源注册，individual 模式下裁掉
#[cfg(not(feature = "individual"))]
pub mod system_manage;
