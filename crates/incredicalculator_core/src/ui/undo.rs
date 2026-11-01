/// How many edits can be undone. Older edits are dropped once this is reached.
pub const UNDO_DEPTH: usize = 16;

/// The text and cursor of a text input at one point in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSnapshot<const N: usize> {
    /// Zero padded after `len`.
    pub data: [u8; N],
    pub len: usize,
    pub cursor: usize,
}

impl<const N: usize> TextSnapshot<N> {
    const EMPTY: Self = Self {
        data: [0; N],
        len: 0,
        cursor: 0,
    };

    /// `text` is truncated to `N` bytes and `cursor` is clamped to the text.
    pub fn new(text: &[u8], cursor: usize) -> Self {
        let len = text.len().min(N);
        let mut data = [0; N];
        data[..len].copy_from_slice(&text[..len]);
        Self {
            data,
            len,
            cursor: cursor.min(len),
        }
    }

    pub fn text(&self) -> &[u8] {
        &self.data[..self.len]
    }
}

/// Fixed-capacity undo stack for a text input. Uses no heap, so it works the
/// same on the embedded target. When full, recording drops the oldest edit.
pub struct UndoHistory<const N: usize> {
    entries: [TextSnapshot<N>; UNDO_DEPTH],
    // index of the slot the next recorded snapshot goes into
    next: usize,
    len: usize,
}

impl<const N: usize> UndoHistory<N> {
    pub const fn new() -> Self {
        Self {
            entries: [TextSnapshot::EMPTY; UNDO_DEPTH],
            next: 0,
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        self.next = 0;
        self.len = 0;
    }

    /// Records `snapshot` as the state to return to on the next undo.
    pub fn push(&mut self, snapshot: TextSnapshot<N>) {
        self.entries[self.next] = snapshot;
        self.next = (self.next + 1) % UNDO_DEPTH;
        if self.len < UNDO_DEPTH {
            self.len += 1;
        }
    }

    /// Records `before` if an edit changed the text from it to `after_text`.
    /// Cursor-only moves are not recorded.
    pub fn record_if_changed(&mut self, before: TextSnapshot<N>, after_text: &[u8]) {
        if before.text() != after_text {
            self.push(before);
        }
    }

    /// Removes and returns the most recently recorded snapshot.
    pub fn pop(&mut self) -> Option<TextSnapshot<N>> {
        if self.len == 0 {
            return None;
        }

        self.next = (self.next + UNDO_DEPTH - 1) % UNDO_DEPTH;
        self.len -= 1;
        Some(self.entries[self.next])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pop_returns_snapshots_newest_first() {
        let mut history = UndoHistory::<8>::new();
        history.push(TextSnapshot::new(b"", 0));
        history.push(TextSnapshot::new(b"1", 1));
        history.push(TextSnapshot::new(b"12", 1));

        let snapshot = history.pop().unwrap();
        assert_eq!(snapshot.text(), b"12");
        assert_eq!(snapshot.cursor, 1);
        assert_eq!(history.pop().unwrap().text(), b"1");
        assert_eq!(history.pop().unwrap().text(), b"");
        assert_eq!(history.pop(), None);
    }

    #[test]
    fn full_history_drops_oldest_snapshot() {
        let mut history = UndoHistory::<4>::new();
        for i in 0..UNDO_DEPTH + 3 {
            history.push(TextSnapshot::new(&[b'a' + i as u8], 1));
        }

        for i in (3..UNDO_DEPTH + 3).rev() {
            assert_eq!(history.pop().unwrap().text(), &[b'a' + i as u8]);
        }
        assert!(history.pop().is_none());
    }

    #[test]
    fn push_after_pop_reuses_slots() {
        let mut history = UndoHistory::<4>::new();
        history.push(TextSnapshot::new(b"a", 1));
        history.push(TextSnapshot::new(b"b", 1));
        history.pop();
        history.push(TextSnapshot::new(b"c", 1));

        assert_eq!(history.pop().unwrap().text(), b"c");
        assert_eq!(history.pop().unwrap().text(), b"a");
        assert!(history.pop().is_none());
    }

    #[test]
    fn record_if_changed_ignores_cursor_only_moves() {
        let mut history = UndoHistory::<4>::new();
        history.record_if_changed(TextSnapshot::new(b"12", 2), b"12");
        assert!(history.pop().is_none());

        history.record_if_changed(TextSnapshot::new(b"12", 2), b"123");
        assert_eq!(history.pop().unwrap().text(), b"12");
    }

    #[test]
    fn snapshot_truncates_and_clamps_cursor() {
        let snapshot = TextSnapshot::<2>::new(b"abc", 3);
        assert_eq!(snapshot.text(), b"ab");
        assert_eq!(snapshot.cursor, 2);
    }

    #[test]
    fn clear_empties_history() {
        let mut history = UndoHistory::<4>::new();
        history.push(TextSnapshot::new(b"a", 1));
        history.clear();
        assert!(history.pop().is_none());
    }
}
