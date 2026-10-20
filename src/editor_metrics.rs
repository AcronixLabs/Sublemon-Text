#[derive(Default)]
pub struct LineMetrics {
    revision: u64,
    wrap_width_chars: usize,
    row_offsets: Option<Vec<usize>>,
    line_starts: Vec<usize>,
}

impl LineMetrics {
    pub fn new(text: &str, revision: u64, wrap_width_chars: usize) -> Self {
        let wrap_width_chars = wrap_width_chars.max(1);
        let wrapped = wrap_width_chars != usize::MAX;
        let mut row_offsets = wrapped.then(|| vec![0usize]);
        let mut line_starts = vec![0usize];
        let mut current_line_chars = 0usize;
        let mut current_visual_rows = 1usize;
        let mut character_offset = 0usize;

        for ch in text.chars() {
            if ch == '\n' {
                if let Some(offsets) = row_offsets.as_mut() {
                    offsets.push(offsets.last().copied().unwrap_or(0) + current_visual_rows);
                }
                line_starts.push(character_offset + 1);
                current_line_chars = 0;
                current_visual_rows = 1;
            } else {
                current_line_chars += 1;
                if wrapped {
                    current_visual_rows = current_line_chars.max(1).div_ceil(wrap_width_chars);
                }
            }
            character_offset += 1;
        }
        if let Some(offsets) = row_offsets.as_mut() {
            offsets.push(offsets.last().copied().unwrap_or(0) + current_visual_rows);
        }

        Self {
            revision,
            wrap_width_chars,
            row_offsets,
            line_starts,
        }
    }

    pub fn matches(&self, revision: u64, wrap_width_chars: usize) -> bool {
        self.revision == revision && self.matches_wrap_width(wrap_width_chars)
    }

    pub fn matches_wrap_width(&self, wrap_width_chars: usize) -> bool {
        self.wrap_width_chars == wrap_width_chars.max(1)
    }

    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    pub fn visual_rows(&self) -> usize {
        self.row_offsets
            .as_ref()
            .and_then(|rows| rows.last().copied())
            .unwrap_or_else(|| self.line_count())
            .max(1)
    }

    pub fn line_visual_rows(&self, line_index: usize) -> usize {
        self.row_offsets
            .as_ref()
            .and_then(|rows| {
                rows.get(line_index + 1)
                    .zip(rows.get(line_index))
                    .map(|(end, start)| end - start)
            })
            .unwrap_or_else(|| usize::from(line_index < self.line_count()))
    }

    pub fn visual_row_start(&self, line_index: usize) -> usize {
        self.row_offsets
            .as_ref()
            .and_then(|rows| rows.get(line_index).copied())
            .unwrap_or(line_index)
    }

    pub fn lines_in_visual_rows(&self, first_row: usize, end_row: usize) -> std::ops::Range<usize> {
        let line_count = self.line_count();
        let Some(row_offsets) = &self.row_offsets else {
            return first_row.min(line_count)
                ..end_row.min(line_count).max(first_row.min(line_count));
        };
        let first = row_offsets
            .partition_point(|row| *row <= first_row)
            .saturating_sub(1)
            .min(line_count);
        let end = row_offsets
            .partition_point(|row| *row < end_row)
            .min(line_count);
        first..end.max(first)
    }

    pub fn cursor_line_column(&self, character_offset: usize) -> (usize, usize) {
        let line_index = self
            .line_starts
            .partition_point(|start| *start <= character_offset)
            .saturating_sub(1);
        (
            line_index + 1,
            character_offset.saturating_sub(self.line_starts[line_index]) + 1,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::LineMetrics;

    #[test]
    fn maps_offsets_to_lines_and_columns() {
        let metrics = LineMetrics::new("abc\nβx\n", 3, 80);
        assert_eq!(metrics.cursor_line_column(0), (1, 1));
        assert_eq!(metrics.cursor_line_column(3), (1, 4));
        assert_eq!(metrics.cursor_line_column(4), (2, 1));
        assert_eq!(metrics.cursor_line_column(6), (2, 3));
        assert_eq!(metrics.cursor_line_column(7), (3, 1));
    }

    #[test]
    fn caches_wrapped_rows_for_each_line() {
        let metrics = LineMetrics::new("123456789\nx\n", 7, 4);
        assert_eq!(metrics.line_count(), 3);
        assert_eq!(metrics.line_visual_rows(0), 3);
        assert_eq!(metrics.line_visual_rows(1), 1);
        assert_eq!(metrics.visual_rows(), 5);
        assert!(metrics.matches(7, 4));
        assert!(!metrics.matches(8, 4));
        assert!(!metrics.matches(7, 5));
    }

    #[test]
    fn large_no_wrap_metrics_use_compact_line_index() {
        let text = (0..100_000)
            .map(|line| format!("line {line}"))
            .collect::<Vec<_>>()
            .join("\n");
        let metrics = LineMetrics::new(&text, 1, usize::MAX);
        assert_eq!(metrics.line_count(), 100_000);
        assert_eq!(
            metrics.cursor_line_column(text.chars().count()),
            (100_000, 12)
        );
        assert!(metrics.matches(1, usize::MAX));
    }
}
