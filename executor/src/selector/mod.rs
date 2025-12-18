use crate::state::Selector;
use anyhow::{bail, Result};

/// Supported control types for selector matching
#[allow(dead_code)]
const VALID_CONTROL_TYPES: &[&str] = &[
    "Button", "Edit", "CheckBox", "ComboBox", "List", "ListItem", "Tree", "TreeItem", "Menu",
    "MenuItem", "Tab", "TabItem", "Text", "Image", "Window",
];

/// Parse a selector string into a Selector struct.
///
/// Syntax: `key="value"` pairs separated by spaces.
/// Supported keys: name, automation_id, control_type, index
///
/// # Examples
/// ```ignore
/// parse(r#"name="OK" control_type="Button""#)
/// parse(r#"automation_id="btn_submit""#)
/// parse(r#"control_type="Edit" index=0"#)
/// ```
#[allow(dead_code)]
pub fn parse(input: &str) -> Result<Selector> {
    let input = input.trim();

    if input.is_empty() {
        bail!("selector cannot be empty");
    }

    let mut selector = Selector::default();
    let mut has_criteria = false;

    // Parse key=value or key="value" pairs
    let mut chars = input.chars().peekable();

    while chars.peek().is_some() {
        // Skip whitespace
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }

        if chars.peek().is_none() {
            break;
        }

        // Read key
        let key: String = chars
            .by_ref()
            .take_while(|c| *c != '=')
            .collect::<String>()
            .trim()
            .to_string();

        if key.is_empty() {
            bail!("invalid selector syntax: missing key");
        }

        // Read value
        let value = parse_value(&mut chars)?;

        // Apply to selector
        match key.as_str() {
            "name" => {
                selector.name = Some(value);
                has_criteria = true;
            }
            "automation_id" => {
                selector.automation_id = Some(value);
                has_criteria = true;
            }
            "control_type" => {
                validate_control_type(&value)?;
                selector.control_type = Some(value);
                has_criteria = true;
            }
            "index" => {
                let idx: usize = value
                    .parse()
                    .map_err(|_| anyhow::anyhow!("index must be a non-negative integer"))?;
                selector.index = Some(idx);
                has_criteria = true;
            }
            _ => bail!("unsupported selector key: '{}'", key),
        }
    }

    if !has_criteria {
        bail!("selector must have at least one criterion");
    }

    Ok(selector)
}

#[allow(dead_code)]
fn parse_value(chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<String> {
    // Check if value is quoted
    if chars.peek() == Some(&'"') {
        chars.next(); // consume opening quote
        let mut value = String::new();
        let mut escaped = false;

        for c in chars.by_ref() {
            if escaped {
                value.push(c);
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                return Ok(value);
            } else {
                value.push(c);
            }
        }
        bail!("unterminated string in selector");
    } else {
        // Unquoted value (for index)
        let value: String = chars.by_ref().take_while(|c| !c.is_whitespace()).collect();

        if value.is_empty() {
            bail!("invalid selector syntax: missing value");
        }

        Ok(value)
    }
}

#[allow(dead_code)]
fn validate_control_type(value: &str) -> Result<()> {
    if !VALID_CONTROL_TYPES.contains(&value) {
        bail!(
            "invalid control_type '{}'. Valid types: {}",
            value,
            VALID_CONTROL_TYPES.join(", ")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_name_only() {
        let selector = parse(r#"name="OK""#).unwrap();
        assert_eq!(selector.name, Some("OK".to_string()));
        assert!(selector.automation_id.is_none());
        assert!(selector.control_type.is_none());
        assert!(selector.index.is_none());
    }

    #[test]
    fn test_parse_automation_id_only() {
        let selector = parse(r#"automation_id="btn_submit""#).unwrap();
        assert!(selector.name.is_none());
        assert_eq!(selector.automation_id, Some("btn_submit".to_string()));
    }

    #[test]
    fn test_parse_control_type_only() {
        let selector = parse(r#"control_type="Button""#).unwrap();
        assert_eq!(selector.control_type, Some("Button".to_string()));
    }

    #[test]
    fn test_parse_index_only() {
        let selector = parse(r#"index=0"#).unwrap();
        assert_eq!(selector.index, Some(0));
    }

    #[test]
    fn test_parse_multiple_criteria() {
        let selector = parse(r#"name="Submit" control_type="Button""#).unwrap();
        assert_eq!(selector.name, Some("Submit".to_string()));
        assert_eq!(selector.control_type, Some("Button".to_string()));
        assert!(selector.automation_id.is_none());
        assert!(selector.index.is_none());
    }

    #[test]
    fn test_parse_all_criteria() {
        let selector =
            parse(r#"name="OK" automation_id="btn_ok" control_type="Button" index=2"#).unwrap();
        assert_eq!(selector.name, Some("OK".to_string()));
        assert_eq!(selector.automation_id, Some("btn_ok".to_string()));
        assert_eq!(selector.control_type, Some("Button".to_string()));
        assert_eq!(selector.index, Some(2));
    }

    #[test]
    fn test_parse_with_spaces_in_value() {
        let selector = parse(r#"name="Save As...""#).unwrap();
        assert_eq!(selector.name, Some("Save As...".to_string()));
    }

    #[test]
    fn test_parse_with_escaped_quote() {
        let selector = parse(r#"name="Say \"Hello\"""#).unwrap();
        assert_eq!(selector.name, Some(r#"Say "Hello""#.to_string()));
    }

    #[test]
    fn test_parse_empty_string_fails() {
        let result = parse("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_parse_whitespace_only_fails() {
        let result = parse("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_control_type_fails() {
        let result = parse(r#"control_type="InvalidType""#);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid control_type"));
    }

    #[test]
    fn test_parse_unsupported_key_fails() {
        let result = parse(r#"class_name="MyClass""#);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unsupported"));
    }

    #[test]
    fn test_parse_invalid_index_fails() {
        let result = parse(r#"index=abc"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("integer"));
    }

    #[test]
    fn test_parse_negative_index_fails() {
        let result = parse(r#"index=-1"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unterminated_string_fails() {
        let result = parse(r#"name="unterminated"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unterminated"));
    }

    #[test]
    fn test_parse_all_control_types_valid() {
        for ct in VALID_CONTROL_TYPES {
            let selector = parse(&format!(r#"control_type="{}""#, ct)).unwrap();
            assert_eq!(selector.control_type, Some(ct.to_string()));
        }
    }

    #[test]
    fn test_parse_extra_whitespace_trimmed() {
        let selector = parse(r#"  name="OK"   control_type="Button"  "#).unwrap();
        assert_eq!(selector.name, Some("OK".to_string()));
        assert_eq!(selector.control_type, Some("Button".to_string()));
    }
}
