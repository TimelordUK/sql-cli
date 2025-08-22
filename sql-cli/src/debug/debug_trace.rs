use std::fmt;

/// A section of debug information with a title and content
#[derive(Debug, Clone)]
pub struct DebugSection {
    /// The title of this debug section (e.g., "VIEWPORT STATE", "DATAVIEW STATE")
    pub title: String,
    /// The content of this debug section
    pub content: String,
    /// Priority for ordering (lower values appear first)
    pub priority: u32,
}

impl DebugSection {
    /// Create a new debug section
    pub fn new(title: impl Into<String>, content: impl Into<String>, priority: u32) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
            priority,
        }
    }

    /// Create a section with a formatted title
    pub fn with_header(
        title: impl Into<String>,
        content: impl Into<String>,
        priority: u32,
    ) -> Self {
        let title_str = title.into();
        let header = format!("\n========== {} ==========\n", title_str);
        Self {
            title: title_str,
            content: format!("{}{}", header, content.into()),
            priority,
        }
    }
}

impl fmt::Display for DebugSection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

/// Trait for components that can provide debug information
pub trait DebugTrace: Send + Sync {
    /// Get the name of this debug provider
    fn name(&self) -> &str;

    /// Generate debug sections for the current state
    /// Returns a vector of debug sections that can be displayed
    fn debug_sections(&self) -> Vec<DebugSection>;

    /// Optional method to get a quick summary (one-liner)
    fn debug_summary(&self) -> Option<String> {
        None
    }

    /// Check if this provider is currently active/relevant
    fn is_active(&self) -> bool {
        true
    }
}

/// Priority constants for standard sections
pub mod Priority {
    pub const PARSER: u32 = 100;
    pub const BUFFER: u32 = 200;
    pub const RESULTS: u32 = 300;
    pub const DATATABLE: u32 = 400;
    pub const DATAVIEW: u32 = 500;
    pub const VIEWPORT: u32 = 600;
    pub const MEMORY: u32 = 700;
    pub const NAVIGATION: u32 = 800;
    pub const RENDER: u32 = 900;
    pub const TRACE: u32 = 1000;
    pub const STATE_LOGS: u32 = 1100;
}

/// Helper builder for creating debug sections
pub struct DebugSectionBuilder {
    sections: Vec<DebugSection>,
}

impl DebugSectionBuilder {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    pub fn add_section(
        &mut self,
        title: impl Into<String>,
        content: impl Into<String>,
        priority: u32,
    ) -> &mut Self {
        self.sections
            .push(DebugSection::with_header(title, content, priority));
        self
    }

    pub fn add_raw(&mut self, section: DebugSection) -> &mut Self {
        self.sections.push(section);
        self
    }

    pub fn add_field(&mut self, name: &str, value: impl fmt::Display) -> &mut Self {
        if let Some(last) = self.sections.last_mut() {
            last.content.push_str(&format!("{}: {}\n", name, value));
        }
        self
    }

    pub fn add_line(&mut self, line: impl Into<String>) -> &mut Self {
        if let Some(last) = self.sections.last_mut() {
            last.content.push_str(&format!("{}\n", line.into()));
        }
        self
    }

    pub fn build(self) -> Vec<DebugSection> {
        self.sections
    }
}

impl Default for DebugSectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}
