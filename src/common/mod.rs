//! Common helpers for `displayz`

pub type Resolution = (usize, usize);
pub type Position = (usize, usize);

pub type Displays = Vec<dyn DisplayOutput>;

pub trait DisplayOutput {
    fn is_primary(&self) -> bool;
    fn is_active(&self) -> bool;
    fn get_position(&self) -> Position;
    fn get_resolution(&self) -> Resolution;
}
