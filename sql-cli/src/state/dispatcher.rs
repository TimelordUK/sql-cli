//! State dispatcher for pub-sub pattern

use crate::buffer::Buffer;
use crate::state::coordinator::StateCoordinator;
use crate::state::events::{StateChange, StateEvent};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use tracing::{debug, info, warn};

/// Trait for components that subscribe to state changes
pub trait StateSubscriber {
    /// Handle a state event
    fn on_state_event(&mut self, event: &StateEvent, buffer: &Buffer);

    /// Get subscriber name for debugging
    fn name(&self) -> &str;
}

/// Commands that subscribers can return to be executed
#[derive(Debug)]
pub enum SubscriberCommand {
    None,
    ClearSearch,
    UpdateViewport,
    RefreshDisplay,
}

/// State dispatcher that coordinates buffer state changes with subscribers
pub struct StateDispatcher {
    /// Weak reference to the buffer (to avoid circular references)
    buffer: Weak<RefCell<Buffer>>,

    /// List of subscribers
    subscribers: Vec<Box<dyn StateSubscriber>>,

    /// Event history for debugging
    event_history: Vec<StateEvent>,

    /// Maximum event history size
    max_history: usize,
}

impl StateDispatcher {
    pub fn new() -> Self {
        Self {
            buffer: Weak::new(),
            subscribers: Vec::new(),
            event_history: Vec::new(),
            max_history: 100,
        }
    }

    /// Set the buffer this dispatcher coordinates
    pub fn set_buffer(&mut self, buffer: Rc<RefCell<Buffer>>) {
        self.buffer = Rc::downgrade(&buffer);
    }

    /// Add a subscriber
    pub fn subscribe(&mut self, subscriber: Box<dyn StateSubscriber>) {
        info!("StateDispatcher: Adding subscriber: {}", subscriber.name());
        self.subscribers.push(subscriber);
    }

    /// Dispatch a state event
    pub fn dispatch(&mut self, event: StateEvent) {
        debug!("StateDispatcher: Dispatching event: {:?}", event);

        // Record event in history
        self.event_history.push(event.clone());
        if self.event_history.len() > self.max_history {
            self.event_history.remove(0);
        }

        // Get buffer reference
        let buffer_rc = match self.buffer.upgrade() {
            Some(b) => b,
            None => {
                warn!("StateDispatcher: Buffer reference lost!");
                return;
            }
        };

        // Process event to get state changes
        let change = {
            let buffer = buffer_rc.borrow();
            buffer.process_event(&event)
        };

        // Apply changes to buffer
        if !matches!(
            change,
            StateChange {
                mode: None,
                search_state: None,
                filter_state: None,
                fuzzy_filter_state: None,
                clear_all_searches: false
            }
        ) {
            info!("StateDispatcher: Applying state change: {:?}", change);
            buffer_rc.borrow_mut().apply_change(change);
        }

        // Notify all subscribers
        let buffer = buffer_rc.borrow();
        for subscriber in &mut self.subscribers {
            debug!(
                "StateDispatcher: Notifying subscriber: {}",
                subscriber.name()
            );
            subscriber.on_state_event(&event, &buffer);
        }
    }

    /// Dispatch a mode change event
    pub fn dispatch_mode_change(
        &mut self,
        from: crate::buffer::AppMode,
        to: crate::buffer::AppMode,
    ) {
        self.dispatch(StateEvent::ModeChanged { from, to });
    }

    /// Dispatch a search start event
    pub fn dispatch_search_start(&mut self, search_type: crate::ui::shadow_state::SearchType) {
        self.dispatch(StateEvent::SearchStarted { search_type });
    }

    /// Dispatch a search end event
    pub fn dispatch_search_end(&mut self, search_type: crate::ui::shadow_state::SearchType) {
        self.dispatch(StateEvent::SearchEnded { search_type });
    }

    /// Get event history for debugging
    pub fn get_event_history(&self) -> &[StateEvent] {
        &self.event_history
    }
}

/// Example subscriber for VimSearchManager
pub struct VimSearchSubscriber {
    active: bool,
}

impl VimSearchSubscriber {
    pub fn new() -> Self {
        Self { active: false }
    }
}

impl StateSubscriber for VimSearchSubscriber {
    fn on_state_event(&mut self, event: &StateEvent, buffer: &Buffer) {
        match event {
            StateEvent::SearchStarted { search_type } => {
                if matches!(search_type, crate::ui::shadow_state::SearchType::Vim) {
                    info!("VimSearchSubscriber: Activating for vim search");
                    self.active = true;
                }
            }
            StateEvent::SearchEnded { search_type } => {
                if matches!(search_type, crate::ui::shadow_state::SearchType::Vim) {
                    info!("VimSearchSubscriber: Deactivating - search ended");
                    self.active = false;
                }
            }
            StateEvent::ModeChanged { from: _, to } => {
                // Check if we should deactivate based on buffer state
                if *to == crate::buffer::AppMode::Results && buffer.search_state.pattern.is_empty()
                {
                    if self.active {
                        info!("VimSearchSubscriber: Deactivating - mode changed to Results with empty search");
                    }
                    self.active = false;
                }
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "VimSearchSubscriber"
    }
}
