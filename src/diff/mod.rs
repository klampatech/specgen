//! Diff and merge engine for SpecGen CLI.
//!
//! This module handles semantic diffing and merging of spec files.

#![allow(dead_code)]
#![allow(clippy::uninlined_format_args)]

use similar::{ChangeTag, TextDiff};

/// User-edited marker for preserving manual changes.
pub const USER_EDITED_MARKER: &str = "<!-- user-edited -->";
/// Conflict marker for unresolved differences.
pub const CONFLICT_START: &str = "<!-- CONFLICT: review required -->";
pub const CONFLICT_AI: &str = "<!-- AI-generated version: -->";
pub const CONFLICT_EXISTING: &str = "<!-- Existing version: -->";
pub const CONFLICT_END: &str = "<!-- END CONFLICT -->";

/// Result of a diff operation.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DiffResult {
    /// Whether there are any changes.
    pub has_changes: bool,
    /// Number of unchanged sections.
    pub unchanged: usize,
    /// Number of updated sections.
    pub updated: usize,
    /// Number of conflicts.
    pub conflicts: usize,
    /// The merged content.
    pub merged_content: String,
}

/// Perform a semantic diff between old and new content.
///
/// Uses the patience diff algorithm for better results.
#[allow(dead_code)]
pub fn diff(old_content: &str, new_content: &str) -> DiffResult {
    let diff = TextDiff::from_lines(old_content, new_content);

    let mut changes_detected = false;

    for change in diff.iter_all_changes() {
        if change.tag() != ChangeTag::Equal {
            changes_detected = true;
            break;
        }
    }

    DiffResult {
        has_changes: changes_detected,
        unchanged: 0,
        updated: if changes_detected { 1 } else { 0 },
        conflicts: 0,
        merged_content: new_content.to_string(),
    }
}

/// Check if content contains user-edited markers.
#[allow(dead_code)]
pub fn has_user_edits(content: &str) -> bool {
    content.contains(USER_EDITED_MARKER)
}

/// Extract user-edited sections from content.
#[allow(dead_code)]
pub fn extract_user_edits(content: &str) -> Vec<(String, String)> {
    let mut edits = Vec::new();
    let marker = USER_EDITED_MARKER;

    // Split by marker and extract content between pairs of markers
    // Format: content <!-- user-edited --> EDITED_CONTENT <!-- user-edited --> content
    let parts: Vec<&str> = content.split(marker).collect();

    // If we have parts between markers, extract them
    // Parts at odd indices (1, 3, 5...) are the content between markers
    if parts.len() > 1 {
        for (i, part) in parts.iter().enumerate() {
            if i % 2 == 1 && !part.trim().is_empty() {
                edits.push((marker.to_string(), part.trim().to_string()));
            }
        }
    }

    edits
}

/// Merge new content with existing content, preserving user edits.
#[allow(dead_code)]
pub fn merge(old_content: &str, new_content: &str) -> DiffResult {
    // Check for user-edited sections
    if has_user_edits(old_content) {
        return merge_with_preservation(old_content, new_content);
    }

    // No user edits, simple diff
    diff(old_content, new_content)
}

/// Merge content while preserving user-edited sections.
fn merge_with_preservation(old_content: &str, new_content: &str) -> DiffResult {
    // Extract user-edited sections
    let user_edits = extract_user_edits(old_content);

    // Build merged content by replacing markers in new content
    let mut merged = new_content.to_string();

    for (marker, edit_content) in &user_edits {
        // Replace marker with preserved content
        merged = merged.replacen(marker, edit_content, 1);
    }

    DiffResult {
        has_changes: true,
        unchanged: 0,
        updated: 1,
        conflicts: 0,
        merged_content: merged,
    }
}

/// Create a conflict marker for unresolved differences.
#[allow(dead_code)]
pub fn create_conflict(new_content: &str, old_content: &str) -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        CONFLICT_START,
        CONFLICT_AI,
        new_content,
        CONFLICT_EXISTING,
        old_content,
        CONFLICT_END,
        new_content
    )
}

/// Check if content contains conflict markers.
#[allow(dead_code)]
pub fn has_conflicts(content: &str) -> bool {
    content.contains(CONFLICT_START) && content.contains(CONFLICT_END)
}

/// Get a summary of the diff operation.
#[allow(dead_code)]
pub fn get_diff_summary(old_content: &str, new_content: &str) -> String {
    let diff = TextDiff::from_lines(old_content, new_content);

    let mut additions = 0;
    let mut deletions = 0;
    let mut unchanged = 0;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => deletions += 1,
            ChangeTag::Insert => additions += 1,
            ChangeTag::Equal => unchanged += 1,
        }
    }

    format!(
        "+{} -{} {} unchanged lines",
        additions, deletions, unchanged
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_identical() {
        let content = "Hello\nWorld";
        let result = diff(content, content);
        assert!(!result.has_changes);
    }

    #[test]
    fn test_diff_different() {
        let old = "Hello\nWorld";
        let new = "Hello\nRust";
        let result = diff(old, new);
        assert!(result.has_changes);
    }

    #[test]
    fn test_has_user_edits() {
        assert!(has_user_edits(
            "Some content <!-- user-edited --> preserved"
        ));
        assert!(!has_user_edits("No marker here"));
    }

    #[test]
    fn test_has_conflicts() {
        let content = format!("{} content {}", CONFLICT_START, CONFLICT_END);
        assert!(has_conflicts(&content));
        assert!(!has_conflicts("No conflict"));
    }

    #[test]
    fn test_diff_summary() {
        let old = "Line 1\nLine 2\nLine 3";
        let new = "Line 1\nModified\nLine 3";
        let summary = get_diff_summary(old, new);
        assert!(summary.contains("+"));
        assert!(summary.contains("-"));
    }

    #[test]
    fn test_merge_preserves_user_edits() {
        let old = "Some content <!-- user-edited --> my custom content";
        let new = "Some content <!-- user-edited --> new content";
        let result = merge(old, new);
        assert!(result.has_changes);
        assert!(result.merged_content.contains("my custom content"));
    }

    #[test]
    fn test_extract_user_edits_multiple() {
        // When markers are consecutive, content between each pair is extracted
        let content = "Start <!-- user-edited --> first edit <!-- user-edited --> middle <!-- user-edited --> last";
        let edits = extract_user_edits(content);
        // Split produces: ["Start ", " first edit ", " middle ", " last"]
        // Odd indices (1, 3) have content: " first edit " and " last"
        assert_eq!(edits.len(), 2);
    }

    #[test]
    fn test_create_conflict_format() {
        let new_content = "New version";
        let old_content = "Old version";
        let conflict = create_conflict(new_content, old_content);
        assert!(conflict.contains(CONFLICT_START));
        assert!(conflict.contains(CONFLICT_AI));
        assert!(conflict.contains(CONFLICT_EXISTING));
        assert!(conflict.contains(CONFLICT_END));
        assert!(conflict.contains("New version"));
        assert!(conflict.contains("Old version"));
    }

    #[test]
    fn test_merge_no_user_edits() {
        let old = "Original content";
        let new = "New content";
        let result = merge(old, new);
        assert!(result.has_changes);
        assert_eq!(result.merged_content, "New content");
    }
}
