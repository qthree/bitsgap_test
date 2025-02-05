pub mod sorted_vec;
pub mod url;

pub trait Has<L: ValueLabel> {
    fn give(&self) -> &L::Value;
}

pub trait ValueLabel {
    type Value;
}
