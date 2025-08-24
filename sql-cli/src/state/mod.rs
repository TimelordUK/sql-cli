//! State management components
//!
//! This module contains the state coordination system that manages
//! Buffer state changes and notifies interested components.

pub mod coordinator;
pub mod dispatcher;
pub mod events;

pub use coordinator::StateCoordinator;
pub use dispatcher::{StateDispatcher, StateSubscriber};
pub use events::{StateChange, StateEvent};

// State components to be extracted from app_state_container.rs:
// - selection_state.rs
// - filter_state.rs
// - sort_state.rs
// - search_state.rs
// - column_search_state.rs
// - clipboard_state.rs
// - chord_state.rs
// - undo_redo_state.rs
// - navigation_state.rs
// - completion_state.rs
