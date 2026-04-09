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
