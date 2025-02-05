pub mod sorted_vec;
pub mod url;

pub trait Has<T> {
    fn give(&self) -> &T;
}
