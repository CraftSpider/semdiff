
pub mod builtin;
#[cfg(feature = "img")]
pub mod img;
pub mod algo;
pub mod fmt;

use algo::DiffAlgo;

#[derive(Debug, PartialEq)]
pub enum DiffRes<T> {
    Left(T),
    Both(T, T),
    Right(T),
}

pub trait Diffable {
    type Item: ?Sized;

    fn diff<'a, A: DiffAlgo<Self::Item>>(&'a self, other: &'a Self) -> A::Diff<'a>;
}

pub fn diff<'a, T: Diffable>(a: &'a T, b: &'a T) -> <algo::Default as DiffAlgo<T::Item>>::Diff<'a>
where
    algo::Default: DiffAlgo<T::Item>,
{
    a.diff::<algo::Default>(b)
}
