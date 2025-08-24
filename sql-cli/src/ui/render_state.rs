//! Render State Manager - Tracks when UI needs re-rendering
//!
//! This module provides centralized tracking of UI state changes
//! and determines when the table (and other components) need to be re-rendered.

use std::time::{Duration, Instant};
use tracing::{debug, trace};

/// Reasons why a re-render might be needed
#[derive(Debug, Clone, PartialEq)]
pub enum RenderReason {
    /// Initial render
    Initial,
    /// User input/key press
    UserInput,
    /// Search results updated
    SearchUpdate,
    /// Navigation/cursor moved
    NavigationChange,
    /// Data changed (filter, sort, etc.)
    DataChange,
    /// Window resized
    WindowResize,
    /// Periodic refresh
    PeriodicRefresh,
    /// Debounced action completed
    DebouncedAction,
}

/// Manages rendering state and dirty flags
pub struct RenderState {
    /// Whether the UI needs re-rendering
    dirty: bool,
    /// Reason for the dirty state
    dirty_reason: Option<RenderReason>,
    /// Last render time
    last_render: Instant,
    /// Minimum time between renders (to prevent excessive redraws)
    min_render_interval: Duration,
    /// Force render on next check
    force_render: bool,
    /// Track if we're in a search/input mode that needs frequent updates
    high_frequency_mode: bool,
}

impl RenderState {
    /// Create a new render state manager
    pub fn new() -> Self {
        Self {
            dirty: true, // Start dirty to trigger initial render
            dirty_reason: Some(RenderReason::Initial),
            last_render: Instant::now(),
            min_render_interval: Duration::from_millis(16), // ~60 FPS max
            force_render: false,
            high_frequency_mode: false,
        }
    }

    /// Mark the UI as needing re-render
    pub fn mark_dirty(&mut self, reason: RenderReason) {
        if !self.dirty {
            debug!("Marking render state dirty: {:?}", reason);
        }
        self.dirty = true;
        self.dirty_reason = Some(reason);
    }

    /// Check if re-render is needed
    pub fn needs_render(&self) -> bool {
        if self.force_render {
            return true;
        }

        if !self.dirty {
            return false;
        }

        // Check if enough time has passed since last render
        let elapsed = self.last_render.elapsed();
        if elapsed < self.min_render_interval && !self.high_frequency_mode {
            trace!("Skipping render, only {:?} elapsed", elapsed);
            return false;
        }

        true
    }

    /// Mark that a render has occurred
    pub fn rendered(&mut self) {
        trace!("Render completed, reason was: {:?}", self.dirty_reason);
        self.dirty = false;
        self.dirty_reason = None;
        self.last_render = Instant::now();
        self.force_render = false;
    }

    /// Force a render on the next check
    pub fn force_render(&mut self) {
        debug!("Forcing render on next check");
        self.force_render = true;
        self.dirty = true;
    }

    /// Set high-frequency mode (for search/input)
    pub fn set_high_frequency_mode(&mut self, enabled: bool) {
        if self.high_frequency_mode != enabled {
            debug!("High-frequency render mode: {}", enabled);
            self.high_frequency_mode = enabled;
            if enabled {
                // Reduce minimum interval for more responsive updates
                self.min_render_interval = Duration::from_millis(8); // ~120 FPS max
            } else {
                self.min_render_interval = Duration::from_millis(16); // ~60 FPS max
            }
        }
    }

    /// Get the current dirty reason
    pub fn dirty_reason(&self) -> Option<&RenderReason> {
        self.dirty_reason.as_ref()
    }

    /// Check if currently dirty
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

/// Helper methods for common state changes
impl RenderState {
    /// Navigation changed (cursor moved)
    pub fn on_navigation_change(&mut self) {
        self.mark_dirty(RenderReason::NavigationChange);
    }

    /// Search results updated
    pub fn on_search_update(&mut self) {
        self.mark_dirty(RenderReason::SearchUpdate);
        // Search updates should render immediately
        self.force_render = true;
    }

    /// Data changed (filter, sort, etc.)
    pub fn on_data_change(&mut self) {
        self.mark_dirty(RenderReason::DataChange);
    }

    /// User input received
    pub fn on_user_input(&mut self) {
        self.mark_dirty(RenderReason::UserInput);
    }

    /// Window resized
    pub fn on_window_resize(&mut self) {
        self.mark_dirty(RenderReason::WindowResize);
        self.force_render = true;
    }

    /// Debounced action completed
    pub fn on_debounced_action(&mut self) {
        self.mark_dirty(RenderReason::DebouncedAction);
        // Debounced actions should render immediately to show results
        self.force_render = true;
    }
}
