#![feature(let_chains)]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate serde;

pub mod assets;
pub mod auth;
pub mod cx;
pub mod db;
pub mod docs;
pub mod err;
pub mod files;
pub mod logging;
pub mod router;
pub mod run;
pub mod schema;
pub mod templates;
pub mod tokens;
