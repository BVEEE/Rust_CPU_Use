#![cfg(not(target_os = "windows"))]

use crate::state::{MatchedElement, Selector, UIElement};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

// Stub implementations for non-Windows platforms

/// Initialize COM for UI Automation (stub)
pub fn initialize_com() -> Result<()> {
    Ok(())
}

/// Uninitialize COM (stub)
pub fn uninitialize_com() {
    // No-op on non-Windows
}

/// Capture UI Automation tree for a window (stub)
pub fn capture_ui_tree(_window_handle: u64, _max_depth: usize) -> Result<Option<UIElement>> {
    Ok(None)
}

/// Find an element matching the selector in the focused window (stub)
pub fn find_element(_window_handle: u64, _selector: &Selector) -> Result<FoundElement> {
    bail!("UI Automation not supported on this platform");
}

/// Opaque handle to a found UI Automation element (stub)
pub struct FoundElement {
    /// Element metadata for response
    pub info: MatchedElement,
}

/// Click the element (stub)
pub fn click_element(_found: &FoundElement) -> Result<()> {
    bail!("UI Automation not supported on this platform");
}

/// Get text from the element (stub)
pub fn get_element_text(_found: &FoundElement) -> Result<String> {
    bail!("UI Automation not supported on this platform");
}

/// Check if the element is enabled (stub)
pub fn is_element_enabled(_found: &FoundElement) -> Result<bool> {
    bail!("UI Automation not supported on this platform");
}

/// Check if the element is visible (stub)
pub fn is_element_visible(_found: &FoundElement) -> Result<bool> {
    bail!("UI Automation not supported on this platform");
}

/// Get the bounding rectangle of the element (stub)
pub fn get_element_bounds(_found: &FoundElement) -> Result<ElementBounds> {
    bail!("UI Automation not supported on this platform");
}

/// Bounding rectangle of a UI element (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Toggle a CheckBox element (stub)
pub fn toggle_element(_found: &FoundElement) -> Result<ToggleState> {
    bail!("UI Automation not supported on this platform");
}

/// Toggle state enumeration (stub)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToggleState {
    Off,
    On,
    Indeterminate,
}

/// Expand or collapse a TreeItem/Menu element (stub)
pub fn expand_collapse_element(_found: &FoundElement, _expand: bool) -> Result<ExpandCollapseState> {
    bail!("UI Automation not supported on this platform");
}

/// Expand/Collapse state enumeration (stub)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpandCollapseState {
    Collapsed,
    Expanded,
    PartiallyExpanded,
    LeafNode,
}

/// Select an item in a List/Combo (stub)
pub fn select_element(_found: &FoundElement) -> Result<()> {
    bail!("UI Automation not supported on this platform");
}

/// Get all selected items from a container (stub)
pub fn get_selected_items(_found: &FoundElement) -> Result<Vec<MatchedElement>> {
    bail!("UI Automation not supported on this platform");
}

/// Scroll an element (stub)
pub fn scroll_element(
    _found: &FoundElement,
    _horizontal: ScrollAmount,
    _vertical: ScrollAmount,
) -> Result<()> {
    bail!("UI Automation not supported on this platform");
}

/// Scroll amount enumeration (stub)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollAmount {
    LargeDecrement,
    SmallDecrement,
    NoAmount,
    LargeIncrement,
    SmallIncrement,
}

/// Get range value from Slider/ProgressBar (stub)
pub fn get_range_value(_found: &FoundElement) -> Result<RangeValueInfo> {
    bail!("UI Automation not supported on this platform");
}

/// Range value information (stub)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeValueInfo {
    pub value: f64,
    pub minimum: f64,
    pub maximum: f64,
    pub step: f64,
    pub is_read_only: bool,
}

/// Set range value for Slider/ProgressBar (stub)
pub fn set_range_value(_found: &FoundElement, _value: f64) -> Result<()> {
    bail!("UI Automation not supported on this platform");
}

/// Set value on the element (stub)
pub fn set_element_value(_found: &FoundElement, _value: &str) -> Result<()> {
    bail!("UI Automation not supported on this platform");
}

/// Invoke the element (stub)
pub fn invoke_element(_found: &FoundElement) -> Result<()> {
    bail!("UI Automation not supported on this platform");
}