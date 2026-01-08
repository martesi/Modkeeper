use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
#[serde(untagged)]
pub enum Author {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct Compatibility {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct Dependency {
    pub id: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Effect {
    Trader,
    Item,
    Other,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LinkType {
    Code,
    Discord,
    Website,
    Documentation,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct Link {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub link_type: Option<LinkType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub url: String,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
#[serde(untagged)]
pub enum Dependencies {
    Object(BTreeMap<String, String>),
    Array(Vec<Dependency>),
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct ModManifest {
    pub id: String,
    pub name: String,
    pub author: Author,
    pub version: String,
    #[serde(rename = "sptVersion")]
    pub spt_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<Compatibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Dependencies>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug, PartialEq)]
pub enum ModType {
    Client,
    Server,
    Both,
    Unknown,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct Mod {
    pub id: String,
    pub is_active: bool,
    pub mod_type: ModType,
    pub name: String,
    pub manifest: Option<ModManifest>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub icon_data: Option<String>,
    // files removed: only needed in cache, not for frontend display
}
