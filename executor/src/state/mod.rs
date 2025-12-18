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

/// Root output structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutorOutput {
    pub windows: Vec<WindowInfo>,
    pub focused_window_ui: Option<UIElement>,
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
}
