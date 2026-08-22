//! Caller-owned configured error definitions and rendering.
//!
//! Stable error identity remains in `audiacore-errors`. This crate owns only
//! presentation metadata loaded from caller-supplied YAML: kind, canonical
//! message template, and resolution. It performs no discovery or I/O, keeps no
//! global registry, and never exposes diagnostic error details implicitly to
//! templates.

use std::{collections::BTreeMap, error::Error, fmt};

use audiacore_errors::ErrorCode;
use audiacore_template::{Template, TemplateContext, TemplateError, TemplateValue};
use serde::{
    Deserialize, Deserializer,
    de::{self, MapAccess, Visitor},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorDefinition {
    kind: String,
    message: Template,
    resolution: String,
    source: String,
}

impl ErrorDefinition {
    pub fn kind(&self) -> &str {
        &self.kind
    }

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
    kind: String,
    message: String,
    resolution: String,
}

impl RenderedError {
    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn kind(&self) -> &str {
        &self.kind
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

    /// Register one component-owned catalogue layer.
    ///
    /// The whole layer is parsed and validated before mutation. A code already
    /// registered by another component source is rejected rather than silently
    /// changing ownership.
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

    /// Overlay a complete configured layer at an application/configuration edge.
    ///
    /// This is explicit replacement of whole definitions, preserving the
    /// original project's later-source override semantics without introducing
    /// filesystem discovery or an ambient global registry here.
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
            kind: definition.kind.clone(),
            message,
            resolution: definition.resolution.clone(),
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDefinition {
    kind: String,
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
        if !valid_kind(&raw.kind) {
            return Err(ErrorCatalogueError::InvalidKind {
                source: source.to_owned(),
                code,
                kind: raw.kind,
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
                kind: raw.kind,
                message,
                resolution: raw.resolution,
                source: source.to_owned(),
            },
        );
    }

    Ok(parsed)
}

fn valid_kind(kind: &str) -> bool {
    let bytes = kind.as_bytes();
    if bytes.is_empty() || !bytes[0].is_ascii_lowercase() {
        return false;
    }

    let mut previous_hyphen = false;
    for &byte in bytes {
        if byte == b'-' {
            if previous_hyphen {
                return false;
            }
            previous_hyphen = true;
        } else if byte.is_ascii_lowercase() || byte.is_ascii_digit() {
            previous_hyphen = false;
        } else {
            return false;
        }
    }

    !previous_hyphen
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
    InvalidKind {
        source: String,
        code: String,
        kind: String,
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
            Self::InvalidKind { source, code, kind } => {
                write!(f, "invalid error kind {kind:?} for {code} in {source}")
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
                write!(
                    f,
                    "no configured definition registered for {}",
                    code.as_str()
                )
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
            | Self::InvalidKind { .. }
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
CON-WORKFLOW-002:
  kind: constraint
  message: "Workflow '{workflow.id}' changed from revision {expected} to {actual}."
  resolution: "Reload the latest workflow state and retry."
"#;

    fn params() -> TemplateContext<String, TemplateValue> {
        let mut workflow = TemplateContext::new();
        workflow.insert("id".to_owned(), TemplateValue::String("wf-7".to_owned()));
        let mut params = TemplateContext::new();
        params.insert("workflow".to_owned(), TemplateValue::Object(workflow));
        params.insert("expected".to_owned(), TemplateValue::from(3));
        params.insert("actual".to_owned(), TemplateValue::from(4));
        params
    }

    #[test]
    fn component_catalogue_owns_static_error_presentation() {
        let mut catalogue = ErrorCatalogue::new();
        catalogue
            .register_yaml("workflow/errors.yaml", BASE)
            .unwrap();

        let rendered = catalogue
            .render(ErrorCode::new("CON-WORKFLOW-002"), &params())
            .unwrap();

        assert_eq!(rendered.kind(), "constraint");
        assert_eq!(
            rendered.message(),
            "Workflow 'wf-7' changed from revision 3 to 4."
        );
        assert_eq!(
            rendered.resolution(),
            "Reload the latest workflow state and retry."
        );
        assert_eq!(
            catalogue
                .definition(ErrorCode::new("CON-WORKFLOW-002"))
                .unwrap()
                .source(),
            "workflow/errors.yaml"
        );
    }

    #[test]
    fn missing_required_params_fail_without_accessing_diagnostics_or_ambient_state() {
        let mut catalogue = ErrorCatalogue::new();
        catalogue
            .register_yaml("workflow/errors.yaml", BASE)
            .unwrap();
        let error = catalogue
            .render(ErrorCode::new("CON-WORKFLOW-002"), &TemplateContext::new())
            .unwrap_err();

        assert!(matches!(error, ErrorCatalogueError::Render { .. }));
        assert_eq!(
            error.source().unwrap().to_string(),
            "missing template value: workflow.id"
        );
    }

    #[test]
    fn extra_message_params_are_tolerated() {
        let mut catalogue = ErrorCatalogue::new();
        catalogue
            .register_yaml("workflow/errors.yaml", BASE)
            .unwrap();
        let mut values = params();
        values.insert(
            "unused".to_owned(),
            TemplateValue::String("safe".to_owned()),
        );

        assert!(
            catalogue
                .render(ErrorCode::new("CON-WORKFLOW-002"), &values)
                .is_ok()
        );
    }

    #[test]
    fn duplicate_codes_inside_one_yaml_source_are_rejected() {
        let duplicate = r#"
VAL-THING-001:
  kind: validation
  message: "one"
  resolution: "fix one"
VAL-THING-001:
  kind: validation
  message: "two"
  resolution: "fix two"
"#;
        let error = ErrorCatalogue::new()
            .register_yaml("thing/errors.yaml", duplicate)
            .unwrap_err();
        assert!(matches!(error, ErrorCatalogueError::InvalidYaml { .. }));
    }

    #[test]
    fn component_registration_rejects_cross_source_duplicates() {
        let mut catalogue = ErrorCatalogue::new();
        catalogue.register_yaml("first/errors.yaml", BASE).unwrap();
        let error = catalogue
            .register_yaml("second/errors.yaml", BASE)
            .unwrap_err();

        assert!(matches!(error, ErrorCatalogueError::DuplicateCode { .. }));
    }

    #[test]
    fn explicit_overlay_replaces_the_complete_definition_atomically() {
        let mut catalogue = ErrorCatalogue::new();
        catalogue
            .register_yaml("workflow/errors.yaml", BASE)
            .unwrap();
        catalogue
            .overlay_yaml(
                "project/errors.yaml",
                r#"
CON-WORKFLOW-002:
  kind: constraint
  message: "Revision {actual} superseded {expected}."
  resolution: "Refresh before retrying."
"#,
            )
            .unwrap();

        let rendered = catalogue
            .render(ErrorCode::new("CON-WORKFLOW-002"), &params())
            .unwrap();
        assert_eq!(rendered.message(), "Revision 4 superseded 3.");
        assert_eq!(rendered.resolution(), "Refresh before retrying.");
        assert_eq!(
            catalogue
                .definition(ErrorCode::new("CON-WORKFLOW-002"))
                .unwrap()
                .source(),
            "project/errors.yaml"
        );
    }

    #[test]
    fn malformed_identity_kind_and_template_fail_before_registration() {
        let invalid_code = r#"
VAL-1THING-001:
  kind: validation
  message: "bad"
  resolution: "fix"
"#;
        assert!(matches!(
            ErrorCatalogue::new()
                .register_yaml("bad-code", invalid_code)
                .unwrap_err(),
            ErrorCatalogueError::InvalidCode { .. }
        ));

        let invalid_kind = r#"
VAL-THING-001:
  kind: Bad Kind
  message: "bad"
  resolution: "fix"
"#;
        assert!(matches!(
            ErrorCatalogue::new()
                .register_yaml("bad-kind", invalid_kind)
                .unwrap_err(),
            ErrorCatalogueError::InvalidKind { .. }
        ));

        let invalid_template = r#"
VAL-THING-001:
  kind: validation
  message: "missing {brace"
  resolution: "fix"
"#;
        assert!(matches!(
            ErrorCatalogue::new()
                .register_yaml("bad-template", invalid_template)
                .unwrap_err(),
            ErrorCatalogueError::InvalidTemplate { .. }
        ));
    }
}
