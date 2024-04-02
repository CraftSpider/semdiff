use std::fmt;
use std::fmt::{Debug, Write};
use crate::DiffRes;

pub fn fmt_string<W: Write>(diff: Vec<DiffRes<&str>>, w: &mut W) -> fmt::Result {
    for d in &diff {
        match d {
            DiffRes::Left(l) => writeln!(w, "-{}", l),
            DiffRes::Both(l, _) => writeln!(w, " {}", l),
            DiffRes::Right(r) => writeln!(w, "+{}", r),
        }?;
    }
    Ok(())
}

pub fn fmt_slice<T: Debug, W: Write>(diff: Vec<DiffRes<&[T]>>, w: &mut W) -> fmt::Result {
    fn write_slice<T: Debug, W: Write>(w: &mut W, slice: &[T]) -> fmt::Result {
        slice.iter()
            .enumerate()
            .try_for_each(|(idx, i)| {
                if idx != 0 {
                    write!(w, " ")?;
                }
                write!(w, "{:?}", i)
            })
    }

    for d in &diff {
        let line = match d {
            DiffRes::Left(l) => {
                write!(w, "-")?;
                *l
            },
            DiffRes::Both(l, _) => {
                write!(w, " ")?;
                *l
            },
            DiffRes::Right(r) => {
                writeln!(w, "+")?;
                *r
            },
        };
        write_slice(w, line)?;
        writeln!(w)?;
    }
    Ok(())
}

pub fn fmt_bytes<W: Write>(diff: Vec<DiffRes<&[u8]>>, w: &mut W) -> fmt::Result {
    fn write_slice<W: Write>(w: &mut W, slice: &[u8]) -> fmt::Result {
        slice.iter()
            .enumerate()
            .try_for_each(|(idx, i)| {
                if idx != 0 {
                    write!(w, " ")?;
                }
                write!(w, "{:02X}", i)
            })
    }

    for d in &diff {
        let line = match d {
            DiffRes::Left(l) => {
                write!(w, "-")?;
                *l
            },
            DiffRes::Both(l, _) => {
                write!(w, " ")?;
                *l
            },
            DiffRes::Right(r) => {
                writeln!(w, "+")?;
                *r
            },
        };
        write_slice(w, line)?;
        writeln!(w)?;
    }
    Ok(())
}
