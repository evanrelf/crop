use std::ops::{Add, AddAssign, Range, Sub, SubAssign};

use super::{TextChunk, TextSlice, TextSummary};
use crate::tree::Metric;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ByteMetric(pub(super) usize);

impl Add for ByteMetric {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for ByteMetric {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl AddAssign for ByteMetric {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0
    }
}

impl SubAssign for ByteMetric {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0
    }
}

impl Metric<TextChunk> for ByteMetric {
    #[inline]
    fn zero() -> Self {
        Self(0)
    }

    #[inline]
    fn one() -> Self {
        Self(1)
    }

    #[inline]
    fn measure(summary: &TextSummary) -> Self {
        Self(summary.bytes)
    }

    #[inline]
    fn split_left<'a>(
        chunk: &'a TextSlice,
        ByteMetric(up_to): Self,
        summary: &TextSummary,
    ) -> (
        &'a TextSlice,
        Option<TextSummary>,
        Option<&'a TextSlice>,
        Option<TextSummary>,
    ) {
        if up_to == chunk.len() {
            (chunk, None, None, None)
        } else {
            (chunk[..up_to].into(), None, Some(chunk[up_to..].into()), None)
        }
    }

    #[inline]
    fn split_right(
        chunk: &TextSlice,
        ByteMetric(from): Self,
    ) -> (Option<&TextSlice>, &TextSlice) {
        if from == 0 {
            (None, chunk)
        } else {
            (Some(chunk[..from].into()), chunk[from..].into())
        }
    }

    #[inline]
    fn slice(chunk: &TextSlice, range: Range<Self>) -> &TextSlice {
        chunk[range.start.0..range.end.0].into()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct LineMetric(pub(super) usize);

impl Add for LineMetric {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for LineMetric {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl AddAssign for LineMetric {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0
    }
}

impl SubAssign for LineMetric {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0
    }
}

impl Metric<TextChunk> for LineMetric {
    #[inline]
    fn zero() -> Self {
        Self(0)
    }

    #[inline]
    fn one() -> Self {
        Self(1)
    }

    #[inline]
    fn measure(summary: &TextSummary) -> Self {
        Self(summary.line_breaks)
    }

    #[inline]
    fn split_left<'a>(
        chunk: &'a TextSlice,
        LineMetric(up_to): Self,
        summary: &TextSummary,
    ) -> (
        &'a TextSlice,
        Option<TextSummary>,
        Option<&'a TextSlice>,
        Option<TextSummary>,
    ) {
        let lf_plus_one = str_indices::lines_lf::to_byte_idx(chunk, up_to);

        let (left, left_summary) = {
            // If the newline is preceded by a carriage return we have to skip
            // it.
            let skip = if (lf_plus_one > 1)
                && (chunk.as_bytes()[lf_plus_one - 2] == b'\r')
            {
                2
            } else {
                1
            };

            let left_bytes = lf_plus_one - skip;

            (
                chunk[..left_bytes].into(),
                Some(TextSummary {
                    bytes: left_bytes,
                    line_breaks: up_to - 1,
                }),
            )
        };

        let (right, right_summary) = if lf_plus_one == chunk.len() {
            (None, None)
        } else {
            (
                Some(chunk[lf_plus_one..].into()),
                Some(TextSummary {
                    bytes: chunk.len() - lf_plus_one,
                    line_breaks: summary.line_breaks - up_to,
                }),
            )
        };

        (left, left_summary, right, right_summary)
    }

    #[inline]
    fn split_right(
        chunk: &TextSlice,
        LineMetric(from): Self,
    ) -> (Option<&TextSlice>, &TextSlice) {
        todo!()
    }

    #[inline]
    fn slice(chunk: &TextSlice, range: Range<Self>) -> &TextSlice {
        todo!()
    }
}
