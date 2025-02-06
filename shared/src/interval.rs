use anyhow::bail;

// dict to convert string interval from exchange to inner representation and back
pub struct ExchangeIntervals;
impl ValueLabel for ExchangeIntervals {
    type Value = IntervalsDict;
}

// dict to convert inner representation to database string interval and back
pub struct DatabaseIntervals;
impl ValueLabel for DatabaseIntervals {
    type Value = IntervalsDict;
}

// TODO: move to config
pub fn database_intervals() -> anyhow::Result<IntervalsDict> {
    IntervalsDict::default()
        .with(IntervalKind::Minute, [(1, "1m"), (15, "15m")])?
        .with(IntervalKind::Hour, [(1, "1h")])?
        .with(IntervalKind::Day, [(1, "1d")])
}

use crate::utils::{
    ValueLabel,
    sorted_vec::{Entry, SortedVec},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Interval {
    pub kind: IntervalKind,
    pub value: u8,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
#[repr(u8)]
pub enum IntervalKind {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

// Two-way dictionary, implemented on top of two sorted Vecs with binary search
// should be optimal for small number of entries, rare inserts, frequent gets
// TODO: encapsulate dict into separate type, check performance, consifer alternatives
#[derive(Debug, Default)]
pub struct IntervalsDict {
    // assume sorted
    kinds: SortedVec<Interval, String>,
    aliases: SortedVec<String, Interval>,
}

impl IntervalsDict {
    pub fn add(&mut self, interval: Interval, alias: String) -> anyhow::Result<()> {
        match self.kinds.entry(interval) {
            Entry::Occupied(entry) => {
                let (_, stored_alias) = entry.key_value();
                bail!(
                    "interval alias is already stored; interval: {interval:?}, old alias: {stored_alias:?}, new alias: {alias:?}"
                );
            }
            Entry::Vacant(kinds_entry) => match self.aliases.entry_ref(alias.as_str()) {
                Entry::Occupied(entry) => {
                    let (_, stored_interval) = entry.key_value();
                    bail!(
                        "interval alias is already stored; alias: {alias:?}, old interval: {stored_interval:?}, new interval: {interval:?}"
                    );
                }
                Entry::Vacant(aliases_entry) => {
                    aliases_entry.insert(interval);
                    kinds_entry.insert(alias);
                }
            },
        };
        Ok(())
    }

    pub fn with<I: IntoIterator<Item = (u8, S)>, S: Into<String>>(
        mut self,
        kind: IntervalKind,
        iter: I,
    ) -> anyhow::Result<Self> {
        for (value, alias) in iter {
            let alias = alias.into();
            self.add(Interval { kind, value }, alias)?;
        }
        Ok(self)
    }

    pub fn to_alias(&self, interval: Interval) -> Option<&str> {
        self.kinds.get(&interval).map(AsRef::as_ref)
    }

    pub fn to_interval(&self, alias: &str) -> Option<Interval> {
        self.aliases.get(alias).cloned()
    }

    pub fn iter_intervals(&self) -> impl '_ + Iterator<Item = Interval> {
        self.kinds.keys().copied()
    }
}
