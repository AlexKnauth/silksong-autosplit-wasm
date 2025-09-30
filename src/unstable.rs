/// Accesses the index of the split the attempt is currently on.
/// If there's no attempt in progress, `None` is returned instead.
/// This returns an index that is equal to the amount of segments
/// when the attempt is finished, but has not been reset.
/// So you need to be careful when using this value for indexing.
/// Same index does not imply same split on undo and then split.
pub fn timer_current_split_index() -> Option<u64> {
    #[cfg(feature = "split-index")]
    return asr::timer::current_split_index();
    #[allow(unreachable_code)]
    None
}

/// Whether the segment at `idx` was splitted this attempt.
/// Returns `Some(true)` if the segment was splitted,
/// or `Some(false)` if skipped.
/// If `idx` is greater than or equal to the current split index,
/// `None` is returned instead.
#[cfg(feature = "split-index")]
pub fn timer_segment_splitted(_idx: u64) -> Option<bool> {
    asr::timer::segment_splitted(_idx)
}
