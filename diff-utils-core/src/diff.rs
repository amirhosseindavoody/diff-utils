//! Side-by-side diff computation for diff-utils.
//!
//! Given two texts, produce a pair of aligned row vectors (`DiffSide`) so the
//! TUI can render file A on the left and file B on the right with matching
//! rows. Equal rows appear on both sides; deletions appear only on the left;
//! insertions appear only on the right.

use similar::{ChangeTag, TextDiff};

/// Classification of a single row within one side of the diff.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    Equal,
    Changed,
    Added,
    Removed,
    Blank,
}

/// A single rendered row on one side of the diff.
#[derive(Debug, Clone)]
pub struct DiffRow {
    pub kind: RowKind,
    pub text: String,
    /// 1-based line number in the source file, when applicable.
    pub line_no: Option<usize>,
}

/// One side of a side-by-side diff (left = old, right = new).
#[derive(Debug, Clone, Default)]
pub struct DiffSide {
    pub rows: Vec<DiffRow>,
}

impl DiffSide {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn get(&self, idx: usize) -> Option<&DiffRow> {
        self.rows.get(idx)
    }
}

/// A complete side-by-side diff: `left` mirrors `right` row-for-row.
#[derive(Debug, Clone, Default)]
pub struct SideBySide {
    pub left: DiffSide,
    pub right: DiffSide,
}

impl SideBySide {
    /// Number of aligned rows (both sides always have the same length).
    pub fn len(&self) -> usize {
        self.left.len()
    }

    pub fn is_empty(&self) -> bool {
        self.left.is_empty()
    }
}

/// Compute a side-by-side diff from two texts.
///
/// Lines are compared as whole lines (no intra-line word diff) which keeps the
/// aligned row model simple and fast even on large files.
pub fn diff_lines(old: &str, new: &str) -> SideBySide {
    let text_diff = TextDiff::from_lines(old, new);

    let mut left = DiffSide::default();
    let mut right = DiffSide::default();

    let mut old_no = 0usize;
    let mut new_no = 0usize;

    for change in text_diff.iter_all_changes() {
        // similar yields lines including their trailing newline; strip it so the
        // TUI can render each row on a single line.
        let value = change.value();
        let text = value.strip_suffix('\n').unwrap_or(value).to_string();

        match change.tag() {
            ChangeTag::Equal => {
                old_no += 1;
                new_no += 1;
                left.rows.push(DiffRow {
                    kind: RowKind::Equal,
                    text: text.clone(),
                    line_no: Some(old_no),
                });
                right.rows.push(DiffRow {
                    kind: RowKind::Equal,
                    text,
                    line_no: Some(new_no),
                });
            }
            ChangeTag::Delete => {
                old_no += 1;
                left.rows.push(DiffRow {
                    kind: RowKind::Removed,
                    text,
                    line_no: Some(old_no),
                });
                right.rows.push(DiffRow {
                    kind: RowKind::Blank,
                    text: String::new(),
                    line_no: None,
                });
            }
            ChangeTag::Insert => {
                new_no += 1;
                left.rows.push(DiffRow {
                    kind: RowKind::Blank,
                    text: String::new(),
                    line_no: None,
                });
                right.rows.push(DiffRow {
                    kind: RowKind::Added,
                    text,
                    line_no: Some(new_no),
                });
            }
        }
    }

    // similar's `Change` replacements (modified regions) surface as a Delete
    // run followed by an Insert run. Re-align those into Changed pairs so the
    // two sides stay row-for-row in sync and the user sees edits inline.
    realign_changes(&mut left, &mut right);

    SideBySide { left, right }
}

/// Collapse adjacent Removed/Blank + Blank/Added pairs into Changed rows so
/// edits line up across the two panels instead of producing staggered blanks.
fn realign_changes(left: &mut DiffSide, right: &mut DiffSide) {
    let n = left.rows.len();
    let mut new_left: Vec<DiffRow> = Vec::with_capacity(n);
    let mut new_right: Vec<DiffRow> = Vec::with_capacity(n);

    let mut i = 0;
    while i < n {
        let l = &left.rows[i];
        let r = &right.rows[i];

        if l.kind == RowKind::Removed && r.kind == RowKind::Blank {
            // Find the matching Removed run on the left that follows.
            let mut j = i;
            while j < n
                && left.rows[j].kind == RowKind::Removed
                && right.rows[j].kind == RowKind::Blank
            {
                j += 1;
            }
            let del_end = j;
            let mut add_end = j;
            while add_end < n
                && left.rows[add_end].kind == RowKind::Blank
                && right.rows[add_end].kind == RowKind::Added
            {
                add_end += 1;
            }

            if add_end == del_end {
                // Pure deletion with no following insert run: keep as Removed.
                new_left.push(l.clone());
                new_right.push(r.clone());
                i += 1;
                continue;
            }

            let dels = &left.rows[i..del_end];
            let adds = &right.rows[del_end..add_end];

            let max = dels.len().max(adds.len());
            for k in 0..max {
                let lrow = dels.get(k).cloned().map(|mut d| {
                    d.kind = RowKind::Changed;
                    d
                });
                let rrow = adds.get(k).cloned().map(|mut a| {
                    a.kind = RowKind::Changed;
                    a
                });
                new_left.push(lrow.unwrap_or(DiffRow {
                    kind: RowKind::Blank,
                    text: String::new(),
                    line_no: None,
                }));
                new_right.push(rrow.unwrap_or(DiffRow {
                    kind: RowKind::Blank,
                    text: String::new(),
                    line_no: None,
                }));
            }

            i = add_end;
        } else {
            new_left.push(l.clone());
            new_right.push(r.clone());
            i += 1;
        }
    }

    left.rows = new_left;
    right.rows = new_right;
}

/// Summary counts for a diff, used in status bars.
#[derive(Debug, Clone, Default)]
pub struct DiffStats {
    pub equal: usize,
    pub added: usize,
    pub removed: usize,
    pub changed: usize,
}

impl SideBySide {
    pub fn stats(&self) -> DiffStats {
        let mut stats = DiffStats::default();
        for (l, r) in self.left.rows.iter().zip(self.right.rows.iter()) {
            match (l.kind, r.kind) {
                (RowKind::Equal, RowKind::Equal) => stats.equal += 1,
                (RowKind::Removed, RowKind::Blank) => stats.removed += 1,
                (RowKind::Blank, RowKind::Added) => stats.added += 1,
                (RowKind::Changed, _) | (_, RowKind::Changed) => stats.changed += 1,
                _ => {}
            }
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_texts_align() {
        let d = diff_lines("a\nb\nc\n", "a\nb\nc\n");
        assert_eq!(d.len(), 3);
        assert!(d.left.rows.iter().all(|r| r.kind == RowKind::Equal));
        assert!(d.right.rows.iter().all(|r| r.kind == RowKind::Equal));
    }

    #[test]
    fn pure_insert_becomes_right_side_add() {
        let d = diff_lines("a\n", "a\nb\n");
        assert_eq!(d.len(), 2);
        assert_eq!(d.right.rows[1].kind, RowKind::Added);
        assert_eq!(d.left.rows[1].kind, RowKind::Blank);
    }

    #[test]
    fn pure_delete_becomes_left_side_remove() {
        let d = diff_lines("a\nb\n", "a\n");
        assert_eq!(d.len(), 2);
        assert_eq!(d.left.rows[1].kind, RowKind::Removed);
        assert_eq!(d.right.rows[1].kind, RowKind::Blank);
    }

    #[test]
    fn changed_lines_realign_to_changed_kind() {
        let d = diff_lines("hello\n", "world\n");
        assert_eq!(d.len(), 1);
        assert_eq!(d.left.rows[0].kind, RowKind::Changed);
        assert_eq!(d.right.rows[0].kind, RowKind::Changed);
    }

    #[test]
    fn both_sides_have_equal_length() {
        let d = diff_lines("a\nb\nc\nd\n", "a\nX\nc\nY\ne\n");
        assert_eq!(d.left.len(), d.right.len());
    }
}
