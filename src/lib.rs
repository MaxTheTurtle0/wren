//! Wren: a fast, native client for Microsoft Teams.
//!
//! See README.md for what works today and docs/architecture.md for how the
//! pieces fit: `ui` draws state and emits `model::Action`s, `app` applies
//! them, `backend` runs every network and disk task on its own runtime.

pub mod app;
pub mod auth;
pub mod backend;
pub mod calling;
pub mod clipboard;
pub mod demo;
pub mod graph;
pub mod html;
pub mod model;
pub mod notify;
pub mod paths;
pub mod platform;
pub mod realtime;
pub mod secrets;
pub mod settings;
pub mod store;
pub mod sync;
pub mod theme;
pub mod time;
pub mod ui;
pub mod updates;
