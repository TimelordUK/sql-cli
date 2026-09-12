//! What the completer offers, as a value rather than a bare string (T3).
//!
//! A `Vec<String>` could not say three things the editor needs to know, and
//! the first of them is what forced this: **the text to insert is not always
//! the text to show**. A column has to be inserted quoted (`"name.common"`)
//! but read back unquoted, and a value has to be inserted quoted or bare
//! depending on where the cursor is, while the user is always looking at,
//! and typing, the plain name.
//!
//! So `insert` is for the buffer and `label` is for the human — including for
//! matching what they have typed so far, which is why filtering compares
//! against the label and no longer has to strip quotes back off.

use crate::sql::identifier::quote_if_needed;

/// What kind of thing a suggestion is, for ranking and for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionKind {
    Column,
    Table,
    /// A SQL function, inserted with its opening parenthesis.
    Function,
    /// A method call on a column: `Contains('')`.
    Method,
    Keyword,
    /// A value from the data, for `WHERE region = <tab>` (T4).
    Value,
}

/// One thing the completer can offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// The text spliced into the query when this is accepted. It replaces
    /// `query[replace_start..cursor_pos]`.
    pub insert: String,
    /// What the user sees, and what their typing is matched against. For a
    /// column that is the unquoted name.
    pub label: String,
    pub kind: SuggestionKind,
    /// Shown beside the label: a row count for a value, a description for a
    /// function. Never inserted.
    pub detail: Option<String>,
}

impl Suggestion {
    /// A suggestion whose inserted text is also its label.
    pub fn new(kind: SuggestionKind, text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            label: text.clone(),
            insert: text,
            kind,
            detail: None,
        }
    }

    /// A column, quoted for insertion where the lexer would not read the bare
    /// name back as one identifier (T8), and labelled with the plain name.
    pub fn column(name: &str) -> Self {
        Self {
            insert: quote_if_needed(name),
            label: name.to_string(),
            kind: SuggestionKind::Column,
            detail: None,
        }
    }

    /// A column whose text has already been written by someone else - the
    /// SELECT list an ORDER BY completion is echoing back, say - so it may
    /// already carry quotes. The label is the name without them.
    pub fn column_text(text: &str) -> Self {
        Self {
            label: strip_identifier_quotes(text).to_string(),
            insert: text.to_string(),
            kind: SuggestionKind::Column,
            detail: None,
        }
    }

    pub fn table(name: impl Into<String>) -> Self {
        Self::new(SuggestionKind::Table, name)
    }

    pub fn keyword(text: impl Into<String>) -> Self {
        Self::new(SuggestionKind::Keyword, text)
    }

    /// A function, inserted with its opening parenthesis (`ROUND(`). T10 will
    /// build these from the registry, which is where the `detail` comes from.
    pub fn function(text: impl Into<String>) -> Self {
        Self::new(SuggestionKind::Function, text)
    }

    pub fn method(text: impl Into<String>) -> Self {
        Self::new(SuggestionKind::Method, text)
    }

    /// A value from the data. `insert` and `label` differ whenever the cursor
    /// is somewhere the value needs quoting (T4).
    pub fn value(insert: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            insert: insert.into(),
            label: label.into(),
            kind: SuggestionKind::Value,
            detail: None,
        }
    }

    #[must_use]
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Label plus detail, for the status line: `Americas (56 rows)`.
    #[must_use]
    pub fn display_text(&self) -> String {
        match &self.detail {
            Some(detail) => format!("{} ({})", self.label, detail),
            None => self.label.clone(),
        }
    }

    /// Does this suggestion continue what the user has typed? Case-insensitive,
    /// and against the label, so `na` and `"na` both reach `name.common`.
    #[must_use]
    pub fn matches_prefix(&self, prefix: &str) -> bool {
        self.label
            .to_lowercase()
            .starts_with(&prefix.to_lowercase())
    }
}

/// Strip the surrounding quotes from a quoted identifier so it can be compared
/// against what the user typed.
fn strip_identifier_quotes(text: &str) -> &str {
    text.strip_prefix('"')
        .map_or(text, |rest| rest.strip_suffix('"').unwrap_or(rest))
}

/// Build suggestions of one kind from text that is already final.
pub fn all(
    kind: SuggestionKind,
    texts: impl IntoIterator<Item = impl Into<String>>,
) -> Vec<Suggestion> {
    texts
        .into_iter()
        .map(|text| Suggestion::new(kind, text))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_column_is_inserted_quoted_and_shown_plain() {
        let dotted = Suggestion::column("name.common");
        assert_eq!(dotted.insert, "\"name.common\"");
        assert_eq!(dotted.label, "name.common");

        // A name the lexer reads back as one identifier needs no quotes, and
        // then the two halves agree.
        let plain = Suggestion::column("region");
        assert_eq!(plain.insert, "region");
        assert_eq!(plain.label, "region");
    }

    #[test]
    fn typing_matches_the_label_whether_or_not_quotes_are_involved() {
        let column = Suggestion::column("name.common");
        assert!(column.matches_prefix("na"));
        assert!(column.matches_prefix("NAME.COM"));
        assert!(!column.matches_prefix("\"na"));

        // Already-quoted text from a SELECT list matches the same way.
        assert!(Suggestion::column_text("\"name.common\"").matches_prefix("na"));
    }

    #[test]
    fn detail_is_shown_but_never_inserted() {
        let value = Suggestion::value("'Americas'", "Americas").with_detail("56 rows");
        assert_eq!(value.insert, "'Americas'");
        assert_eq!(value.display_text(), "Americas (56 rows)");
    }
}
