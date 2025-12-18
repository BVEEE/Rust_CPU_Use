#![cfg(target_os = "windows")]

use crate::state::{MatchedElement, Selector, UIElement};
use anyhow::{bail, Result};

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

// ============================================================================
// v1 Functions - Element Search and Pattern Helpers
// ============================================================================

/// Find an element matching the selector in the focused window
#[allow(dead_code)]
pub fn find_element(window_handle: u64, selector: &Selector) -> Result<FoundElement> {
    #[cfg(target_os = "windows")]
    {
        find_element_impl(window_handle, selector)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (window_handle, selector);
        bail!("element search only supported on Windows")
    }
}

/// Opaque handle to a found UI Automation element (Windows-only)
#[allow(dead_code)]
pub struct FoundElement {
    #[cfg(target_os = "windows")]
    inner: windows::Win32::UI::Accessibility::IUIAutomationElement,
    /// Element metadata for response
    pub info: MatchedElement,
}

#[cfg(target_os = "windows")]
fn find_element_impl(window_handle: u64, selector: &Selector) -> Result<FoundElement> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
    use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation, IUIAutomationElement};

    if selector.is_empty() {
        bail!("selector cannot be empty");
    }

    unsafe {
        let automation: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL)?;
        let hwnd = HWND(window_handle as isize as *mut _);
        let root = automation.ElementFromHandle(hwnd)?;

        let mut matches: Vec<IUIAutomationElement> = Vec::new();
        search_element(&automation, &root, selector, &mut matches)?;

        if matches.is_empty() {
            bail!("no element found matching selector");
        }

        // Apply index filter if specified
        let element = if let Some(idx) = selector.index {
            if idx >= matches.len() {
                bail!(
                    "index {} out of range (found {} matching elements)",
                    idx,
                    matches.len()
                );
            }
            matches.remove(idx)
        } else if matches.len() > 1 {
            bail!(
                "selector matched {} elements; use index to disambiguate",
                matches.len()
            );
        } else {
            matches.remove(0)
        };

        let info = MatchedElement {
            name: element.CurrentName()?.to_string(),
            control_type: get_control_type_name(element.CurrentControlType()?.0),
            automation_id: element.CurrentAutomationId()?.to_string(),
        };

        Ok(FoundElement {
            inner: element,
            info,
        })
    }
}

#[cfg(target_os = "windows")]
unsafe fn search_element(
    automation: &windows::Win32::UI::Accessibility::IUIAutomation,
    element: &windows::Win32::UI::Accessibility::IUIAutomationElement,
    selector: &Selector,
    matches: &mut Vec<windows::Win32::UI::Accessibility::IUIAutomationElement>,
) -> Result<()> {
    // Check if current element matches selector (excluding index)
    if element_matches(element, selector)? {
        matches.push(element.clone());
    }

    // Recurse into children
    let walker = automation.ControlViewWalker()?;
    if let Ok(child) = walker.GetFirstChildElement(element) {
        let mut current_child = Some(child);
        while let Some(child_elem) = current_child {
            search_element(automation, &child_elem, selector, matches)?;
            current_child = walker.GetNextSiblingElement(&child_elem).ok();
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
unsafe fn element_matches(
    element: &windows::Win32::UI::Accessibility::IUIAutomationElement,
    selector: &Selector,
) -> Result<bool> {
    // Check name
    if let Some(ref expected_name) = selector.name {
        let actual_name = element.CurrentName()?.to_string();
        if &actual_name != expected_name {
            return Ok(false);
        }
    }

    // Check automation_id
    if let Some(ref expected_id) = selector.automation_id {
        let actual_id = element.CurrentAutomationId()?.to_string();
        if &actual_id != expected_id {
            return Ok(false);
        }
    }

    // Check control_type
    if let Some(ref expected_type) = selector.control_type {
        let actual_type_id = element.CurrentControlType()?.0;
        let actual_type_name = get_control_type_name(actual_type_id);
        if &actual_type_name != expected_type {
            return Ok(false);
        }
    }

    // index is handled at collection level, not per-element
    Ok(true)
}

/// Map UI Automation control type ID to human-readable name
#[cfg(target_os = "windows")]
fn get_control_type_name(id: i32) -> String {
    match id {
        50000 => "Button".to_string(),
        50001 => "Calendar".to_string(),
        50002 => "CheckBox".to_string(),
        50003 => "ComboBox".to_string(),
        50004 => "Edit".to_string(),
        50005 => "Hyperlink".to_string(),
        50006 => "Image".to_string(),
        50007 => "ListItem".to_string(),
        50008 => "List".to_string(),
        50009 => "Menu".to_string(),
        50010 => "MenuBar".to_string(),
        50011 => "MenuItem".to_string(),
        50012 => "ProgressBar".to_string(),
        50013 => "RadioButton".to_string(),
        50014 => "ScrollBar".to_string(),
        50015 => "Slider".to_string(),
        50016 => "Spinner".to_string(),
        50017 => "StatusBar".to_string(),
        50018 => "Tab".to_string(),
        50019 => "TabItem".to_string(),
        50020 => "Text".to_string(),
        50021 => "ToolBar".to_string(),
        50022 => "ToolTip".to_string(),
        50023 => "Tree".to_string(),
        50024 => "TreeItem".to_string(),
        50025 => "Custom".to_string(),
        50026 => "Group".to_string(),
        50027 => "Thumb".to_string(),
        50028 => "DataGrid".to_string(),
        50029 => "DataItem".to_string(),
        50030 => "Document".to_string(),
        50031 => "SplitButton".to_string(),
        50032 => "Window".to_string(),
        50033 => "Pane".to_string(),
        50034 => "Header".to_string(),
        50035 => "HeaderItem".to_string(),
        50036 => "Table".to_string(),
        50037 => "TitleBar".to_string(),
        50038 => "Separator".to_string(),
        _ => format!("ControlType_{}", id),
    }
}

/// Invoke the element using InvokePattern
#[allow(dead_code)]
pub fn invoke_element(found: &FoundElement) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        invoke_element_impl(found)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = found;
        bail!("invoke only supported on Windows")
    }
}

#[cfg(target_os = "windows")]
fn invoke_element_impl(found: &FoundElement) -> Result<()> {
    use windows::Win32::UI::Accessibility::{IUIAutomationInvokePattern, UIA_InvokePatternId};

    unsafe {
        let pattern: Result<IUIAutomationInvokePattern, _> =
            found.inner.GetCurrentPatternAs(UIA_InvokePatternId);

        match pattern {
            Ok(invoke_pattern) => {
                invoke_pattern.Invoke()?;
                Ok(())
            }
            Err(_) => {
                // Try legacy accessible fallback
                invoke_via_legacy(&found.inner)
            }
        }
    }
}

#[cfg(target_os = "windows")]
unsafe fn invoke_via_legacy(
    element: &windows::Win32::UI::Accessibility::IUIAutomationElement,
) -> Result<()> {
    use windows::Win32::UI::Accessibility::{
        IUIAutomationLegacyIAccessiblePattern, UIA_LegacyIAccessiblePatternId,
    };

    let pattern: IUIAutomationLegacyIAccessiblePattern = element
        .GetCurrentPatternAs(UIA_LegacyIAccessiblePatternId)
        .map_err(|_| {
            anyhow::anyhow!("element does not support Invoke or LegacyIAccessible pattern")
        })?;

    pattern.DoDefaultAction()?;
    Ok(())
}

/// Set value on the element using ValuePattern
#[allow(dead_code)]
pub fn set_element_value(found: &FoundElement, value: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        set_element_value_impl(found, value)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (found, value);
        bail!("set_value only supported on Windows")
    }
}

#[cfg(target_os = "windows")]
fn set_element_value_impl(found: &FoundElement, value: &str) -> Result<()> {
    use windows::core::BSTR;
    use windows::Win32::UI::Accessibility::{IUIAutomationValuePattern, UIA_ValuePatternId};

    unsafe {
        let pattern = found
            .inner
            .GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
            .map_err(|_| anyhow::anyhow!("element does not support ValuePattern"))?;

        let bstr = BSTR::from(value);
        pattern.SetValue(&bstr)?;
        Ok(())
    }
}

/// Click the element using InvokePattern or fallback methods
#[allow(dead_code)]
pub fn click_element(found: &FoundElement) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        click_element_impl(found)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = found;
        bail!("click only supported on Windows")
    }
}

#[cfg(target_os = "windows")]
fn click_element_impl(found: &FoundElement) -> Result<()> {
    use windows::Win32::UI::Accessibility::{
        IUIAutomationInvokePattern, IUIAutomationLegacyIAccessiblePattern, UIA_InvokePatternId,
        UIA_LegacyIAccessiblePatternId,
    };

    unsafe {
        // First try InvokePattern (most common for buttons)
        if let Ok(invoke) = found
            .inner
            .GetCurrentPatternAs::<IUIAutomationInvokePattern>(UIA_InvokePatternId)
        {
            invoke.Invoke()?;
            return Ok(());
        }

        // Fallback to LegacyIAccessible pattern
        if let Ok(legacy) = found
            .inner
            .GetCurrentPatternAs::<IUIAutomationLegacyIAccessiblePattern>(
                UIA_LegacyIAccessiblePatternId,
            )
        {
            legacy.DoDefaultAction()?;
            return Ok(());
        }

        bail!("element does not support Invoke or LegacyIAccessible pattern");
    }
}

/// Get text from the element using TextPattern or ValuePattern
#[allow(dead_code)]
pub fn get_element_text(found: &FoundElement) -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        get_element_text_impl(found)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = found;
        bail!("get_text only supported on Windows")
    }
}

#[cfg(target_os = "windows")]
fn get_element_text_impl(found: &FoundElement) -> Result<String> {
    use windows::core::BSTR;
    use windows::Win32::UI::Accessibility::{
        IUIAutomationTextPattern, IUIAutomationValuePattern, UIA_TextPatternId, UIA_ValuePatternId,
    };

    unsafe {
        // Try TextPattern first (for text controls)
        if let Ok(text_pattern) = found.inner.GetCurrentPatternAs(UIA_TextPatternId) {
            let pattern: IUIAutomationTextPattern = text_pattern;
            if let Ok(range) = pattern.DocumentRange() {
                if let Ok(text) = range.GetText(-1) {
                    return Ok(text.to_string());
                }
            }
        }

        // Fallback to ValuePattern (for edit controls)
        if let Ok(value_pattern) = found.inner.GetCurrentPatternAs(UIA_ValuePatternId) {
            let pattern: IUIAutomationValuePattern = value_pattern;
            let value = pattern.CurrentValue()?;
            return Ok(value.to_string());
        }

        // Final fallback to the element's name property
        let name = found.inner.CurrentName()?;
        Ok(name.to_string())
    }
}

/// Check if the element is enabled
#[allow(dead_code)]
pub fn is_element_enabled(found: &FoundElement) -> Result<bool> {
    #[cfg(target_os = "windows")]
    {
        is_element_enabled_impl(found)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = found;
        bail!("is_enabled only supported on Windows")
    }
}

#[cfg(target_os = "windows")]
fn is_element_enabled_impl(found: &FoundElement) -> Result<bool> {
    unsafe {
        let is_enabled = found.inner.CurrentIsEnabled()?;
        Ok(is_enabled.into())
    }
}

/// Check if the element is visible
#[allow(dead_code)]
pub fn is_element_visible(found: &FoundElement) -> Result<bool> {
    #[cfg(target_os = "windows")]
    {
        is_element_visible_impl(found)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = found;
        bail!("is_visible only supported on Windows")
    }
}

#[cfg(target_os = "windows")]
fn is_element_visible_impl(found: &FoundElement) -> Result<bool> {
    use windows::Win32::Foundation::BOOL;

    unsafe {
        let is_offscreen = found.inner.CurrentIsOffscreen()?;
        Ok(is_offscreen == BOOL(0))
    }
}

/// Get the bounding rectangle of the element
#[allow(dead_code)]
pub fn get_element_bounds(found: &FoundElement) -> Result<ElementBounds> {
    #[cfg(target_os = "windows")]
    {
        get_element_bounds_impl(found)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = found;
        bail!("get_bounds only supported on Windows")
    }
}

/// Bounding rectangle of a UI element
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ElementBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[cfg(target_os = "windows")]
fn get_element_bounds_impl(found: &FoundElement) -> Result<ElementBounds> {
    unsafe {
        let rect = found.inner.CurrentBoundingRectangle()?;
        Ok(ElementBounds {
            x: rect.left as f64,
            y: rect.top as f64,
            width: (rect.right - rect.left) as f64,
            height: (rect.bottom - rect.top) as f64,
        })
    }
}

/// Toggle a CheckBox element using TogglePattern
#[allow(dead_code)]
pub fn toggle_element(found: &FoundElement) -> Result<ToggleState> {
    #[cfg(target_os = "windows")]
    {
        toggle_element_impl(found)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = found;
        bail!("toggle only supported on Windows")
    }
}

/// Toggle state enumeration
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ToggleState {
    Off,
    On,
    Indeterminate,
}

#[cfg(target_os = "windows")]
fn toggle_element_impl(found: &FoundElement) -> Result<ToggleState> {
    use windows::Win32::UI::Accessibility::{
        IUIAutomationTogglePattern, ToggleState as WinToggleState, UIA_TogglePatternId,
    };

    unsafe {
        let pattern = found
            .inner
            .GetCurrentPatternAs::<IUIAutomationTogglePattern>(UIA_TogglePatternId)
            .map_err(|_| anyhow::anyhow!("element does not support TogglePattern"))?;

        pattern.Toggle()?;

        match pattern.CurrentToggleState()? {
            WinToggleState::Off => Ok(ToggleState::Off),
            WinToggleState::On => Ok(ToggleState::On),
            WinToggleState::Indeterminate => Ok(ToggleState::Indeterminate),
            _ => bail!("unknown toggle state"),
        }
    }
}

/// Expand or collapse a TreeItem/Menu element using ExpandCollapsePattern
#[allow(dead_code)]
pub fn expand_collapse_element(found: &FoundElement, expand: bool) -> Result<ExpandCollapseState> {
    #[cfg(target_os = "windows")]
    {
        expand_collapse_element_impl(found, expand)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (found, expand);
        bail!("expand_collapse only supported on Windows")
    }
}

/// Expand/Collapse state enumeration
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExpandCollapseState {
    Collapsed,
    Expanded,
    PartiallyExpanded,
    LeafNode,
}

#[cfg(target_os = "windows")]
fn expand_collapse_element_impl(found: &FoundElement, expand: bool) -> Result<ExpandCollapseState> {
    use windows::Win32::UI::Accessibility::{
        ExpandCollapseState as WindowsExpandCollapseState, IUIAutomationExpandCollapsePattern,
        UIA_ExpandCollapsePatternId,
    };

    unsafe {
        let pattern: IUIAutomationExpandCollapsePattern = found
            .inner
            .GetCurrentPatternAs(UIA_ExpandCollapsePatternId)
            .map_err(|_| anyhow::anyhow!("element does not support ExpandCollapsePattern"))?;

        if expand {
            pattern.Expand()?;
        } else {
            pattern.Collapse()?;
        }

        let state = pattern.CurrentExpandCollapseState()?;
        match state {
            WindowsExpandCollapseState::ExpandCollapseState_Collapsed => {
                Ok(ExpandCollapseState::Collapsed)
            }
            WindowsExpandCollapseState::ExpandCollapseState_Expanded => {
                Ok(ExpandCollapseState::Expanded)
            }
            WindowsExpandCollapseState::ExpandCollapseState_PartiallyExpanded => {
                Ok(ExpandCollapseState::PartiallyExpanded)
            }
            WindowsExpandCollapseState::ExpandCollapseState_LeafNode => {
                Ok(ExpandCollapseState::LeafNode)
            }
            _ => bail!("unknown expand collapse state"),
        }
    }
}

/// Select an item in a List/Combo using SelectionItemPattern
#[allow(dead_code)]
pub fn select_element(found: &FoundElement) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        select_element_impl(found)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = found;
        bail!("select only supported on Windows")
    }
}

#[cfg(target_os = "windows")]
fn select_element_impl(found: &FoundElement) -> Result<()> {
    use windows::Win32::UI::Accessibility::{
        IUIAutomationSelectionItemPattern, UIA_SelectionItemPatternId,
    };

    unsafe {
        let pattern: IUIAutomationSelectionItemPattern = found
            .inner
            .GetCurrentPatternAs(UIA_SelectionItemPatternId)
            .map_err(|_| anyhow::anyhow!("element does not support SelectionItemPattern"))?;

        pattern.Select()?;
        Ok(())
    }
}

/// Get all selected items from a container using SelectionPattern
#[allow(dead_code)]
pub fn get_selected_items(found: &FoundElement) -> Result<Vec<MatchedElement>> {
    #[cfg(target_os = "windows")]
    {
        get_selected_items_impl(found)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = found;
        bail!("get_selected_items only supported on Windows")
    }
}

#[cfg(target_os = "windows")]
fn get_selected_items_impl(found: &FoundElement) -> Result<Vec<MatchedElement>> {
    use windows::Win32::UI::Accessibility::{
        IUIAutomationElementArray, IUIAutomationSelectionPattern, UIA_SelectionPatternId,
    };

    unsafe {
        let pattern: IUIAutomationSelectionPattern = found
            .inner
            .GetCurrentPatternAs(UIA_SelectionPatternId)
            .map_err(|_| anyhow::anyhow!("element does not support SelectionPattern"))?;

        let selection: IUIAutomationElementArray = pattern.CurrentSelection()?;
        let length = selection.Length()?;
        let mut items = Vec::new();

        for i in 0..length {
            if let Ok(element) = selection.GetElement(i) {
                let item = MatchedElement {
                    name: element.CurrentName()?.to_string(),
                    control_type: get_control_type_name(element.CurrentControlType()?.0),
                    automation_id: element.CurrentAutomationId()?.to_string(),
                };
                items.push(item);
            }
        }

        Ok(items)
    }
}

/// Scroll an element using ScrollPattern
#[allow(dead_code)]
pub fn scroll_element(
    found: &FoundElement,
    horizontal: ScrollAmount,
    vertical: ScrollAmount,
) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        scroll_element_impl(found, horizontal, vertical)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (found, horizontal, vertical);
        bail!("scroll only supported on Windows")
    }
}

/// Scroll amount enumeration
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScrollAmount {
    LargeDecrement,
    SmallDecrement,
    NoAmount,
    LargeIncrement,
    SmallIncrement,
}

#[cfg(target_os = "windows")]
fn scroll_element_impl(
    found: &FoundElement,
    horizontal: ScrollAmount,
    vertical: ScrollAmount,
) -> Result<()> {
    use windows::Win32::UI::Accessibility::{
        IUIAutomationScrollPattern, ScrollAmount as WindowsScrollAmount, UIA_ScrollPatternId,
    };

    unsafe {
        let pattern: IUIAutomationScrollPattern = found
            .inner
            .GetCurrentPatternAs(UIA_ScrollPatternId)
            .map_err(|_| anyhow::anyhow!("element does not support ScrollPattern"))?;

        let h_scroll = match horizontal {
            ScrollAmount::LargeDecrement => WindowsScrollAmount::ScrollAmountLargeDecrement,
            ScrollAmount::SmallDecrement => WindowsScrollAmount::ScrollAmountSmallDecrement,
            ScrollAmount::NoAmount => WindowsScrollAmount::ScrollAmountNoAmount,
            ScrollAmount::LargeIncrement => WindowsScrollAmount::ScrollAmountLargeIncrement,
            ScrollAmount::SmallIncrement => WindowsScrollAmount::ScrollAmountSmallIncrement,
        };

        let v_scroll = match vertical {
            ScrollAmount::LargeDecrement => WindowsScrollAmount::ScrollAmountLargeDecrement,
            ScrollAmount::SmallDecrement => WindowsScrollAmount::ScrollAmountSmallDecrement,
            ScrollAmount::NoAmount => WindowsScrollAmount::ScrollAmountNoAmount,
            ScrollAmount::LargeIncrement => WindowsScrollAmount::ScrollAmountLargeIncrement,
            ScrollAmount::SmallIncrement => WindowsScrollAmount::ScrollAmountSmallIncrement,
        };

        pattern.Scroll(h_scroll, v_scroll)?;
        Ok(())
    }
}

/// Get range value from Slider/ProgressBar using RangeValuePattern
#[allow(dead_code)]
pub fn get_range_value(found: &FoundElement) -> Result<RangeValueInfo> {
    #[cfg(target_os = "windows")]
    {
        get_range_value_impl(found)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = found;
        bail!("get_range_value only supported on Windows")
    }
}

/// Range value information
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RangeValueInfo {
    pub value: f64,
    pub minimum: f64,
    pub maximum: f64,
    pub step: f64,
    pub is_read_only: bool,
}

#[cfg(target_os = "windows")]
fn get_range_value_impl(found: &FoundElement) -> Result<RangeValueInfo> {
    use windows::Win32::UI::Accessibility::{
        IUIAutomationRangeValuePattern, UIA_RangeValuePatternId,
    };

    unsafe {
        let pattern = found
            .inner
            .GetCurrentPatternAs::<IUIAutomationRangeValuePattern>(UIA_RangeValuePatternId)
            .map_err(|_| anyhow::anyhow!("element does not support RangeValuePattern"))?;

        Ok(RangeValueInfo {
            value: pattern.CurrentValue()?,
            minimum: pattern.CurrentMinimum()?,
            maximum: pattern.CurrentMaximum()?,
            step: pattern.CurrentSmallChange()?,
            is_read_only: pattern.CurrentIsReadOnly()?.into(),
        })
    }
}

/// Set range value for Slider/ProgressBar using RangeValuePattern
#[allow(dead_code)]
pub fn set_range_value(found: &FoundElement, value: f64) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        set_range_value_impl(found, value)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (found, value);
        bail!("set_range_value only supported on Windows")
    }
}

#[cfg(target_os = "windows")]
fn set_range_value_impl(found: &FoundElement, value: f64) -> Result<()> {
    use windows::Win32::UI::Accessibility::{
        IUIAutomationRangeValuePattern, UIA_RangeValuePatternId,
    };

    unsafe {
        let pattern = found
            .inner
            .GetCurrentPatternAs::<IUIAutomationRangeValuePattern>(UIA_RangeValuePatternId)
            .map_err(|_| anyhow::anyhow!("element does not support RangeValuePattern"))?;

        pattern.SetValue(value)?;
        Ok(())
    }
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

    // Helper function tests - these will mostly test structure on non-Windows
    #[test]
    fn test_element_bounds_serialize() {
        let bounds = ElementBounds {
            x: 10.5,
            y: 20.5,
            width: 100.0,
            height: 50.0,
        };
        let json = serde_json::to_string(&bounds).unwrap();
        assert!(json.contains("10.5"));
        assert!(json.contains("100.0"));
    }

    #[test]
    fn test_element_bounds_deserialize() {
        let json = r#"{"x":10.5,"y":20.5,"width":100.0,"height":50.0}"#;
        let bounds: ElementBounds = serde_json::from_str(json).unwrap();
        assert_eq!(bounds.x, 10.5);
        assert_eq!(bounds.y, 20.5);
        assert_eq!(bounds.width, 100.0);
        assert_eq!(bounds.height, 50.0);
    }

    #[test]
    fn test_toggle_state_serialize() {
        assert_eq!(serde_json::to_string(&ToggleState::On).unwrap(), "\"On\"");
        assert_eq!(serde_json::to_string(&ToggleState::Off).unwrap(), "\"Off\"");
        assert_eq!(
            serde_json::to_string(&ToggleState::Indeterminate).unwrap(),
            "\"Indeterminate\""
        );
    }

    #[test]
    fn test_toggle_state_deserialize() {
        assert_eq!(
            serde_json::from_str::<ToggleState>("\"On\"").unwrap(),
            ToggleState::On
        );
        assert_eq!(
            serde_json::from_str::<ToggleState>("\"Off\"").unwrap(),
            ToggleState::Off
        );
        assert_eq!(
            serde_json::from_str::<ToggleState>("\"Indeterminate\"").unwrap(),
            ToggleState::Indeterminate
        );
    }

    #[test]
    fn test_expand_collapse_state_serialize() {
        assert_eq!(
            serde_json::to_string(&ExpandCollapseState::Expanded).unwrap(),
            "\"Expanded\""
        );
        assert_eq!(
            serde_json::to_string(&ExpandCollapseState::Collapsed).unwrap(),
            "\"Collapsed\""
        );
        assert_eq!(
            serde_json::to_string(&ExpandCollapseState::PartiallyExpanded).unwrap(),
            "\"PartiallyExpanded\""
        );
        assert_eq!(
            serde_json::to_string(&ExpandCollapseState::LeafNode).unwrap(),
            "\"LeafNode\""
        );
    }

    #[test]
    fn test_expand_collapse_state_deserialize() {
        assert_eq!(
            serde_json::from_str::<ExpandCollapseState>("\"Expanded\"").unwrap(),
            ExpandCollapseState::Expanded
        );
        assert_eq!(
            serde_json::from_str::<ExpandCollapseState>("\"Collapsed\"").unwrap(),
            ExpandCollapseState::Collapsed
        );
        assert_eq!(
            serde_json::from_str::<ExpandCollapseState>("\"PartiallyExpanded\"").unwrap(),
            ExpandCollapseState::PartiallyExpanded
        );
        assert_eq!(
            serde_json::from_str::<ExpandCollapseState>("\"LeafNode\"").unwrap(),
            ExpandCollapseState::LeafNode
        );
    }

    #[test]
    fn test_scroll_amount_serialize() {
        assert_eq!(
            serde_json::to_string(&ScrollAmount::LargeIncrement).unwrap(),
            "\"LargeIncrement\""
        );
        assert_eq!(
            serde_json::to_string(&ScrollAmount::SmallIncrement).unwrap(),
            "\"SmallIncrement\""
        );
        assert_eq!(
            serde_json::to_string(&ScrollAmount::NoAmount).unwrap(),
            "\"NoAmount\""
        );
        assert_eq!(
            serde_json::to_string(&ScrollAmount::LargeDecrement).unwrap(),
            "\"LargeDecrement\""
        );
        assert_eq!(
            serde_json::to_string(&ScrollAmount::SmallDecrement).unwrap(),
            "\"SmallDecrement\""
        );
    }

    #[test]
    fn test_scroll_amount_deserialize() {
        assert_eq!(
            serde_json::from_str::<ScrollAmount>("\"LargeIncrement\"").unwrap(),
            ScrollAmount::LargeIncrement
        );
        assert_eq!(
            serde_json::from_str::<ScrollAmount>("\"SmallDecrement\"").unwrap(),
            ScrollAmount::SmallDecrement
        );
        assert_eq!(
            serde_json::from_str::<ScrollAmount>("\"NoAmount\"").unwrap(),
            ScrollAmount::NoAmount
        );
    }

    #[test]
    fn test_range_value_info_serialize() {
        let info = RangeValueInfo {
            value: 50.0,
            minimum: 0.0,
            maximum: 100.0,
            step: 1.0,
            is_read_only: false,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("50.0"));
        assert!(json.contains("100.0"));
        assert!(json.contains("false"));
    }

    #[test]
    fn test_range_value_info_deserialize() {
        let json = r#"{"value":75.5,"minimum":0.0,"maximum":200.0,"step":2.5,"is_read_only":true}"#;
        let info: RangeValueInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.value, 75.5);
        assert_eq!(info.minimum, 0.0);
        assert_eq!(info.maximum, 200.0);
        assert_eq!(info.step, 2.5);
        assert!(info.is_read_only);
    }
}
