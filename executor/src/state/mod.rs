use serde::{Deserialize, Serialize};

/// Represents a window in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub handle: u64,
    pub title: String,
    pub is_visible: bool,
    pub is_focused: bool,
}

/// Represents a UI Automation element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIElement {
    pub name: String,
    pub control_type: String,
    pub automation_id: String,
    pub children: Vec<UIElement>,
}

/// Root output structure for observe mode (v0)
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutorOutput {
    pub windows: Vec<WindowInfo>,
    pub focused_window_ui: Option<UIElement>,
}

// ============================================================================
// v1 Models - Action Execution
// NOTE: These types are not yet used in main.rs but will be consumed in Phase 5
// ============================================================================

/// Supported action types for v1
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Click,
    Invoke,
    SetValue,
}

/// Selector criteria for targeting UI elements
#[allow(dead_code)]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selector {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub control_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,
}

impl Selector {
    /// Returns true if the selector has no criteria set
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.automation_id.is_none()
            && self.control_type.is_none()
            && self.index.is_none()
    }
}

/// Action request received via stdin (v1)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    pub action: ActionType,
    pub selector: Selector,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Information about the matched element (included in success responses)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedElement {
    pub name: String,
    pub control_type: String,
    pub automation_id: String,
}

/// Action result returned via stdout (v1)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<MatchedElement>,
}

impl ActionResult {
    /// Create a successful result with element info
    #[allow(dead_code)]
    pub fn success(element: MatchedElement) -> Self {
        Self {
            success: true,
            error: None,
            element: Some(element),
        }
    }

    /// Create a failure result with error message
    #[allow(dead_code)]
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            error: Some(error.into()),
            element: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_info_serialize() {
        let window = WindowInfo {
            handle: 12345,
            title: "Test Window".to_string(),
            is_visible: true,
            is_focused: false,
        };
        let json = serde_json::to_string(&window).unwrap();
        assert!(json.contains("Test Window"));
    }

    #[test]
    fn test_executor_output_serialize() {
        let output = ExecutorOutput {
            windows: vec![],
            focused_window_ui: None,
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("windows"));
    }

    // ========================================================================
    // v1 Model Tests
    // ========================================================================

    #[test]
    fn test_action_type_serialize_snake_case() {
        assert_eq!(
            serde_json::to_string(&ActionType::Click).unwrap(),
            "\"click\""
        );
        assert_eq!(
            serde_json::to_string(&ActionType::Invoke).unwrap(),
            "\"invoke\""
        );
        assert_eq!(
            serde_json::to_string(&ActionType::SetValue).unwrap(),
            "\"set_value\""
        );
    }

    #[test]
    fn test_action_type_deserialize_snake_case() {
        assert_eq!(
            serde_json::from_str::<ActionType>("\"click\"").unwrap(),
            ActionType::Click
        );
        assert_eq!(
            serde_json::from_str::<ActionType>("\"invoke\"").unwrap(),
            ActionType::Invoke
        );
        assert_eq!(
            serde_json::from_str::<ActionType>("\"set_value\"").unwrap(),
            ActionType::SetValue
        );
    }

    #[test]
    fn test_selector_default_is_empty() {
        let selector = Selector::default();
        assert!(selector.is_empty());
    }

    #[test]
    fn test_selector_with_name_not_empty() {
        let selector = Selector {
            name: Some("OK".to_string()),
            ..Default::default()
        };
        assert!(!selector.is_empty());
    }

    #[test]
    fn test_selector_with_automation_id_not_empty() {
        let selector = Selector {
            automation_id: Some("btn_ok".to_string()),
            ..Default::default()
        };
        assert!(!selector.is_empty());
    }

    #[test]
    fn test_selector_with_control_type_not_empty() {
        let selector = Selector {
            control_type: Some("Button".to_string()),
            ..Default::default()
        };
        assert!(!selector.is_empty());
    }

    #[test]
    fn test_selector_with_index_not_empty() {
        let selector = Selector {
            index: Some(0),
            ..Default::default()
        };
        assert!(!selector.is_empty());
    }

    #[test]
    fn test_selector_serialization_omits_none() {
        let selector = Selector {
            name: Some("Submit".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&selector).unwrap();
        assert!(json.contains("name"));
        assert!(!json.contains("automation_id"));
        assert!(!json.contains("control_type"));
        assert!(!json.contains("index"));
    }

    #[test]
    fn test_action_request_click_serialize() {
        let request = ActionRequest {
            action: ActionType::Click,
            selector: Selector {
                name: Some("OK".to_string()),
                control_type: Some("Button".to_string()),
                ..Default::default()
            },
            value: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"click\""));
        assert!(json.contains("\"OK\""));
        assert!(json.contains("\"Button\""));
        assert!(!json.contains("value"));
    }

    #[test]
    fn test_action_request_set_value_serialize() {
        let request = ActionRequest {
            action: ActionType::SetValue,
            selector: Selector {
                automation_id: Some("txtInput".to_string()),
                ..Default::default()
            },
            value: Some("Hello World".to_string()),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"set_value\""));
        assert!(json.contains("\"txtInput\""));
        assert!(json.contains("\"Hello World\""));
    }

    #[test]
    fn test_action_request_deserialize() {
        let json = r#"{"action":"click","selector":{"name":"OK","control_type":"Button"}}"#;
        let request: ActionRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.action, ActionType::Click);
        assert_eq!(request.selector.name, Some("OK".to_string()));
        assert_eq!(request.selector.control_type, Some("Button".to_string()));
        assert!(request.selector.automation_id.is_none());
        assert!(request.value.is_none());
    }

    #[test]
    fn test_action_result_success() {
        let element = MatchedElement {
            name: "OK".to_string(),
            control_type: "Button".to_string(),
            automation_id: "btn_ok".to_string(),
        };
        let result = ActionResult::success(element);
        assert!(result.success);
        assert!(result.error.is_none());
        assert!(result.element.is_some());
    }

    #[test]
    fn test_action_result_failure() {
        let result = ActionResult::failure("Element not found");
        assert!(!result.success);
        assert_eq!(result.error, Some("Element not found".to_string()));
        assert!(result.element.is_none());
    }

    #[test]
    fn test_action_result_success_serialize() {
        let element = MatchedElement {
            name: "Submit".to_string(),
            control_type: "Button".to_string(),
            automation_id: "".to_string(),
        };
        let result = ActionResult::success(element);
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(!json.contains("\"error\""));
        assert!(json.contains("\"element\""));
    }

    #[test]
    fn test_action_result_failure_serialize() {
        let result = ActionResult::failure("Pattern not supported");
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"Pattern not supported\""));
        assert!(!json.contains("\"element\""));
    }

    #[test]
    fn test_matched_element_serialize() {
        let element = MatchedElement {
            name: "File".to_string(),
            control_type: "MenuItem".to_string(),
            automation_id: "menu_file".to_string(),
        };
        let json = serde_json::to_string(&element).unwrap();
        assert!(json.contains("\"File\""));
        assert!(json.contains("\"MenuItem\""));
        assert!(json.contains("\"menu_file\""));
    }
}
