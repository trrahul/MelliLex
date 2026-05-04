use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Screen-space point expressed in physical pixels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScreenPoint {
    pub x: i32,
    pub y: i32,
}

impl ScreenPoint {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Rectangle on screen expressed in physical pixels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScreenRect {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

impl ScreenRect {
    pub const fn new(left: i32, top: i32, width: i32, height: i32) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }
}

/// User-configurable capture preferences and environment metadata.
#[derive(Debug, Clone)]
pub struct CaptureRequest {
    pub cursor: Option<ScreenPoint>,
    pub screen_id: Option<u32>,
    pub timeout: Duration,
    pub prefer_ocr: bool,
}

impl Default for CaptureRequest {
    fn default() -> Self {
        Self {
            cursor: None,
            screen_id: None,
            timeout: Duration::from_millis(150),
            prefer_ocr: false,
        }
    }
}

/// Source describing where captured text originated.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CaptureSource {
    UiaControl,
    MsaaAccessible,
    SelectionApi,
    Ocr,
    Unknown,
}

/// Successful capture payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureResult {
    pub text: String,
    pub source: CaptureSource,
    pub locale: Option<String>,
    pub bounding_box: Option<ScreenRect>,
    pub confidence: f32,
    pub raw_metadata: serde_json::Value,
}

impl CaptureResult {
    pub fn new(text: impl Into<String>, source: CaptureSource) -> Self {
        Self {
            text: text.into(),
            source,
            locale: None,
            bounding_box: None,
            confidence: 1.0,
            raw_metadata: serde_json::Value::Null,
        }
    }
}

impl Default for CaptureResult {
    fn default() -> Self {
        Self {
            text: String::new(),
            source: CaptureSource::Unknown,
            locale: None,
            bounding_box: None,
            confidence: 1.0,
            raw_metadata: serde_json::Value::Null,
        }
    }
}
