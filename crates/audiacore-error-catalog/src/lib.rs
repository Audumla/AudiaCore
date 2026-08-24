//! Caller-owned configured error definitions and rendering.
//!
//! Stable error code/category identity remains in `audiacore-errors`. This crate
//! owns caller-supplied canonical messages, resolutions, and definition-source
//! provenance. It performs no discovery or I/O and keeps no global registry.

use std::{collections::BTreeMap, error::Error, fmt};

use audiacore_errors::ErrorCode;
use audiacore_template::{Template, TemplateContext, TemplateError, TemplateValue};
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, Visitor},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorDefinition {
    message: Template,
    resolution: String,
    source: String,
}

impl ErrorDefinition {
    pub fn resolution(&self) -> &str {
        &self.resolution
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedError {
    code: ErrorCode,
    message: String,
    resolution: String,
}

impl RenderedError {
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn kind(&self) -> &'static str {
        self.code.category().as_str()
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn resolution(&self) -> &str {
        &self.resolution
    }
}

#[derive(Debug, Default)]
pub struct ErrorCatalogue {
    definitions: BTreeMap<String, ErrorDefinition>,
}

impl ErrorCatalogue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one owner-local catalogue layer. Duplicate codes across owners
    /// are rejected rather than silently changing ownership.
    pub fn register_yaml(
        &mut self,
        source: impl Into<String>,
        yaml: &str,
    ) -> Result<(), ErrorCatalogueError> {
        let source = source.into();
        let parsed = parse_layer(&source, yaml)?;

        for code in parsed.keys() {
            if let Some(existing) = self.definitions.get(code) {
                return Err(ErrorCatalogueError::DuplicateCode {
                    code: code.clone(),
                    first_source: existing.source.clone(),
                    second_source: source,
                });
            }
        }

        self.definitions.extend(parsed);
        Ok(())
    }

    /// Explicitly overlay a complete configured definition layer at an
    /// application/configuration edge.
    pub fn overlay_yaml(
        &mut self,
        source: impl Into<String>,
        yaml: &str,
    ) -> Result<(), ErrorCatalogueError> {
        let source = source.into();
        let parsed = parse_layer(&source, yaml)?;
        self.definitions.extend(parsed);
        Ok(())
    }

    pub fn definition(&self, code: ErrorCode) -> Option<&ErrorDefinition> {
        self.definitions.get(code.as_str())
    }

    pub fn render(
        &self,
        code: ErrorCode,
        message_params: &TemplateContext<String, TemplateValue>,
    ) -> Result<RenderedError, ErrorCatalogueError> {
        let definition = self
            .definition(code)
            .ok_or(ErrorCatalogueError::MissingDefinition { code })?;
        let message = definition
            .message
            .render(message_params)
            .map_err(|source| ErrorCatalogueError::Render { code, source })?;

        Ok(RenderedError {
            code,
            message,
            resolution: definition.resolution.clone(),
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDefinition {
    message: String,
    resolution: String,
}

struct RawCatalogue(BTreeMap<String, RawDefinition>);

impl<'de> Deserialize<'de> for RawCatalogue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CatalogueVisitor;

        impl<'de> Visitor<'de> for CatalogueVisitor {
            type Value = RawCatalogue;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a mapping of stable error codes to complete definitions")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut definitions = BTreeMap::new();
                while let Some((code, definition)) = map.next_entry::<String, RawDefinition>()? {
                    if definitions.insert(code.clone(), definition).is_some() {
                        return Err(de::Error::custom(format!(
                            "duplicate error definition for {code}"
                        )));
                    }
                }
                Ok(RawCatalogue(definitions))
            }
        }

        deserializer.deserialize_map(CatalogueVisitor)
    }
}

fn parse_layer(
    source: &str,
    yaml: &str,
) -> Result<BTreeMap<String, ErrorDefinition>, ErrorCatalogueError> {
    let RawCatalogue(raw) =
        yaml_serde::from_str(yaml).map_err(|error| ErrorCatalogueError::InvalidYaml {
            source_name: source.to_owned(),
            source: error,
        })?;

    let mut parsed = BTreeMap::new();
    for (code, raw) in raw {
        if !ErrorCode::is_valid(&code) {
            return Err(ErrorCatalogueError::InvalidCode {
                source: source.to_owned(),
                code,
            });
        }
        if raw.message.trim().is_empty() {
            return Err(ErrorCatalogueError::EmptyMessage {
                source: source.to_owned(),
                code,
            });
        }
        if raw.resolution.trim().is_empty() {
            return Err(ErrorCatalogueError::EmptyResolution {
                source: source.to_owned(),
                code,
            });
        }

        let message = Template::parse(&raw.message).map_err(|error| {
            ErrorCatalogueError::InvalidTemplate {
                source: source.to_owned(),
                code: code.clone(),
                error,
            }
        })?;
        parsed.insert(
            code,
            ErrorDefinition {
                message,
                resolution: raw.resolution,
                source: source.to_owned(),
            },
        );
    }

    Ok(parsed)
}

#[derive(Debug)]
pub enum ErrorCatalogueError {
    InvalidYaml {
        source_name: String,
        source: yaml_serde::Error,
    },
    InvalidCode {
        source: String,
        code: String,
    },
    EmptyMessage {
        source: String,
        code: String,
    },
    EmptyResolution {
        source: String,
        code: String,
    },
    InvalidTemplate {
        source: String,
        code: String,
        error: TemplateError,
    },
    DuplicateCode {
        code: String,
        first_source: String,
        second_source: String,
    },
    MissingDefinition {
        code: ErrorCode,
    },
    Render {
        code: ErrorCode,
        source: TemplateError,
    },
}

impl fmt::Display for ErrorCatalogueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidYaml { source_name, .. } => {
                write!(f, "invalid error catalogue YAML from {source_name}")
            }
            Self::InvalidCode { source, code } => {
                write!(f, "invalid error code {code:?} in {source}")
            }
            Self::EmptyMessage { source, code } => {
                write!(f, "empty canonical message for {code} in {source}")
            }
            Self::EmptyResolution { source, code } => {
                write!(f, "empty resolution for {code} in {source}")
            }
            Self::InvalidTemplate { source, code, .. } => {
                write!(f, "invalid message template for {code} in {source}")
            }
            Self::DuplicateCode {
                code,
                first_source,
                second_source,
            } => write!(
                f,
                "duplicate error code {code} in {first_source} and {second_source}"
            ),
            Self::MissingDefinition { code } => {
                write!(f, "no configured definition registered for {}", code.as_str())
            }
            Self::Render { code, .. } => write!(
                f,
                "configured message for {} could not be rendered",
                code.as_str()
            ),
        }
    }
}

impl Error for ErrorCatalogueError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidYaml { source, .. } => Some(source),
            Self::InvalidTemplate { error, .. } | Self::Render { source: error, .. } => Some(error),
            Self::InvalidCode { .. }
            | Self::EmptyMessage { .. }
            | Self::EmptyResolution { .. }
            | Self::DuplicateCode { .. }
            | Self::MissingDefinition { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = r#"
CON-EXAMPLE-002:
  message: "Object '{object.id}' changed from revision {expected} to {actual}."
  resolution: "Reload the latest state and retry."
"#;

    fn params() -> TemplateContext<String, TemplateValue> {
        let mut object = TemplateContext::new();
        object.insert("id".to_owned(), TemplateValue::String("obj-7".to_owned()));
        let mut params = TemplateContext::new();
        params.insert("object".to_owned(), TemplateValue::Object(object));
        params.insert("expected".to_owned(), TemplateValue::from(3));
        params.insert("actual".to_owned(), TemplateValue::from(4));
        params
    }

    #[test]
    fn code_owns_category_and_catalogue_owns_presentation() {
        let mut catalogue = ErrorCatalogue::new();
        catalogue.register_yaml("example/errors.yaml", BASE).unwrap();

        let rendered = catalogue
            .render(ErrorCode::new("CON-EXAMPLE-002"), &params())
            .unwrap();

        assert_eq!(rendered.kind(), "constraint");
        assert_eq!(
            rendered.message(),
            "Object 'obj-7' changed from revision 3 to 4."
        );
        assert_eq!(rendered.resolution(), "Reload the latest state and retry.");
        assert_eq!(
            catalogue
                .definition(ErrorCode::new("CON-EXAMPLE-002"))
                .unwrap()
                .source(),
            "example/errors.yaml"
        );
    }

    #[test]
    fn missing_required_params_fail_without_ambient_state() {
        let mut catalogue = ErrorCatalogue::new();
        catalogue.register_yaml("example/errors.yaml", BASE).unwrap();
        let error = catalogue
            .render(ErrorCode::new("CON-EXAMPLE-002"), &TemplateContext::new())
            .unwrap_err();

        assert!(matches!(error, ErrorCatalogueError::Render { .. }));
        assert_eq!(
            error.source().unwrap().to_string(),
            "missing template value: object.id"
        );
    }

    #[test]
    fn duplicate_codes_inside_or_across_sources_are_rejected() {
        let duplicate = r#"
VAL-THING-001:
  message: "one"
  resolution: "fix one"
VAL-THING-001:
  message: "two"
  resolution: "fix two"
"#;
        assert!(matches!(
            ErrorCatalogue::new()
                .register_yaml("thing/errors.yaml", duplicate)
                .unwrap_err(),
            ErrorCatalogueError::InvalidYaml { .. }
        ));

        let mut catalogue = ErrorCatalogue::new();
        catalogue.register_yaml("first/errors.yaml", BASE).unwrap();
        assert!(matches!(
            catalogue.register_yaml("second/errors.yaml", BASE).unwrap_err(),
            ErrorCatalogueError::DuplicateCode { .. }
        ));
    }

    #[test]
    fn explicit_overlay_replaces_complete_definition() {
        let mut catalogue = ErrorCatalogue::new();
        catalogue.register_yaml("example/errors.yaml", BASE).unwrap();
        catalogue
            .overlay_yaml(
                "project/errors.yaml",
                r#"
CON-EXAMPLE-002:
  message: "Revision {actual} superseded {expected}."
  resolution: "Refresh before retrying."
"#,
            )
            .unwrap();

        let rendered = catalogue
            .render(ErrorCode::new("CON-EXAMPLE-002"), &params())
            .unwrap();
        assert_eq!(rendered.message(), "Revision 4 superseded 3.");
        assert_eq!(rendered.resolution(), "Refresh before retrying.");
        assert_eq!(
            catalogue
                .definition(ErrorCode::new("CON-EXAMPLE-002"))
                .unwrap()
                .source(),
            "project/errors.yaml"
        );
    }

    #[test]
    fn legacy_kind_and_malformed_definitions_fail_before_registration() {
        let legacy_kind = r#"
VAL-THING-001:
  kind: validation
  message: "bad"
  resolution: "fix"
"#;
        assert!(matches!(
            ErrorCatalogue::new()
                .register_yaml("legacy-kind", legacy_kind)
                .unwrap_err(),
            ErrorCatalogueError::InvalidYaml { .. }
        ));

        let invalid_code = r#"
BAD-THING-001:
  message: "bad"
  resolution: "fix"
"#;
        assert!(matches!(
            ErrorCatalogue::new()
                .register_yaml("bad-code", invalid_code)
                .unwrap_err(),
            ErrorCatalogueError::InvalidCode { .. }
        ));
    }
}
