use core::cmp::Ordering;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum SplitterAction {
    Split,
    Skip,
    Reset,
    ManualSplit,
}

pub fn should_split(b: bool) -> Option<SplitterAction> {
    if b {
        Some(SplitterAction::Split)
    } else {
        None
    }
}

/// Splits when equal, skips when greater than expected
pub fn reached_up_to_split<T: PartialOrd>(expected: T, actual: Result<T, asr::Error>) -> Option<SplitterAction> {
    match actual.ok()?.partial_cmp(&expected)? {
        Ordering::Equal => Some(SplitterAction::Split),
        Ordering::Greater => Some(SplitterAction::Skip),
        _ => None
    }
}
