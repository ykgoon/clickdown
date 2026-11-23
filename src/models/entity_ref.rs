//! The reference stub ClickUp embeds inside other payloads.
//!
//! A task's `list`/`folder`/`space`/`project`, a list's `space`/`folder`, and a
//! document's `space`/`folder` are all the same shape: an id plus whatever
//! descriptive fields the endpoint felt like including. They used to be six
//! near-identical structs, which meant six places to fix when a field came back
//! null. Only `id` is load-bearing; everything else is display metadata and is
//! optional by default.

use serde::{Deserialize, Serialize};

/// A `{ id, name, hidden, access }` stub.
///
/// Absent fields deserialize to `None` rather than failing, because ClickUp
/// omits whichever ones the caller does not need for that endpoint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EntityRef {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub hidden: Option<bool>,
    #[serde(default)]
    pub access: Option<bool>,
}
