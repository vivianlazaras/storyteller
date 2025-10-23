#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
//#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(improper_ctypes)]
#![allow(unused_variables)]
#![allow(non_local_impls)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate err_derive;
use time::UtcOffset;
use time::{OffsetDateTime, format_description, format_description::well_known::Rfc3339};

pub mod characters;
pub mod config;
pub mod errors;
pub mod locations;
mod model;
pub mod profiles;
pub mod stories;
pub use config::Config;
pub mod api;
pub mod assets;
pub mod auth;
pub mod fragments;
pub mod groups;
pub mod notes;
pub mod organizations;
pub mod patterns;
pub mod relations;
pub mod render;
pub mod search;
pub mod themes;
pub mod timelines;
pub use api::*;
mod tests;
