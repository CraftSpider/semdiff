use std::fmt::Debug;
use crate::Diffable;
use crate::algo::DiffAlgo;

impl<T: PartialEq + Debug> Diffable for [T] {
    type Item = [T];

    fn diff<'a, A: DiffAlgo<Self::Item>>(&'a self, other: &'a Self) -> A::Diff<'a> {
        A::diff(self, other)
    }
}

impl Diffable for str {
    type Item = str;

    fn diff<'a, A: DiffAlgo<Self::Item>>(&'a self, other: &'a Self) -> A::Diff<'a> {
        A::diff(self, other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{algo, Diffable, DiffRes};

    #[test]
    fn test_slice() {
        let a = [1, 2, 3, 4, 5, 6, 7, 8];
        let b = [1, 3, 4, 5, 2, 6, 7];
        let d = a.diff::<algo::Default>(&b);

        assert_eq!(
            d,
            vec![
                DiffRes::Both(&[1] as &[_], &[1]),
                DiffRes::Left(&[2]),
                DiffRes::Both(&[3, 4, 5], &[3, 4, 5]),
                DiffRes::Right(&[2]),
                DiffRes::Both(&[6, 7], &[6, 7]),
                DiffRes::Left(&[8]),
            ],
        );

        let a = [1, 2, 3, 4, 5];
        let b = [0, 1, 2, 3, 4, 5, 6, 7, 8];
        let d = a.diff::<algo::Default>(&b);
        assert_eq!(
            d,
            vec![
                DiffRes::Right(&[0] as &[_]),
                DiffRes::Both(&[1, 2, 3, 4, 5], &[1, 2, 3, 4, 5]),
                DiffRes::Right(&[6, 7, 8]),
            ],
        );
    }
}
