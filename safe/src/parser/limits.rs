use std::collections::BTreeSet;

use crate::ffi::types::EXIF_IFD_COUNT;

pub(crate) const MAX_APP1_LENGTH: usize = 0xfffe;
pub(crate) const MAX_RECURSION_DEPTH: usize = 8;
pub(crate) const MAX_VISITED_OFFSETS: usize = 64;
pub(crate) const MAX_PARSE_WORK: usize = 1 << 20;
pub(crate) const MAX_SERIALIZED_BYTES: usize = u32::MAX as usize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParseError {
    Corrupt(&'static str),
    Overflow(&'static str),
    ResourceLimit(&'static str),
}

pub(crate) struct ParseBudget {
    remaining_work: usize,
    visited_offsets: BTreeSet<u32>,
}

impl ParseBudget {
    pub(crate) fn new(input_size: usize) -> Self {
        let remaining_work = input_size
            .saturating_mul(8)
            .saturating_add(1024)
            .min(MAX_PARSE_WORK)
            .max(256);

        Self {
            remaining_work,
            visited_offsets: BTreeSet::new(),
        }
    }

    pub(crate) fn charge_ifd(&mut self, entry_count: usize) -> Result<(), ParseError> {
        let units = entry_count
            .saturating_add(level_cost(entry_count))
            .saturating_add(1);
        self.charge(units, "EXIF parse-work budget exhausted")
    }

    pub(crate) fn charge_work(
        &mut self,
        units: usize,
        context: &'static str,
    ) -> Result<(), ParseError> {
        self.charge(units, context)
    }

    pub(crate) fn record_offset(&mut self, offset: u32) -> Result<(), ParseError> {
        if self.visited_offsets.contains(&offset) {
            return Err(ParseError::ResourceLimit("Linked IFD cycle detected"));
        }
        if self.visited_offsets.len() >= MAX_VISITED_OFFSETS {
            return Err(ParseError::ResourceLimit(
                "Too many linked IFD offsets encountered",
            ));
        }
        self.visited_offsets.insert(offset);
        Ok(())
    }

    fn charge(&mut self, units: usize, context: &'static str) -> Result<(), ParseError> {
        if units > self.remaining_work {
            self.remaining_work = 0;
            return Err(ParseError::ResourceLimit(context));
        }
        self.remaining_work -= units;
        Ok(())
    }
}

pub(crate) fn checked_add(
    lhs: usize,
    rhs: usize,
    context: &'static str,
) -> Result<usize, ParseError> {
    lhs.checked_add(rhs).ok_or(ParseError::Overflow(context))
}

pub(crate) fn checked_mul(
    lhs: usize,
    rhs: usize,
    context: &'static str,
) -> Result<usize, ParseError> {
    lhs.checked_mul(rhs).ok_or(ParseError::Overflow(context))
}

pub(crate) fn ifd_index(ifd: i32) -> Result<usize, ParseError> {
    if (0..EXIF_IFD_COUNT).contains(&ifd) {
        Ok(ifd as usize)
    } else {
        Err(ParseError::Corrupt("Invalid IFD index"))
    }
}

fn level_cost(entry_count: usize) -> usize {
    if entry_count <= 1 {
        return 1;
    }

    let numerator = (entry_count as f64 + 0.1).ln();
    let denominator = 1.1f64.ln();
    (numerator / denominator).ceil() as usize
}
