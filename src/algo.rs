use crate::DiffRes;

pub trait DiffAlgo<T: ?Sized> {
    type Diff<'a>
    where
        T: 'a;

    fn diff<'a>(l: &'a T, r: &'a T) -> Self::Diff<'a>;
}

pub struct Default;

impl<T: PartialEq> DiffAlgo<[T]> for Default {
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
                LR::Both => { l_idx += 1; r_idx += 1 },
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

impl DiffAlgo<str> for Default {
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
