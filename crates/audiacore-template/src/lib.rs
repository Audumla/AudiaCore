//! Deterministic dotted-path templating over explicitly supplied mappings.
//!
//! Templates resolve `{dotted.path}` placeholders only through nested JSON
//! object mappings supplied by the caller. They never traverse Rust objects,
//! invoke methods, read ambient state, or perform I/O.

use std::{error::Error, fmt};

use audiacore_errors::{CodedError, ErrorCode};
pub use serde_json::{Map as TemplateContext, Value as TemplateValue};

const EMPTY_SLOT: ErrorCode = ErrorCode::new("VAL-TEMPLATE-001");
const UNCLOSED_SLOT: ErrorCode = ErrorCode::new("VAL-TEMPLATE-002");
const MISSING_VALUE: ErrorCode = ErrorCode::new("RES-TEMPLATE-001");

#[derive(Debug, Clone, PartialEq, Eq)]
enum Part {
    Literal(String),
    Slot(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    parts: Vec<Part>,
}

impl Template {
    pub fn parse(source: &str) -> Result<Self, TemplateError> {
        let mut parts = Vec::new();
        let mut cursor = 0;

        while let Some(relative_start) = source[cursor..].find('{') {
            let start = cursor + relative_start;
            if start > cursor {
                parts.push(Part::Literal(source[cursor..start].to_owned()));
            }

            let slot_start = start + 1;
            let relative_end = source[slot_start..]
                .find('}')
                .ok_or(TemplateError::UnclosedSlot)?;
            let end = slot_start + relative_end;
            let path = source[slot_start..end].trim();
            if path.is_empty() {
                return Err(TemplateError::EmptySlot);
            }
            parts.push(Part::Slot(path.to_owned()));
            cursor = end + 1;
        }

        if cursor < source.len() {
            parts.push(Part::Literal(source[cursor..].to_owned()));
        }

        Ok(Self { parts })
    }

    pub fn has_placeholders(&self) -> bool {
        self.parts.iter().any(|part| matches!(part, Part::Slot(_)))
    }

    pub fn render(
        &self,
        context: &TemplateContext<String, TemplateValue>,
    ) -> Result<String, TemplateError> {
        let mut rendered = String::new();
        for part in &self.parts {
            match part {
                Part::Literal(value) => rendered.push_str(value),
                Part::Slot(path) => {
                    let value = resolve_path(context, path)
                        .ok_or_else(|| TemplateError::MissingValue(path.clone()))?;
                    render_value(value, &mut rendered);
                }
            }
        }
        Ok(rendered)
    }
}

fn resolve_path<'a>(
    context: &'a TemplateContext<String, TemplateValue>,
    path: &str,
) -> Option<&'a TemplateValue> {
    let mut segments = path.split('.');
    let first = segments.next()?;
    let mut current = context.get(first)?;

    for segment in segments {
        current = match current {
            TemplateValue::Object(mapping) => mapping.get(segment)?,
            _ => return None,
        };
    }

    Some(current)
}

fn render_value(value: &TemplateValue, rendered: &mut String) {
    match value {
        TemplateValue::Null => {}
        TemplateValue::String(value) => rendered.push_str(value),
        TemplateValue::Bool(_)
        | TemplateValue::Number(_)
        | TemplateValue::Array(_)
        | TemplateValue::Object(_) => rendered.push_str(&value.to_string()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateError {
    EmptySlot,
    UnclosedSlot,
    MissingValue(String),
}

impl CodedError for TemplateError {
    fn code(&self) -> ErrorCode {
        match self {
            Self::EmptySlot => EMPTY_SLOT,
            Self::UnclosedSlot => UNCLOSED_SLOT,
            Self::MissingValue(_) => MISSING_VALUE,
        }
    }
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySlot => f.write_str("template path must not be empty"),
            Self::UnclosedSlot => f.write_str("template placeholder is not closed"),
            Self::MissingValue(path) => write!(f, "missing template value: {path}"),
        }
    }
}

impl Error for TemplateError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn context(value: TemplateValue) -> TemplateContext<String, TemplateValue> {
        value.as_object().unwrap().clone()
    }

    #[test]
    fn renders_established_single_brace_dotted_mapping_paths() {
        let template =
            Template::parse("Provider {provider.name} resumed {session.provider-session-id}.")
                .unwrap();
        let values = context(json!({
            "provider": {"name": "local"},
            "session": {"provider-session-id": "abc-123"}
        }));

        assert_eq!(
            template.render(&values).unwrap(),
            "Provider local resumed abc-123."
        );
    }

    #[test]
    fn mappings_sequences_scalars_and_null_have_deterministic_text_semantics() {
        let template = Template::parse("{object}|{list}|{count}|{enabled}|{nothing}").unwrap();
        let values = context(json!({
            "object": {"a": 1},
            "list": [1, 2],
            "count": 3,
            "enabled": true,
            "nothing": null
        }));

        assert_eq!(template.render(&values).unwrap(), "{\"a\":1}|[1,2]|3|true|");
    }

    #[test]
    fn parser_and_missing_paths_have_distinct_stable_codes() {
        assert_eq!(
            Template::parse("{}").unwrap_err().code().as_str(),
            "VAL-TEMPLATE-001"
        );
        assert_eq!(
            Template::parse("{name").unwrap_err().code().as_str(),
            "VAL-TEMPLATE-002"
        );

        let template = Template::parse("{profile.name}").unwrap();
        let error = template.render(&TemplateContext::new()).unwrap_err();
        assert_eq!(
            error,
            TemplateError::MissingValue("profile.name".to_owned())
        );
        assert_eq!(error.code().as_str(), "RES-TEMPLATE-001");
    }

    #[test]
    fn non_mapping_intermediate_values_are_not_traversed() {
        let template = Template::parse("{session.id}").unwrap();
        let values = context(json!({"session": "opaque-object"}));

        assert_eq!(
            template.render(&values).unwrap_err(),
            TemplateError::MissingValue("session.id".to_owned())
        );
    }
}
