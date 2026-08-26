//! Ordered in-memory configuration resolution with exact-input provenance.
//!
//! This crate parses and merges configuration content already acquired by an
//! application edge. It never discovers files, reads environment variables,
//! performs I/O, or owns policy semantics.

use std::{error::Error, fmt};

use audiacore_errors::{CodedError, ErrorCode};
use serde::de::DeserializeOwned;
use toml::{Table, Value};

const EMPTY_LAYER_ID: ErrorCode = ErrorCode::new("VAL-CONFIG-001");
const INVALID_LAYER: ErrorCode = ErrorCode::new("CFG-CONFIG-001");
const RESOLUTION_FAILED: ErrorCode = ErrorCode::new("CFG-CONFIG-002");

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x00000100000001b3;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConfigLayerId(String);

impl ConfigLayerId {
    pub fn new(value: impl Into<String>) -> Result<Self, ConfigError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ConfigError::EmptyLayerId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ConfigLayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConfigRevision(u64);

impl ConfigRevision {
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ConfigRevision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    EmptyLayerId,
    InvalidLayer {
        layer: ConfigLayerId,
        source: toml::de::Error,
    },
    ResolutionFailed {
        source: toml::de::Error,
    },
}

impl CodedError for ConfigError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::EmptyLayerId => EMPTY_LAYER_ID,
            Self::InvalidLayer { .. } => INVALID_LAYER,
            Self::ResolutionFailed { .. } => RESOLUTION_FAILED,
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyLayerId => f.write_str("configuration layer id must not be empty"),
            Self::InvalidLayer { layer, .. } => {
                write!(f, "configuration layer {layer} is invalid TOML")
            }
            Self::ResolutionFailed { .. } => f.write_str("configuration resolution failed"),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EmptyLayerId => None,
            Self::InvalidLayer { source, .. } | Self::ResolutionFailed { source } => Some(source),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConfig<T> {
    value: T,
    revision: ConfigRevision,
    layers: Vec<ConfigLayerId>,
}

impl<T> ResolvedConfig<T> {
    pub const fn value(&self) -> &T {
        &self.value
    }

    pub const fn revision(&self) -> ConfigRevision {
        self.revision
    }

    pub fn layers(&self) -> &[ConfigLayerId] {
        &self.layers
    }
}

#[derive(Debug, Clone)]
pub struct ConfigLayers {
    merged: Table,
    revision_state: u64,
    layers: Vec<ConfigLayerId>,
}

impl Default for ConfigLayers {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigLayers {
    pub fn new() -> Self {
        Self {
            merged: Table::new(),
            revision_state: FNV_OFFSET_BASIS,
            layers: Vec::new(),
        }
    }

    pub fn merge_toml(mut self, id: ConfigLayerId, source: &str) -> Result<Self, ConfigError> {
        let parsed = source
            .parse::<Table>()
            .map_err(|source| ConfigError::InvalidLayer {
                layer: id.clone(),
                source,
            })?;

        merge_table(&mut self.merged, parsed);
        self.revision_state = hash_part(self.revision_state, id.as_str().as_bytes());
        self.revision_state = hash_part(self.revision_state, source.as_bytes());
        self.layers.push(id);
        Ok(self)
    }

    pub fn resolve<T>(self) -> Result<ResolvedConfig<T>, ConfigError>
    where
        T: DeserializeOwned,
    {
        let value = T::deserialize(self.merged)
            .map_err(|source| ConfigError::ResolutionFailed { source })?;

        Ok(ResolvedConfig {
            value,
            revision: ConfigRevision(self.revision_state),
            layers: self.layers,
        })
    }
}

fn merge_table(base: &mut Table, incoming: Table) {
    for (key, incoming_value) in incoming {
        match incoming_value {
            Value::Table(incoming_table) => match base.get_mut(&key) {
                Some(Value::Table(base_table)) => merge_table(base_table, incoming_table),
                _ => {
                    base.insert(key, Value::Table(incoming_table));
                }
            },
            value => {
                base.insert(key, value);
            }
        }
    }
}

fn hash_part(mut state: u64, bytes: &[u8]) -> u64 {
    for byte in (bytes.len() as u64).to_le_bytes() {
        state = fnv_byte(state, byte);
    }
    for &byte in bytes {
        state = fnv_byte(state, byte);
    }
    state
}

const fn fnv_byte(state: u64, byte: u8) -> u64 {
    (state ^ byte as u64).wrapping_mul(FNV_PRIME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct TestConfig {
        events: EventConfig,
        nested: NestedConfig,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct EventConfig {
        retention: usize,
        mode: String,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct NestedConfig {
        left: u32,
        right: u32,
    }

    fn layer(id: &str) -> ConfigLayerId {
        ConfigLayerId::new(id).unwrap()
    }

    #[test]
    fn default_and_new_share_the_same_provenance_basis() {
        assert_eq!(
            ConfigLayers::default().revision_state,
            ConfigLayers::new().revision_state
        );
    }

    #[test]
    fn ordered_layers_recursively_override_without_external_sources() {
        let resolved = ConfigLayers::new()
            .merge_toml(
                layer("defaults"),
                r#"
                    [events]
                    retention = 8
                    mode = "durable"
                    [nested]
                    left = 1
                    right = 2
                "#,
            )
            .unwrap()
            .merge_toml(
                layer("project"),
                r#"
                    [events]
                    retention = 4
                    [nested]
                    right = 9
                "#,
            )
            .unwrap()
            .resolve::<TestConfig>()
            .unwrap();

        assert_eq!(resolved.value().events.retention, 4);
        assert_eq!(resolved.value().events.mode, "durable");
        assert_eq!(resolved.value().nested.left, 1);
        assert_eq!(resolved.value().nested.right, 9);
        assert_eq!(
            resolved
                .layers()
                .iter()
                .map(ConfigLayerId::as_str)
                .collect::<Vec<_>>(),
            vec!["defaults", "project"]
        );
    }

    #[test]
    fn revision_tracks_exact_ordered_inputs_not_semantic_equivalence() {
        let first = ConfigLayers::new()
            .merge_toml(layer("defaults"), "value = 1\n")
            .unwrap();
        let second = ConfigLayers::new()
            .merge_toml(layer("defaults"), "value=1\n")
            .unwrap();

        #[derive(Debug, Deserialize)]
        struct ValueConfig {
            value: u32,
        }

        let first = first.resolve::<ValueConfig>().unwrap();
        let second = second.resolve::<ValueConfig>().unwrap();
        assert_eq!(first.value().value, second.value().value);
        assert_ne!(first.revision(), second.revision());
    }

    #[test]
    fn layer_order_changes_revision_and_effective_value() {
        #[derive(Debug, Deserialize)]
        struct ValueConfig {
            value: u32,
        }

        let first = ConfigLayers::new()
            .merge_toml(layer("one"), "value = 1")
            .unwrap()
            .merge_toml(layer("two"), "value = 2")
            .unwrap()
            .resolve::<ValueConfig>()
            .unwrap();
        let reversed = ConfigLayers::new()
            .merge_toml(layer("two"), "value = 2")
            .unwrap()
            .merge_toml(layer("one"), "value = 1")
            .unwrap()
            .resolve::<ValueConfig>()
            .unwrap();

        assert_eq!(first.value().value, 2);
        assert_eq!(reversed.value().value, 1);
        assert_ne!(first.revision(), reversed.revision());
    }

    #[test]
    fn invalid_layer_and_resolution_have_distinct_stable_codes() {
        let parse = ConfigLayers::new()
            .merge_toml(layer("broken"), "[not closed")
            .unwrap_err();
        assert_eq!(parse.code().as_str(), "CFG-CONFIG-001");

        let resolution = ConfigLayers::new()
            .merge_toml(layer("wrong-type"), "value = true")
            .unwrap()
            .resolve::<u32>()
            .unwrap_err();
        assert_eq!(resolution.code().as_str(), "CFG-CONFIG-002");
    }

    #[test]
    fn empty_layer_id_is_rejected_with_stable_identity() {
        let error = ConfigLayerId::new(" ").unwrap_err();
        assert_eq!(error.code().as_str(), "VAL-CONFIG-001");
    }
}
