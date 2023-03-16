use std::ops::RangeBounds;

use super::gap_buffer::ChunkSummary;
use super::utils::*;
use crate::range_bounds_to_start_end;
use crate::tree::Summarize;

/// A slice of a [`GapBuffer`](super::gap_buffer::GapBuffer).
///
/// TODO: docs
#[derive(Copy, Clone, Default)]
pub(super) struct GapSlice<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) len_left: u16,
    pub(super) line_breaks_left: u16,
    pub(super) len_right: u16,
}

impl std::fmt::Debug for GapSlice<'_> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("\"")?;
        debug_no_quotes(self.first_segment(), f)?;
        write!(f, "{:~^1$}", "", self.len_gap())?;
        debug_no_quotes(self.second_segment(), f)?;
        f.write_str("\"")
    }
}

impl<'a> GapSlice<'a> {
    /// Returns the byte at the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds, i.e. greater than or equal to
    /// [`len()`](Self::len()).
    #[inline]
    pub(super) fn byte(&self, byte_index: usize) -> u8 {
        debug_assert!(byte_index < self.len());

        if byte_index < self.len_first_segment() {
            self.first_segment().as_bytes()[byte_index]
        } else {
            self.second_segment().as_bytes()
                [byte_index - self.len_first_segment()]
        }
    }

    #[inline]
    pub(super) fn truncate_trailing_line_break(&mut self) -> usize {
        if !self.has_trailing_newline() {
            return 0;
        }
        let bytes_line_break = bytes_line_break(self.last_segment());
        // let bytes_line_break = ;
        // first = first.byte_slice(..first.len() - bytes_line_break);

        todo!();
    }

    #[inline]
    fn byte_slice<R>(&self, byte_range: R) -> GapSlice<'a>
    where
        R: RangeBounds<usize>,
    {
        let (start, end) =
            range_bounds_to_start_end(byte_range, 0, self.len());

        debug_assert!(start <= end);
        debug_assert!(end <= self.len());

        match (
            start <= self.len_first_segment(),
            end <= self.len_first_segment(),
        ) {
            (true, true) => Self {
                bytes: &self.bytes[start..end],
                len_left: (end - start) as u16,
                line_breaks_left: todo!(),
                len_right: 0,
            },

            (true, false) => Self {
                bytes: &self.bytes[start..end + self.len_gap()],
                len_left: self.len_left - (start as u16),
                line_breaks_left: todo!(),
                len_right: (end as u16) - self.len_left,
            },

            (false, false) => Self {
                bytes: &self.bytes
                    [start + self.len_gap()..end + self.len_gap()],
                len_left: 0,
                line_breaks_left: todo!(),
                len_right: (end - start) as u16,
            },

            (false, true) => unreachable!(),
        }
    }

    /// Returns the byte offset of the start of the given line.
    #[inline]
    pub(super) fn byte_of_line(&self, line_index: usize) -> usize {
        if line_index <= self.line_breaks_left as usize {
            line_of_byte(self.first_segment(), line_index)
        } else {
            self.len_first_segment()
                + line_of_byte(
                    self.second_segment(),
                    line_index - self.line_breaks_left as usize,
                )
        }
    }

    #[inline]
    pub(super) fn empty() -> Self {
        Self::default()
    }

    #[inline]
    pub(super) fn first_segment(&self) -> &'a str {
        // SAFETY: this `GapSlice` was obtained by slicing a `GapBuffer` whose
        // first `len_first_segment` bytes were valid UTF-8.
        unsafe {
            std::str::from_utf8_unchecked(
                &self.bytes[..self.len_first_segment()],
            )
        }
    }

    /// Returns `true` if it ends with a newline (either LF or CRLF).
    #[inline]
    pub(super) fn has_trailing_newline(&self) -> bool {
        last_byte_is_newline(self.last_segment())
    }

    #[inline]
    pub(super) fn is_char_boundary(&self, byte_offset: usize) -> bool {
        debug_assert!(byte_offset <= self.len());

        if byte_offset <= self.len_first_segment() {
            self.first_segment().is_char_boundary(byte_offset)
        } else {
            self.second_segment()
                .is_char_boundary(byte_offset - self.len_first_segment())
        }
    }

    /// The second segment if it's not empty, or the first one otherwise.
    #[inline]
    pub(super) fn last_segment(&self) -> &'a str {
        if !self.second_segment().is_empty() {
            self.second_segment()
        } else {
            self.first_segment()
        }
    }

    #[inline]
    pub(super) fn len(&self) -> usize {
        self.len_first_segment() + self.len_second_segment()
    }

    #[inline]
    pub(super) fn len_first_segment(&self) -> usize {
        self.len_left as _
    }

    #[inline]
    fn len_gap(&self) -> usize {
        self.bytes.len() - self.len()
    }

    #[inline]
    pub(super) fn len_second_segment(&self) -> usize {
        self.len_right as _
    }

    #[inline]
    pub(super) fn second_segment(&self) -> &'a str {
        // SAFETY: this `GapSlice` was obtained by slicing a `GapBuffer` whose
        // last `len_second_segment` bytes were valid UTF-8.
        unsafe {
            std::str::from_utf8_unchecked(
                &self.bytes[self.bytes.len() - self.len_second_segment()..],
            )
        }
    }

    #[inline]
    pub(super) fn split_at_offset(
        &self,
        byte_offset: usize,
        tot_line_breaks: usize,
    ) -> (Self, Self) {
        (self.byte_slice(..byte_offset), self.byte_slice(byte_offset..))
    }
}

impl Summarize for GapSlice<'_> {
    type Summary = ChunkSummary;

    #[inline]
    fn summarize(&self) -> Self::Summary {
        let line_breaks = self.line_breaks_left as usize
            + count_line_breaks(self.second_segment());

        ChunkSummary { bytes: self.len(), line_breaks }
    }
}

#[cfg(test)]
mod tests {
    use crate::rope::gap_buffer::GapBuffer;
    use crate::tree::AsSlice;

    #[test]
    fn debug_slice() {
        let buffer = GapBuffer::<10>::from("Hello");
        assert_eq!("\"He~~~~~llo\"", format!("{:?}", buffer.as_slice()));
    }
}
