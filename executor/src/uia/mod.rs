use crate::state::UIElement;
use anyhow::Result;

/// Initialize COM for UI Automation
pub fn initialize_com() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        initialize_com_impl()
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn initialize_com_impl() -> Result<()> {
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
    }

    Ok(())
}

/// Uninitialize COM
pub fn uninitialize_com() {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Com::CoUninitialize;
        unsafe {
            CoUninitialize();
        }
    }
}

/// Capture UI Automation tree for a window
pub fn capture_ui_tree(window_handle: u64, max_depth: usize) -> Result<Option<UIElement>> {
    #[cfg(target_os = "windows")]
    {
        capture_ui_tree_impl(window_handle, max_depth)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (window_handle, max_depth);
        Ok(None)
    }
}

#[cfg(target_os = "windows")]
fn capture_ui_tree_impl(window_handle: u64, max_depth: usize) -> Result<Option<UIElement>> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
    use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation};

    unsafe {
        let automation: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL)?;

        let hwnd = HWND(window_handle as isize as *mut _);
        let element = automation.ElementFromHandle(hwnd)?;

        let root = traverse_element(&automation, &element, 0, max_depth)?;
        Ok(Some(root))
    }
}

#[cfg(target_os = "windows")]
unsafe fn traverse_element(
    automation: &windows::Win32::UI::Accessibility::IUIAutomation,
    element: &windows::Win32::UI::Accessibility::IUIAutomationElement,
    current_depth: usize,
    max_depth: usize,
) -> Result<UIElement> {
    let name = element.CurrentName()?.to_string();
    let automation_id = element.CurrentAutomationId()?.to_string();

    let control_type_id = element.CurrentControlType()?;
    let control_type = format!("ControlType_{}", control_type_id.0);

    let mut children = Vec::new();

    if current_depth < max_depth {
        let walker = automation.ControlViewWalker()?;

        if let Ok(child) = walker.GetFirstChildElement(element) {
            let mut current_child = Some(child);

            while let Some(child_elem) = current_child {
                if let Ok(child_ui) =
                    traverse_element(automation, &child_elem, current_depth + 1, max_depth)
                {
                    children.push(child_ui);
                }

                current_child = walker.GetNextSiblingElement(&child_elem).ok();
            }
        }
    }

    Ok(UIElement {
        name,
        control_type,
        automation_id,
        children,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_com_returns_ok() {
        let result = initialize_com();
        assert!(result.is_ok());
    }

    #[test]
    fn test_capture_ui_tree_returns_result() {
        // On non-Windows, always returns Ok(None)
        // On Windows, handle 0 is invalid so may return Err
        let result = capture_ui_tree(0, 3);
        #[cfg(not(target_os = "windows"))]
        assert!(result.is_ok());
        #[cfg(target_os = "windows")]
        let _ = result; // Windows may fail with invalid handle - that's correct
    }
}
