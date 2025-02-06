pub mod sorted_vec;
pub mod url;

// TODO: make derive macro
pub trait Has<L: ValueLabel> {
    fn give(&self, label: L) -> &L::Value;
}

pub trait ValueLabel {
    type Value;
}
