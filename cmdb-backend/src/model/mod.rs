// table struct
// KM relate table
pub mod km;
// others table
pub mod cloudserver;
pub mod lighthouse;
pub mod logservice;
pub mod cloudstorage;
// table schema
pub mod schema;

// cross-subsystem tables (feature-gated)
#[cfg(feature = "full-schema")]
pub mod watchman;
#[cfg(feature = "full-schema")]
pub mod yell;
