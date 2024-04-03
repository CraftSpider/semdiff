use crate::algo::{DiffAlgo, DiffPatch};
use crate::{algo, DiffRes, Diffable};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

/// Generate a diff based on the longest common subsequence (McIlroy-Hunt) algorithm.
pub struct LcsDiff;

impl<T: PartialEq> DiffAlgo<[T]> for LcsDiff {
    type Diff<'a> = Vec<DiffRes<&'a [T]>>
    where
        T: 'a;

    fn diff<'a>(l: &'a [T], r: &'a [T]) -> Self::Diff<'a> {
        #[derive(Debug, PartialEq, Copy, Clone)]
        enum LR {
            Left,
            Right,
            Both,
        }

        let mut last = LR::Both;
        let mut splits = Vec::new();
        let mut l_idx = 0;
        let mut r_idx = 0;
        for val in diff::slice(l, r) {
            let val = match val {
                diff::Result::Left(_) => LR::Left,
                diff::Result::Right(_) => LR::Right,
                diff::Result::Both(_, _) => LR::Both,
            };
            if val != last {
                splits.push((l_idx, r_idx, last, val));
                last = val;
            }
            match val {
                LR::Left => l_idx += 1,
                LR::Right => r_idx += 1,
                LR::Both => {
                    l_idx += 1;
                    r_idx += 1
                }
            }
        }

        let mut last_l_idx = 0;
        let mut last_r_idx = 0;
        let mut diff = Vec::new();
        let mut trailing = LR::Both;
        for (l_idx, r_idx, was, now) in splits {
            trailing = now;
            if l[last_l_idx..l_idx].is_empty() && r[last_r_idx..r_idx].is_empty() {
                last_l_idx = l_idx;
                last_r_idx = r_idx;
                continue;
            }
            let left = &l[last_l_idx..usize::min(l_idx, l.len())];
            let right = &r[last_r_idx..usize::min(r_idx, r.len())];
            let res = match was {
                LR::Left => DiffRes::Left(left),
                LR::Right => DiffRes::Right(right),
                LR::Both => DiffRes::Both(left, right),
            };
            diff.push(res);
            last_l_idx = l_idx;
            last_r_idx = r_idx;
        }
        if last_l_idx < l.len() || last_r_idx < r.len() {
            let left = &l[usize::min(last_l_idx, l.len())..];
            let right = &r[usize::min(last_r_idx, r.len())..];
            let res = match trailing {
                LR::Left => DiffRes::Left(left),
                LR::Right => DiffRes::Right(right),
                LR::Both => DiffRes::Both(left, right),
            };
            diff.push(res);
        }

        diff
    }
}

impl<T: PartialEq> DiffPatch<[T]> for LcsDiff {}

impl<T: PartialEq> DiffAlgo<[T]> for algo::Default {
    type Diff<'a> = Vec<DiffRes<&'a [T]>>
    where
        T: 'a;

    fn diff<'a>(l: &'a [T], r: &'a [T]) -> Self::Diff<'a> {
        LcsDiff::diff(l, r)
    }
}

impl DiffAlgo<str> for LcsDiff {
    type Diff<'a> = Vec<DiffRes<&'a str>>;

    fn diff<'a>(l: &'a str, r: &'a str) -> Self::Diff<'a> {
        diff::lines(l, r)
            .into_iter()
            .map(|d| match d {
                diff::Result::Left(l) => DiffRes::Left(l),
                diff::Result::Both(l, r) => DiffRes::Both(l, r),
                diff::Result::Right(r) => DiffRes::Right(r),
            })
            .collect()
    }
}

impl DiffPatch<str> for LcsDiff {}

impl DiffAlgo<str> for algo::Default {
    type Diff<'a> = Vec<DiffRes<&'a str>>;

    fn diff<'a>(l: &'a str, r: &'a str) -> Self::Diff<'a> {
        LcsDiff::diff(l, r)
    }
}

impl<T: Eq + Hash> DiffAlgo<HashSet<T>> for LcsDiff {
    type Diff<'a> = HashSet<DiffRes<&'a T>>
    where
        HashSet<T>: 'a;

    fn diff<'a>(l: &'a HashSet<T>, r: &'a HashSet<T>) -> Self::Diff<'a> {
        let mut out = HashSet::new();
        for item in l {
            out.insert(DiffRes::Left(item));
        }
        for item in r {
            if out.remove(&DiffRes::Left(item)) {
                out.insert(DiffRes::Both(item, item));
            } else {
                out.insert(DiffRes::Right(item));
            }
        }
        out
    }
}

impl<T: Eq + Hash> DiffPatch<HashSet<T>> for LcsDiff {}

impl<T: Eq + Hash> Diffable for HashSet<T> {
    type Diff<'a, A: DiffAlgo<Self::Item>> = A::Diff<'a>
    where
        Self: 'a;
    type Item = HashSet<T>;

    fn diff<'a, A: DiffAlgo<Self::Item>>(&'a self, other: &'a Self) -> Self::Diff<'a, A> {
        A::diff(self, other)
    }
}

impl<T: PartialEq + Debug> Diffable for [T] {
    type Diff<'a, A: DiffAlgo<Self::Item>> = A::Diff<'a>
    where
        Self: 'a;
    type Item = [T];
    fn diff<'a, A: DiffAlgo<Self::Item>>(&'a self, other: &'a Self) -> A::Diff<'a> {
        A::diff(self, other)
    }
}

impl Diffable for str {
    type Diff<'a, A: DiffAlgo<Self::Item>> = A::Diff<'a>;
    type Item = str;

    fn diff<'a, A: DiffAlgo<Self::Item>>(&'a self, other: &'a Self) -> A::Diff<'a> {
        A::diff(self, other)
    }
}

#[cfg(test)]
mod tests {
    use crate::{algo, DiffRes, Diffable};

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
