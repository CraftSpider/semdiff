//! A library for generating human-readable differences between strings, images, and even more.
//!
//!

pub mod algo;
pub mod builtin;
pub mod fmt;
#[cfg(feature = "img")]
pub mod img;

use algo::DiffAlgo;

#[derive(Debug, PartialEq)]
pub enum DiffRes<T> {
    Left(T),
    Both(T, T),
    Right(T),
}

/// A type that supports determining the difference between two instances. A difference should
/// generally be meaningful to a person, but may not allow re-creating the original values from
/// one-another. Difference algorithms that allow that are known as 'patch' differences.
pub trait Diffable {
    /// The output of diffing this type using a given algorithm. May borrow from either input.
    /// For types where [`Item`] == [`Self`], this will generally be [`A::Diff`].
    type Diff<'a, A: DiffAlgo<Self::Item>>
    where
        Self: 'a;

    /// The value that this type actually compares to determine difference, this decides which
    /// algorithms are applicable to it.
    type Item: ?Sized;

    /// Generate the difference of this to another value, using the algorithm [`A`].
    fn diff<'a, A: DiffAlgo<Self::Item>>(&'a self, other: &'a Self) -> Self::Diff<'a, A>;
}

/// Generate the difference between two types, using the default difference algorithm.
pub fn diff<'a, T: Diffable>(a: &'a T, b: &'a T) -> T::Diff<'a, algo::Default>
where
    algo::Default: DiffAlgo<T::Item>,
{
    a.diff::<algo::Default>(b)
}
