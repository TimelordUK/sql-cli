use super::debug_trace::{DebugSection, DebugTrace};
use std::sync::{Arc, RwLock};

/// Registry for managing debug trace providers
pub struct DebugRegistry {
    providers: Arc<RwLock<Vec<Arc<dyn DebugTrace>>>>,
}

impl DebugRegistry {
    /// Create a new debug registry
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a debug provider
    pub fn register(&self, provider: Arc<dyn DebugTrace>) {
        if let Ok(mut providers) = self.providers.write() {
            // Check if provider with same name already exists
            let name = provider.name();
            providers.retain(|p| p.name() != name);
            providers.push(provider);
        }
    }

    /// Unregister a debug provider by name
    pub fn unregister(&self, name: &str) {
        if let Ok(mut providers) = self.providers.write() {
            providers.retain(|p| p.name() != name);
        }
    }

    /// Clear all providers
    pub fn clear(&self) {
        if let Ok(mut providers) = self.providers.write() {
            providers.clear();
        }
    }

    /// Get all debug sections from active providers
    pub fn collect_debug_sections(&self) -> Vec<DebugSection> {
        let mut all_sections = Vec::new();

        if let Ok(providers) = self.providers.read() {
            for provider in providers.iter() {
                if provider.is_active() {
                    all_sections.extend(provider.debug_sections());
                }
            }
        }

        // Sort by priority
        all_sections.sort_by_key(|s| s.priority);
        all_sections
    }

    /// Generate a complete debug report
    pub fn generate_debug_report(&self) -> String {
        let sections = self.collect_debug_sections();
        let mut report = String::new();

        for section in sections {
            report.push_str(&section.content);
        }

        report
    }

    /// Get a list of registered provider names
    pub fn list_providers(&self) -> Vec<String> {
        if let Ok(providers) = self.providers.read() {
            providers.iter().map(|p| p.name().to_string()).collect()
        } else {
            Vec::new()
        }
    }

    /// Get debug summaries from all active providers
    pub fn collect_summaries(&self) -> Vec<(String, String)> {
        let mut summaries = Vec::new();

        if let Ok(providers) = self.providers.read() {
            for provider in providers.iter() {
                if provider.is_active() {
                    if let Some(summary) = provider.debug_summary() {
                        summaries.push((provider.name().to_string(), summary));
                    }
                }
            }
        }

        summaries
    }

    /// Check if a provider is registered
    pub fn has_provider(&self, name: &str) -> bool {
        if let Ok(providers) = self.providers.read() {
            providers.iter().any(|p| p.name() == name)
        } else {
            false
        }
    }

    /// Get the count of registered providers
    pub fn provider_count(&self) -> usize {
        if let Ok(providers) = self.providers.read() {
            providers.len()
        } else {
            0
        }
    }
}

impl Default for DebugRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for DebugRegistry {
    fn clone(&self) -> Self {
        Self {
            providers: Arc::clone(&self.providers),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProvider {
        name: String,
        active: bool,
    }

    impl DebugTrace for TestProvider {
        fn name(&self) -> &str {
            &self.name
        }

        fn debug_sections(&self) -> Vec<DebugSection> {
            vec![DebugSection::new(
                format!("{} Section", self.name),
                format!("Debug info from {}", self.name),
                100,
            )]
        }

        fn is_active(&self) -> bool {
            self.active
        }
    }

    #[test]
    fn test_registry_basic() {
        let registry = DebugRegistry::new();

        let provider1 = Arc::new(TestProvider {
            name: "Provider1".to_string(),
            active: true,
        });

        registry.register(provider1);
        assert_eq!(registry.provider_count(), 1);
        assert!(registry.has_provider("Provider1"));

        registry.unregister("Provider1");
        assert_eq!(registry.provider_count(), 0);
    }

    #[test]
    fn test_collect_sections() {
        let registry = DebugRegistry::new();

        let provider1 = Arc::new(TestProvider {
            name: "Provider1".to_string(),
            active: true,
        });

        let provider2 = Arc::new(TestProvider {
            name: "Provider2".to_string(),
            active: false, // Not active
        });

        registry.register(provider1);
        registry.register(provider2);

        let sections = registry.collect_debug_sections();
        assert_eq!(sections.len(), 1); // Only active provider's section
        assert!(sections[0].title.contains("Provider1"));
    }
}
