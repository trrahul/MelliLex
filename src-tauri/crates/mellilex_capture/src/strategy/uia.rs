use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::{HashSet, VecDeque};
use std::ffi::c_void;
use tokio::task::spawn_blocking;
use uiautomation::patterns::{UITextPattern, UITextRange, UIValuePattern};
use uiautomation::types::{Point, TextPatternRangeEndpoint, TextUnit};
use uiautomation::{UIAutomation, UIElement, UITreeWalker};
use windows::Win32::Foundation::POINT;
use windows::Win32::System::Com::SAFEARRAY;
use windows::Win32::System::Ole::{
    SafeArrayAccessData, SafeArrayDestroy, SafeArrayGetLBound, SafeArrayGetUBound,
    SafeArrayUnaccessData,
};
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

macro_rules! capture_debug {
    ($($arg:tt)*) => {
        log::debug!($($arg)*);
    };
}

macro_rules! capture_info {
    ($($arg:tt)*) => {
        log::info!($($arg)*);
    };
}

use crate::{
    CaptureAttempt, CaptureError, CaptureRequest, CaptureResult, CaptureSource, CaptureStrategy,
};

/// UI Automation-based capture strategy.
#[derive(Clone, Copy)]
pub struct UiaCaptureStrategy;

impl UiaCaptureStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UiaCaptureStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CaptureStrategy for UiaCaptureStrategy {
    fn name(&self) -> &'static str {
        "uiautomation"
    }

    fn is_supported(&self) -> bool {
        cfg!(windows)
    }

    async fn capture(&self, request: &CaptureRequest) -> Result<CaptureAttempt, CaptureError> {
        #[cfg(windows)]
        {
            let strategy = *self;
            let request = request.clone();
            match spawn_blocking(move || strategy.capture_windows(&request))
                .await
                .map_err(|err| CaptureError::internal(format!("UIA capture panicked: {}", err)))?
                .map_err(|e| CaptureError::internal(e.to_string()))?
            {
                Some(result) => Ok(CaptureAttempt::Success(result)),
                None => Ok(CaptureAttempt::NoData),
            }
        }

        #[cfg(not(windows))]
        {
            let _ = request;
            Ok(CaptureAttempt::NoData)
        }
    }
}

#[cfg(windows)]
impl UiaCaptureStrategy {
    fn capture_windows(&self, request: &CaptureRequest) -> Result<Option<CaptureResult>> {
        capture_debug!("[UIA] Attempting capture");

        let cursor_pos = self.get_cursor_point(request)?;
        capture_debug!(
            "[UIA] Cursor at ({}, {})",
            cursor_pos.get_x(),
            cursor_pos.get_y()
        );

        let automation = UIAutomation::new()
            .map_err(|e| anyhow!("Failed to initialize UIAutomation: {:?}", e))?;
        let walker = automation
            .create_tree_walker()
            .map_err(|e| anyhow!("Failed to create tree walker: {:?}", e))?;

        let element = automation
            .element_from_point(cursor_pos)
            .map_err(|e| anyhow!("No element at cursor: {:?}", e))?;

        let element_class = element.get_classname().unwrap_or_default();
        let element_control_type = element.get_control_type().ok();
        let mut element_name = String::new();
        if let Ok(name) = element.get_name() {
            capture_debug!("[UIA] Element name: '{}'", name);
            element_name = name;
        }
        capture_info!(
            "[UIA] Element at cursor: class='{}', control_type={:?}",
            element_class,
            element_control_type
        );

        capture_info!("[UIA] Searching for TextPattern in element tree");
        match self.try_text_pattern_capture(&walker, &element, cursor_pos) {
            Ok(Some(word)) => {
                capture_debug!("[UIA] Captured from TextPattern: '{}'", word);
                return Ok(Some(CaptureResult::new(word, CaptureSource::UiaControl)));
            }
            Ok(None) => {
                capture_info!("[UIA] TextPattern search yielded no text (element may not support IUIAutomationTextPattern)");
            }
            Err(ref e) => {
                capture_info!("[UIA] TextPattern search failed: {:?}", e);
            }
        }

        if !element_name.is_empty() && !element_name.trim().is_empty() {
            let full_text = element_name.trim();
            // Extract single word from the text (split on whitespace/punctuation)
            if let Some(word) = Self::extract_single_word(full_text) {
                log::warn!(
                    "[UIA] Falling back to element.Name — this is the control label, not selected text. class='{}'",
                    element_class
                );
                capture_debug!(
                    "[UIA] Captured word from element.Name: '{}' (full text: '{}')",
                    word,
                    full_text
                );
                return Ok(Some(CaptureResult::new(word, CaptureSource::UiaControl)));
            }
        }

        if let Some(text) = self.try_get_value_pattern(&element)? {
            if let Some(word) = Self::extract_single_word(&text) {
                capture_info!(
                    "[UIA] Captured word from ValuePattern: '{}' (full text: '{}')",
                    word,
                    text
                );
                return Ok(Some(CaptureResult::new(word, CaptureSource::UiaControl)));
            }
        }
        capture_debug!("[UIA] All strategies exhausted, no text found");
        Ok(None)
    }

    /// Try to capture word using TextPattern
    fn try_text_pattern_capture(
        &self,
        walker: &UITreeWalker,
        element: &UIElement,
        cursor_pos: Point,
    ) -> Result<Option<String>> {
        let text_pattern = match self.try_get_text_pattern(walker, element) {
            Some(pattern) => pattern,
            None => {
                capture_debug!("[UIA] No TextPattern near cursor element");
                return Ok(None);
            }
        };

        let raw_range = text_pattern
            .get_ragne_from_point(cursor_pos)
            .map_err(|e| anyhow!("Failed to get range from point: {:?}", e))?;
        let character_range = Self::normalize_to_character_range(&raw_range)?;

        let baseline_geometry = Self::range_geometry(&character_range, cursor_pos)?;
        capture_debug!(
            "[UIA] baseline geometry: rects={}, contains_cursor={}",
            baseline_geometry.rects.len(),
            baseline_geometry.contains_cursor
        );

        // Check if RangeFromPoint returned a range that actually contains the cursor.
        // Chromium-based browsers often return a range at the start of the text block
        // instead of at the cursor position.
        let range_from_point_reliable =
            baseline_geometry.contains_cursor || baseline_geometry.rects.is_empty();

        if range_from_point_reliable {
            let chosen_word =
                self.run_extractors(&character_range, cursor_pos, &baseline_geometry)?;
            if chosen_word.is_some() {
                return Ok(chosen_word);
            }
        } else {
            capture_info!(
                "[UIA] RangeFromPoint appears inaccurate (cursor not in baseline rects), skipping to word walk"
            );
        }

        // Fallback: walk word-by-word through the document to find the word at cursor
        self.try_word_walk(&text_pattern, cursor_pos)
    }

    /// Run the standard word extractors on a character range.
    fn run_extractors(
        &self,
        character_range: &UITextRange,
        cursor_pos: Point,
        baseline_geometry: &RangeGeometry,
    ) -> Result<Option<String>> {
        let mut chosen_word: Option<String> = None;
        let mut attempt_index = 0;

        for extractor in self.word_extractors() {
            attempt_index += 1;
            let range_instance = character_range.clone();
            capture_debug!(
                "[UIA] extractor[{}] '{}' starting",
                attempt_index,
                extractor.name()
            );

            match extractor.extract(self, &range_instance, cursor_pos, baseline_geometry) {
                Ok(Some(word)) => {
                    capture_debug!(
                        "[UIA] extractor[{}] '{}' succeeded with '{}'",
                        attempt_index,
                        extractor.name(),
                        word
                    );
                    if chosen_word.is_none() {
                        chosen_word = Some(word);
                    } else {
                        capture_debug!(
                            "[UIA] extractor[{}] '{}' also produced '{}' (already have primary result)",
                            attempt_index,
                            extractor.name(),
                            word
                        );
                    }
                }
                Ok(None) => {
                    capture_debug!(
                        "[UIA] extractor[{}] '{}' returned no data",
                        attempt_index,
                        extractor.name()
                    );
                }
                Err(err) => {
                    capture_debug!(
                        "[UIA] extractor[{}] '{}' failed: {:?}",
                        attempt_index,
                        extractor.name(),
                        err
                    );
                }
            }
        }

        Ok(chosen_word)
    }

    /// Fallback when RangeFromPoint is inaccurate (common in Chromium-based browsers).
    /// Walks word-by-word through the document range, checking each word's bounding
    /// rectangle against the cursor position.
    fn try_word_walk(
        &self,
        text_pattern: &UITextPattern,
        cursor_pos: Point,
    ) -> Result<Option<String>> {
        let doc_range = text_pattern
            .get_document_range()
            .map_err(|e| anyhow!("Failed to get document range: {:?}", e))?;

        let probe = doc_range.clone();
        // Collapse to start of document
        probe
            .move_text(TextUnit::Character, -999_999)
            .map_err(|e| anyhow!("Failed to collapse range to start: {:?}", e))?;

        const MAX_WORDS: usize = 200;
        let cursor_y = cursor_pos.get_y() as f64;

        for word_idx in 0..MAX_WORDS {
            let word_range = probe.clone();
            if word_range
                .expand_to_enclosing_unit(TextUnit::Word)
                .is_err()
            {
                break;
            }

            let geo = Self::range_geometry(&word_range, cursor_pos)?;

            if geo.contains_cursor {
                if let Some(text) = Self::extract_word_from_range(&word_range)? {
                    if !text.is_empty() {
                        capture_info!(
                            "[UIA] Word walk found word at cursor (position {})",
                            word_idx + 1
                        );
                        capture_debug!("[UIA] Word walk result: '{}'", text);
                        return Ok(Some(text));
                    }
                }
            }

            // Early exit: if all rects are clearly below the cursor, stop
            if !geo.rects.is_empty() {
                let min_top = geo
                    .rects
                    .iter()
                    .map(|r| r.top)
                    .fold(f64::INFINITY, f64::min);
                if min_top > cursor_y + CURSOR_HIT_PADDING {
                    capture_debug!("[UIA] Word walk: past cursor vertically, stopping");
                    break;
                }
            }

            let moved = probe
                .move_text(TextUnit::Word, 1)
                .map_err(|e| anyhow!("Failed to advance to next word: {:?}", e))?;
            if moved == 0 {
                break;
            }
        }

        capture_debug!("[UIA] Word walk did not find cursor word");
        Ok(None)
    }

    fn get_cursor_point(&self, request: &CaptureRequest) -> Result<Point> {
        if let Some(cursor) = request.cursor {
            return Ok(Point::new(cursor.x, cursor.y));
        }

        let mut raw = POINT::default();
        unsafe {
            GetCursorPos(&mut raw).map_err(|e| anyhow!("GetCursorPos failed: {:?}", e))?;
        }
        Ok(Point::new(raw.x, raw.y))
    }

    fn try_get_text_pattern(
        &self,
        walker: &UITreeWalker,
        element: &UIElement,
    ) -> Option<UITextPattern> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(element.clone());
        let mut nodes_checked = 0usize;

        while let Some(current) = queue.pop_front() {
            let runtime_id = current.get_runtime_id().ok()?;
            if visited.contains(&runtime_id) {
                continue;
            }
            nodes_checked += 1;
            let node_name = current.get_name().unwrap_or_default();
            let node_class = current.get_classname().unwrap_or_default();
            capture_debug!(
                "[UIA] BFS node {}: name='{}' class='{}'" ,
                nodes_checked, node_name, node_class
            );
            if let Ok(pattern) = current.get_pattern::<UITextPattern>() {
                capture_info!(
                    "[UIA] Found TextPattern on node {} (class='{}')",
                    nodes_checked, node_class
                );
                return Some(pattern);
            }
            visited.insert(runtime_id);

            if let Ok(child) = walker.get_first_child(&current) {
                queue.push_back(child);
            }
            if let Ok(sibling) = walker.get_next_sibling(&current) {
                queue.push_back(sibling);
            }
            if let Ok(parent) = walker.get_parent(&current) {
                queue.push_back(parent);
            }
        }

        capture_info!("[UIA] No TextPattern found after checking {} nodes", nodes_checked);
        None
    }

    fn extract_word_from_range(range: &UITextRange) -> Result<Option<String>> {
        let text = range
            .get_text(-1)
            .map_err(|e| anyhow!("Failed to get text: {:?}", e))?;
        capture_debug!("[UIA] range text raw='{}'", text);
        // Strip U+FFFC (object replacement character) and other non-text chars
        // that apps like Kindle web reader return instead of actual text.
        let cleaned: String = text
            .chars()
            .filter(|&c| c != '\u{FFFC}' && c != '\u{FFFE}' && c != '\u{FFFF}')
            .collect();
        let trimmed = cleaned.trim();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed.to_string()))
        }
    }

    fn try_expand_character_window(
        &self,
        range: &UITextRange,
        cursor_pos: Point,
        baseline: &RangeGeometry,
    ) -> Result<Option<String>> {
        const MAX_RADIUS: i32 = 12;
        for radius in 1..=MAX_RADIUS {
            let candidate = range.clone();
            let start_moved = candidate
                .move_endpoint_by_unit(
                    TextPatternRangeEndpoint::Start,
                    TextUnit::Character,
                    -radius,
                )
                .map_err(|e| anyhow!("Failed moving start endpoint: {:?}", e))?;
            let end_moved = candidate
                .move_endpoint_by_unit(TextPatternRangeEndpoint::End, TextUnit::Character, radius)
                .map_err(|e| anyhow!("Failed moving end endpoint: {:?}", e))?;

            if start_moved == 0 && end_moved == 0 {
                continue;
            }

            candidate
                .expand_to_enclosing_unit(TextUnit::Word)
                .map_err(|e| anyhow!("Failed to expand candidate: {:?}", e))?;

            if !Self::cursor_in_range(&candidate, cursor_pos, Some(baseline))? {
                continue;
            }

            if let Some(word) = Self::extract_word_from_range(&candidate)? {
                return Ok(Some(word));
            }
        }

        Ok(None)
    }

    /// Try to get text using the Value pattern (works for textboxes, combo boxes, etc.)
    fn try_get_value_pattern(&self, element: &UIElement) -> Result<Option<String>> {
        match element.get_pattern::<UIValuePattern>() {
            Ok(pattern) => {
                if let Ok(value) = pattern.get_value() {
                    let trimmed = value.trim();
                    if !trimmed.is_empty() {
                        capture_debug!("[UIA] ValuePattern returned: '{}'", trimmed);
                        return Ok(Some(trimmed.to_string()));
                    }
                }
                Ok(None)
            }
            Err(e) => {
                capture_debug!("[UIA] ValuePattern not available: {:?}", e);
                Ok(None)
            }
        }
    }

    /// Extract a single word from text (prioritizes first alphabetic word)
    fn extract_single_word(text: &str) -> Option<String> {
        // Split on whitespace and common separators
        let words: Vec<&str> = text
            .split(|c: char| {
                c.is_whitespace() || c == '.' || c == '_' || c == '-' || c == '/' || c == '\\'
            })
            .filter(|w| !w.is_empty())
            .collect();

        // Find first word with alphabetic characters
        for word in &words {
            if word.chars().any(|c| c.is_alphabetic()) {
                // Remove file extensions if present
                let clean = word.trim_end_matches(|c: char| !c.is_alphanumeric());
                if !clean.is_empty() {
                    capture_debug!("[UIA] Extracted word '{}' from text '{}'", clean, text);
                    return Some(clean.to_string());
                }
            }
        }

        // Fallback: return first non-empty word
        words.first().map(|w| {
            let clean = w.trim_matches(|c: char| !c.is_alphanumeric());
            capture_debug!("[UIA] Fallback: extracted '{}' from text '{}'", clean, text);
            clean.to_string()
        })
    }

    fn word_extractors(&self) -> Vec<Box<dyn WordRangeExtractor>> {
        vec![
            Box::new(CursorWalkExtractor),
            Box::new(WordUnitExtractor),
            Box::new(RadialExpansionExtractor),
        ]
    }

    fn normalize_to_character_range(range: &UITextRange) -> Result<UITextRange> {
        let character_range = range.clone();
        character_range
            .expand_to_enclosing_unit(TextUnit::Character)
            .map_err(|e| anyhow!("Failed to clamp range to character: {:?}", e))?;
        Ok(character_range)
    }

    fn walk_to_word(&self, char_range: &UITextRange) -> Result<Option<String>> {
        let word_range = char_range.clone();
        Self::extend_endpoint_to_boundary(&word_range, TextPatternRangeEndpoint::Start, -1)?;
        Self::extend_endpoint_to_boundary(&word_range, TextPatternRangeEndpoint::End, 1)?;

        Self::extract_word_from_range(&word_range)
    }

    fn extend_endpoint_to_boundary(
        range: &UITextRange,
        endpoint: TextPatternRangeEndpoint,
        step: i32,
    ) -> Result<()> {
        const MAX_STEPS: i32 = 64;
        let mut traversed = 0;

        loop {
            let moved = range
                .move_endpoint_by_unit(endpoint, TextUnit::Character, step)
                .map_err(|e| anyhow!("Failed to move endpoint: {:?}", e))?;

            if moved == 0 {
                break;
            }
            traversed += moved.abs();
            if traversed >= MAX_STEPS {
                break;
            }

            let text = range
                .get_text(-1)
                .map_err(|e| anyhow!("Failed to read range text: {:?}", e))?;

            if text.is_empty() {
                continue;
            }

            let boundary_char = match endpoint {
                TextPatternRangeEndpoint::Start => text.chars().next(),
                TextPatternRangeEndpoint::End => text.chars().last(),
            };

            if let Some(ch) = boundary_char {
                if is_boundary_char(ch) {
                    range
                        .move_endpoint_by_unit(endpoint, TextUnit::Character, -step)
                        .map_err(|e| anyhow!("Failed to rewind endpoint: {:?}", e))?;
                    break;
                }
            }
        }

        Ok(())
    }

    fn cursor_in_range(
        range: &UITextRange,
        cursor: Point,
        baseline: Option<&RangeGeometry>,
    ) -> Result<bool> {
        let geometry = Self::range_geometry(range, cursor)?;
        capture_debug!(
            "[UIA] range geometry: rects={} (raw={}), contains_cursor={}",
            geometry.rects.len(),
            geometry.raw_rect_count,
            geometry.contains_cursor
        );

        if geometry.rects.is_empty() {
            capture_debug!("[UIA] no bounding rectangles, accepting range heuristically");
            return Ok(true);
        }

        if !geometry.contains_cursor {
            let near_enough = geometry
                .rects
                .first()
                .map(|rect| rect.distance_to_point(cursor.get_x() as f64, cursor.get_y() as f64))
                .map(|distance| {
                    capture_debug!(
                        "[UIA] cursor is {}px away from bounding box",
                        distance as i32
                    );
                    distance <= CURSOR_HIT_PADDING
                })
                .unwrap_or(false);

            if !near_enough {
                capture_debug!("[UIA] rejecting range: cursor not inside bounding rectangle");
                return Ok(false);
            }
        }

        if let Some(base) = baseline {
            if Self::geometry_exceeds_bounds(base, &geometry) {
                capture_debug!(
                    "[UIA] rejecting range: geometry exceeds baseline (base rects={}, candidate rects={})",
                    base.rects.len(),
                    geometry.rects.len()
                );
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn geometry_exceeds_bounds(base: &RangeGeometry, candidate: &RangeGeometry) -> bool {
        if base.rects.is_empty() || candidate.rects.is_empty() {
            return false;
        }

        let base_area = base.rects.iter().map(|r| r.area()).sum::<f64>().max(1.0);
        let candidate_area = candidate.rects.iter().map(|r| r.area()).sum::<f64>();
        let area_ratio = candidate_area / base_area;

        let base_max_height = base
            .rects
            .iter()
            .map(|r| r.height)
            .fold(0.0_f64, f64::max)
            .max(1.0);
        let candidate_height_sum: f64 = candidate.rects.iter().map(|r| r.height).sum();

        let multi_line = candidate.rects.len() > 3 && candidate_height_sum > base_max_height * 3.0;

        area_ratio > 80.0 || multi_line
    }

    fn range_geometry(range: &UITextRange, cursor: Point) -> Result<RangeGeometry> {
        let rects = Self::range_bounding_rectangles(range)?;
        let raw_rect_count = rects.len();

        if raw_rect_count == 0 {
            return Ok(RangeGeometry {
                rects,
                contains_cursor: false,
                raw_rect_count,
            });
        }

        let cursor_x = cursor.get_x() as f64;
        let cursor_y = cursor.get_y() as f64;

        let rects_with_cursor: Vec<BoundingBox> = rects
            .iter()
            .cloned()
            .filter(|rect| rect.contains_with_padding(cursor_x, cursor_y, CURSOR_HIT_PADDING))
            .collect();

        let contains_cursor = !rects_with_cursor.is_empty();
        let effective_rects = if contains_cursor {
            rects_with_cursor
        } else {
            rects.clone()
        };

        Ok(RangeGeometry {
            rects: effective_rects,
            contains_cursor,
            raw_rect_count,
        })
    }

    fn range_bounding_rectangles(range: &UITextRange) -> Result<Vec<BoundingBox>> {
        let raw_array = unsafe {
            match range.as_ref().GetBoundingRectangles() {
                Ok(ptr) => ptr as *mut SAFEARRAY,
                Err(err) => {
                    capture_debug!("[UIA] Bounding rectangles unavailable: {:?}", err);
                    return Ok(Vec::new());
                }
            }
        };

        if raw_array.is_null() {
            return Ok(Vec::new());
        }

        let handle = SafeArrayHandle::new(raw_array);
        let lower = unsafe {
            SafeArrayGetLBound(handle.as_const(), 1)
                .map_err(|e| anyhow!("SafeArrayGetLBound failed: {:?}", e))?
        };
        let upper = unsafe {
            SafeArrayGetUBound(handle.as_const(), 1)
                .map_err(|e| anyhow!("SafeArrayGetUBound failed: {:?}", e))?
        };

        if upper < lower {
            return Ok(Vec::new());
        }

        let len = (upper - lower + 1) as usize;
        if len == 0 {
            return Ok(Vec::new());
        }

        let mut data_ptr: *mut f64 = std::ptr::null_mut();
        unsafe {
            SafeArrayAccessData(
                handle.as_const(),
                &mut data_ptr as *mut *mut f64 as *mut *mut c_void,
            )
            .map_err(|e| anyhow!("SafeArrayAccessData failed: {:?}", e))?;
        }
        let access_guard = SafeArrayAccessGuard::new(handle.as_const());

        let values = unsafe { std::slice::from_raw_parts(data_ptr as *const f64, len) }.to_vec();
        drop(access_guard);

        let mut rects = Vec::new();
        for chunk in values.chunks(4) {
            if chunk.len() < 4 {
                break;
            }
            let rect = BoundingBox {
                left: chunk[0],
                top: chunk[1],
                width: chunk[2],
                height: chunk[3],
            };
            capture_debug!("[UIA] bounding box {:?}", rect);
            rects.push(rect);
        }

        Ok(rects)
    }
}

fn is_boundary_char(ch: char) -> bool {
    if ch.is_whitespace() || ch.is_control() {
        return true;
    }

    if ch.is_alphanumeric() {
        return false;
    }

    !matches!(ch, '\'' | '-')
}

#[cfg(windows)]
#[derive(Clone)]
struct RangeGeometry {
    rects: Vec<BoundingBox>,
    contains_cursor: bool,
    raw_rect_count: usize,
}

const CURSOR_HIT_PADDING: f64 = 96.0;

#[cfg(windows)]
#[derive(Debug, Clone, Copy)]
struct BoundingBox {
    left: f64,
    top: f64,
    width: f64,
    height: f64,
}

#[cfg(windows)]
impl BoundingBox {
    fn area(&self) -> f64 {
        let w = self.width.max(0.0);
        let h = self.height.max(0.0);
        w * h
    }

    fn contains_with_padding(&self, x: f64, y: f64, padding: f64) -> bool {
        let left = self.left - padding;
        let top = self.top - padding;
        let right = self.left + self.width + padding;
        let bottom = self.top + self.height + padding;
        x >= left && x <= right && y >= top && y <= bottom
    }

    fn distance_to_point(&self, x: f64, y: f64) -> f64 {
        let clamped_x = if x < self.left {
            self.left
        } else if x > self.left + self.width {
            self.left + self.width
        } else {
            x
        };
        let clamped_y = if y < self.top {
            self.top
        } else if y > self.top + self.height {
            self.top + self.height
        } else {
            y
        };

        let dx = x - clamped_x;
        let dy = y - clamped_y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[cfg(windows)]
struct SafeArrayHandle(*mut SAFEARRAY);

#[cfg(windows)]
impl SafeArrayHandle {
    fn new(ptr: *mut SAFEARRAY) -> Self {
        Self(ptr)
    }

    fn as_const(&self) -> *const SAFEARRAY {
        self.0 as *const SAFEARRAY
    }
}

#[cfg(windows)]
impl Drop for SafeArrayHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let _ = SafeArrayDestroy(self.0 as *const SAFEARRAY);
            }
            self.0 = std::ptr::null_mut();
        }
    }
}

#[cfg(windows)]
struct SafeArrayAccessGuard(*const SAFEARRAY);

#[cfg(windows)]
impl SafeArrayAccessGuard {
    fn new(ptr: *const SAFEARRAY) -> Self {
        Self(ptr)
    }
}

#[cfg(windows)]
impl Drop for SafeArrayAccessGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let _ = SafeArrayUnaccessData(self.0);
            }
            self.0 = std::ptr::null();
        }
    }
}

#[cfg(windows)]
trait WordRangeExtractor {
    fn name(&self) -> &'static str;
    fn extract(
        &self,
        owner: &UiaCaptureStrategy,
        range: &UITextRange,
        cursor_pos: Point,
        baseline: &RangeGeometry,
    ) -> Result<Option<String>>;
}

#[cfg(windows)]
struct CursorWalkExtractor;

#[cfg(windows)]
impl WordRangeExtractor for CursorWalkExtractor {
    fn name(&self) -> &'static str {
        "cursor_walk"
    }

    fn extract(
        &self,
        owner: &UiaCaptureStrategy,
        range: &UITextRange,
        cursor_pos: Point,
        _baseline: &RangeGeometry,
    ) -> Result<Option<String>> {
        if !UiaCaptureStrategy::cursor_in_range(range, cursor_pos, None)? {
            capture_debug!("[UIA][cursor_walk] cursor not within range geometry");
            return Ok(None);
        }

        let result = owner.walk_to_word(range)?;
        if result.is_none() {
            capture_debug!("[UIA][cursor_walk] walk_to_word returned empty result");
        }
        Ok(result)
    }
}

#[cfg(windows)]
struct WordUnitExtractor;

#[cfg(windows)]
impl WordRangeExtractor for WordUnitExtractor {
    fn name(&self) -> &'static str {
        "textunit_word"
    }

    fn extract(
        &self,
        _owner: &UiaCaptureStrategy,
        range: &UITextRange,
        cursor_pos: Point,
        baseline: &RangeGeometry,
    ) -> Result<Option<String>> {
        if !UiaCaptureStrategy::cursor_in_range(range, cursor_pos, None)? {
            capture_debug!("[UIA][textunit_word] initial range rejected by geometry");
            return Ok(None);
        }

        let candidate = range.clone();
        candidate
            .expand_to_enclosing_unit(TextUnit::Word)
            .map_err(|e| anyhow!("Failed to expand to word: {:?}", e))?;

        if !UiaCaptureStrategy::cursor_in_range(&candidate, cursor_pos, Some(baseline))? {
            capture_debug!("[UIA][textunit_word] expanded range lost cursor");
            return Ok(None);
        }

        let word = UiaCaptureStrategy::extract_word_from_range(&candidate)?;
        if word.is_none() {
            capture_debug!("[UIA][textunit_word] no text extracted after expansion");
        }
        Ok(word)
    }
}

#[cfg(windows)]
struct RadialExpansionExtractor;

#[cfg(windows)]
impl WordRangeExtractor for RadialExpansionExtractor {
    fn name(&self) -> &'static str {
        "radial_word_probe"
    }

    fn extract(
        &self,
        owner: &UiaCaptureStrategy,
        range: &UITextRange,
        cursor_pos: Point,
        baseline: &RangeGeometry,
    ) -> Result<Option<String>> {
        if !UiaCaptureStrategy::cursor_in_range(range, cursor_pos, None)? {
            capture_debug!("[UIA][radial_word_probe] initial range rejected");
            return Ok(None);
        }

        let word = owner.try_expand_character_window(range, cursor_pos, baseline)?;
        if word.is_none() {
            capture_debug!("[UIA][radial_word_probe] radial expansion returned no result");
        }
        Ok(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strategy_name() {
        assert_eq!(UiaCaptureStrategy::new().name(), "uiautomation");
    }

    #[test]
    fn strategy_availability_matches_platform() {
        let strategy = UiaCaptureStrategy::new();
        #[cfg(windows)]
        assert!(strategy.is_supported());
        #[cfg(not(windows))]
        assert!(!strategy.is_supported());
    }

    #[test]
    fn boundary_detection_handles_word_characters() {
        assert!(!super::is_boundary_char('a'));
        assert!(!super::is_boundary_char('-'));
        assert!(!super::is_boundary_char('\''));
        assert!(super::is_boundary_char(' '));
        assert!(super::is_boundary_char(','));
    }
}
