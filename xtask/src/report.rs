use std::collections::BTreeSet;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FindingSet {
    pub stale: BTreeSet<String>,
    pub missing: BTreeSet<String>,
}

impl FindingSet {
    pub fn is_empty(&self) -> bool {
        self.stale.is_empty() && self.missing.is_empty()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FlatFindingSet {
    pub entries: BTreeSet<String>,
}

impl FlatFindingSet {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
