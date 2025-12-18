mod selector;
mod state;
mod uia;
mod win32;

use anyhow::Result;
use state::ExecutorOutput;

const MAX_UI_DEPTH: usize = 5;

fn main() -> Result<()> {
    // Initialize COM for UI Automation
    uia::initialize_com()?;

    // Ensure COM cleanup on exit
    let _com_guard = ComGuard;

    // Enumerate all visible windows
    let mut windows = win32::enumerate_windows()?;

    // Get focused window handle
    let focused_handle = win32::get_focused_window()?;

    // Mark the focused window
    if let Some(focused) = focused_handle {
        for window in &mut windows {
            if window.handle == focused {
                window.is_focused = true;
                break;
            }
        }
    }

    // Capture UI tree for focused window
    let focused_window_ui = if let Some(focused) = focused_handle {
        uia::capture_ui_tree(focused, MAX_UI_DEPTH)?
    } else {
        None
    };

    // Build output
    let output = ExecutorOutput {
        windows,
        focused_window_ui,
    };

    // Output JSON to stdout
    let json = serde_json::to_string_pretty(&output)?;
    println!("{}", json);

    Ok(())
}

/// RAII guard for COM cleanup
struct ComGuard;

impl Drop for ComGuard {
    fn drop(&mut self) {
        uia::uninitialize_com();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_ui_depth_is_reasonable() {
        assert!(MAX_UI_DEPTH > 0);
        assert!(MAX_UI_DEPTH < 100);
    }
}
