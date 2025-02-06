pub mod sorted_vec;
pub mod time;
pub mod url;

// TODO: make derive macro
pub trait Has<L: ValueLabel> {
    fn give(&self, label: L) -> &L::Value;
}

pub trait ValueLabel {
    type Value;
}

pub trait Strict: 'static + Send + Sync + Sized {}

impl<T: 'static + Send + Sync + Sized> Strict for T {}
