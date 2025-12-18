use crate::state::{ActionRequest, ActionResult, ActionType};
use crate::uia::{click_element, get_element_text, set_element_value, FoundElement};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Action execution outcome with state tracking
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionOutcome {
    pub success: bool,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_state: Option<String>,
}

impl ActionOutcome {
    /// Create a successful outcome
    pub fn success(action: &str) -> Self {
        Self {
            success: true,
            action: action.to_string(),
            error: None,
            pre_state: None,
            post_state: None,
        }
    }

    /// Create a successful outcome with state information
    pub fn success_with_state(
        action: &str,
        pre_state: Option<String>,
        post_state: Option<String>,
    ) -> Self {
        Self {
            success: true,
            action: action.to_string(),
            error: None,
            pre_state,
            post_state,
        }
    }

    /// Create a failed outcome
    pub fn failure(action: &str, error: impl Into<String>) -> Self {
        Self {
            success: false,
            action: action.to_string(),
            error: Some(error.into()),
            pre_state: None,
            post_state: None,
        }
    }
}

/// Execute an action request on a found element
#[allow(dead_code)]
pub fn execute_action(request: &ActionRequest, element: &FoundElement) -> Result<ActionResult> {
    // Execute the specific action
    let outcome = match request.action {
        ActionType::Click => execute_click(element),
        ActionType::Invoke => execute_invoke(element),
        ActionType::SetValue => {
            let value = request
                .value
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("SetValue requires a value"))?;
            execute_set_value(element, value)
        }
    };

    // Convert ActionOutcome to ActionResult
    let outcome = outcome?;
    let action_result = if outcome.success {
        ActionResult::success(element.info.clone())
    } else {
        ActionResult::failure(outcome.error.unwrap_or_default())
    };

    Ok(action_result)
}

/// Execute a click action on the element
#[allow(dead_code)]
pub fn execute_click(element: &FoundElement) -> Result<ActionOutcome> {
    match click_element(element) {
        Ok(()) => Ok(ActionOutcome::success("click")),
        Err(e) => Ok(ActionOutcome::failure("click", e.to_string())),
    }
}

/// Execute an invoke action on the element (alias for click)
#[allow(dead_code)]
pub fn execute_invoke(element: &FoundElement) -> Result<ActionOutcome> {
    match click_element(element) {
        Ok(()) => Ok(ActionOutcome::success("invoke")),
        Err(e) => Ok(ActionOutcome::failure("invoke", e.to_string())),
    }
}

/// Execute a set value action on the element
#[allow(dead_code)]
pub fn execute_set_value(element: &FoundElement, value: &str) -> Result<ActionOutcome> {
    // Get pre-state
    let pre_state = get_element_text(element).ok();

    // Set the value
    let result = set_element_value(element, value);

    // Get post-state for verification
    let post_state = get_element_text(element).ok();

    match result {
        Ok(()) => {
            // Verify the value was actually set (if possible)
            if let Some(ref actual) = post_state {
                if actual == value {
                    Ok(ActionOutcome::success_with_state(
                        "set_value",
                        pre_state,
                        post_state,
                    ))
                } else {
                    Ok(ActionOutcome::failure(
                        "set_value",
                        format!(
                            "Value verification failed: expected '{}', got '{}'",
                            value, actual
                        ),
                    ))
                }
            } else {
                // Can't verify, assume success
                Ok(ActionOutcome::success_with_state(
                    "set_value",
                    pre_state,
                    post_state,
                ))
            }
        }
        Err(e) => Ok(ActionOutcome::failure("set_value", e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{MatchedElement, Selector};

    // Helper to create a mock FoundElement for testing (non-Windows only)
    fn create_mock_element() -> FoundElement {
        #[cfg(not(target_os = "windows"))]
        {
            FoundElement {
                info: MatchedElement {
                    name: "Test Button".to_string(),
                    control_type: "Button".to_string(),
                    automation_id: "test_btn".to_string(),
                },
            }
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, we can't create a real mock without complex setup
            // Tests for Windows-specific functionality should be integration tests
            panic!("Mock element creation not supported on Windows in unit tests");
        }
    }

    #[test]
    fn test_action_outcome_success() {
        let outcome = ActionOutcome::success("test_action");
        assert!(outcome.success);
        assert_eq!(outcome.action, "test_action");
        assert!(outcome.error.is_none());
        assert!(outcome.pre_state.is_none());
        assert!(outcome.post_state.is_none());
    }

    #[test]
    fn test_action_outcome_success_with_state() {
        let pre = "old_value".to_string();
        let post = "new_value".to_string();
        let outcome =
            ActionOutcome::success_with_state("test_action", Some(pre.clone()), Some(post.clone()));

        assert!(outcome.success);
        assert_eq!(outcome.action, "test_action");
        assert!(outcome.error.is_none());
        assert_eq!(outcome.pre_state, Some(pre));
        assert_eq!(outcome.post_state, Some(post));
    }

    #[test]
    fn test_action_outcome_failure() {
        let outcome = ActionOutcome::failure("test_action", "Something went wrong");
        assert!(!outcome.success);
        assert_eq!(outcome.action, "test_action");
        assert_eq!(outcome.error, Some("Something went wrong".to_string()));
        assert!(outcome.pre_state.is_none());
        assert!(outcome.post_state.is_none());
    }

    #[test]
    fn test_action_outcome_serialize() {
        let outcome = ActionOutcome::success_with_state(
            "set_value",
            Some("old".to_string()),
            Some("new".to_string()),
        );
        let json = serde_json::to_string(&outcome).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"set_value\""));
        assert!(json.contains("\"old\""));
        assert!(json.contains("\"new\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_action_outcome_failure_serialize() {
        let outcome = ActionOutcome::failure("click", "Element not clickable");
        let json = serde_json::to_string(&outcome).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"click\""));
        assert!(json.contains("\"Element not clickable\""));
        assert!(!json.contains("\"pre_state\""));
        assert!(!json.contains("\"post_state\""));
    }

    #[test]
    fn test_execute_click_not_supported_on_non_windows() {
        #[cfg(not(target_os = "windows"))]
        {
            let element = create_mock_element();
            let result = execute_click(&element);
            assert!(!result.unwrap().success);
        }

        #[cfg(target_os = "windows")]
        {
            // Can't test easily without real Windows UI elements
            // This would need to be an integration test
        }
    }

    #[test]
    fn test_execute_invoke_not_supported_on_non_windows() {
        #[cfg(not(target_os = "windows"))]
        {
            let element = create_mock_element();
            let result = execute_invoke(&element);
            assert!(!result.unwrap().success);
        }

        #[cfg(target_os = "windows")]
        {
            // Can't test easily without real Windows UI elements
            // This would need to be an integration test
        }
    }

    #[test]
    fn test_execute_set_value_not_supported_on_non_windows() {
        #[cfg(not(target_os = "windows"))]
        {
            let element = create_mock_element();
            let result = execute_set_value(&element, "test value");
            assert!(!result.unwrap().success);
        }

        #[cfg(target_os = "windows")]
        {
            // Can't test easily without real Windows UI elements
            // This would need to be an integration test
        }
    }

    #[test]
    fn test_execute_action_click() {
        let request = ActionRequest {
            action: ActionType::Click,
            selector: Selector {
                name: Some("OK".to_string()),
                ..Default::default()
            },
            value: None,
        };

        #[cfg(not(target_os = "windows"))]
        {
            let element = create_mock_element();
            let result = execute_action(&request, &element);
            assert!(result.is_ok());
            assert!(!result.unwrap().success);
        }

        #[cfg(target_os = "windows")]
        {
            // Can't test easily without real Windows UI elements
            // This would need to be an integration test
        }
    }

    #[test]
    fn test_execute_action_set_value() {
        let request = ActionRequest {
            action: ActionType::SetValue,
            selector: Selector {
                control_type: Some("Edit".to_string()),
                ..Default::default()
            },
            value: Some("Hello World".to_string()),
        };

        #[cfg(not(target_os = "windows"))]
        {
            let element = create_mock_element();
            let result = execute_action(&request, &element);
            assert!(result.is_ok());
            assert!(!result.unwrap().success);
        }

        #[cfg(target_os = "windows")]
        {
            // Can't test easily without real Windows UI elements
            // This would need to be an integration test
        }
    }

    #[test]
    fn test_execute_action_set_value_without_value_errors() {
        let request = ActionRequest {
            action: ActionType::SetValue,
            selector: Selector::default(),
            value: None,
        };

        let element = create_mock_element();
        let result = execute_action(&request, &element);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("requires a value"));
    }
}
