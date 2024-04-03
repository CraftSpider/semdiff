use crate::algo::{DiffAlgo, DiffPatch};
use crate::{algo, Diffable};
use image::{ImageBuffer, Luma, LumaA, Pixel, Primitive, Rgb, Rgba};
use num_traits::identities::Zero;
use num_traits::{Bounded, One};
use std::cmp::Ordering;
use std::ops::Deref;

fn max_subpix<P: Pixel>(_: P::Subpixel) -> P::Subpixel {
    P::Subpixel::DEFAULT_MAX_VALUE
}

pub trait PixelExt: Pixel {
    fn is_alpha() -> bool {
        Self::COLOR_MODEL.contains('A')
    }

    fn is_rgb() -> bool {
        Self::COLOR_MODEL.contains("RGB")
    }

    fn channels_no_alpha(&self) -> &[Self::Subpixel] {
        let chans = self.channels();
        if Self::is_alpha() {
            &chans[..chans.len() - 1]
        } else {
            chans
        }
    }

    fn channels_no_alpha_mut(&mut self) -> &mut [Self::Subpixel] {
        if Self::is_alpha() {
            let chans = self.channels_mut();
            let len = chans.len();
            &mut chans[..len - 1]
        } else {
            self.channels_mut()
        }
    }

    fn alpha(&self) -> Option<&Self::Subpixel> {
        if Self::is_alpha() {
            self.channels().last()
        } else {
            None
        }
    }

    fn alpha_mut(&mut self) -> Option<&mut Self::Subpixel> {
        if Self::is_alpha() {
            self.channels_mut().last_mut()
        } else {
            None
        }
    }

    fn map2_with_alpha(
        &self,
        other: &Self,
        mut f: impl FnMut(Self::Subpixel, Self::Subpixel) -> Self::Subpixel,
        mut g: impl FnMut(Self::Subpixel, Self::Subpixel) -> Self::Subpixel,
    ) -> Self {
        let mut new = *self;
        let mut idx = 0;
        let cl = self.channels();
        let cr = other.channels();
        new.apply_with_alpha(
            |_| {
                let out = f(cl[idx], cr[idx]);
                idx += 1;
                out
            },
            |_| g(*self.alpha().unwrap(), *other.alpha().unwrap()),
        );
        new
    }

    fn red(&self) -> Self {
        if Self::is_rgb() {
            let mut idx = 0;
            self.map_with_alpha(
                |_| {
                    let out = if idx == 0 {
                        Self::Subpixel::DEFAULT_MAX_VALUE
                    } else {
                        Self::Subpixel::zero()
                    };
                    idx += 1;
                    out
                },
                max_subpix::<Self>,
            )
        } else {
            self.map_with_alpha(|_| Self::Subpixel::zero(), max_subpix::<Self>)
        }
    }

    fn green(&self) -> Self {
        if Self::is_rgb() {
            let mut idx = 0;
            self.map_with_alpha(
                |_| {
                    let out = if idx == 1 {
                        Self::Subpixel::DEFAULT_MAX_VALUE
                    } else {
                        Self::Subpixel::zero()
                    };
                    idx += 1;
                    out
                },
                max_subpix::<Self>,
            )
        } else {
            self.map_with_alpha(max_subpix::<Self>, max_subpix::<Self>)
        }
    }

    fn blue(&self) -> Self {
        if Self::is_rgb() {
            let mut idx = 0;
            self.map_with_alpha(
                |_| {
                    let out = if idx == 2 {
                        Self::Subpixel::DEFAULT_MAX_VALUE
                    } else {
                        Self::Subpixel::zero()
                    };
                    idx += 1;
                    out
                },
                max_subpix::<Self>,
            )
        } else {
            self.map_with_alpha(
                |_| {
                    Self::Subpixel::DEFAULT_MAX_VALUE
                        / (Self::Subpixel::one() + Self::Subpixel::one())
                },
                max_subpix::<Self>,
            )
        }
    }
}

impl<P: Pixel> PixelExt for P {}

pub struct ColorSub;

impl<P, C> DiffAlgo<ImageBuffer<P, C>> for ColorSub
where
    P: Pixel,
    C: Deref<Target = [P::Subpixel]>,
{
    type Diff<'a> = ImageBuffer<P, Vec<P::Subpixel>>
    where
        ImageBuffer<P, C>: 'a;

    fn diff<'a>(l: &'a ImageBuffer<P, C>, r: &'a ImageBuffer<P, C>) -> Self::Diff<'a> {
        let width = u32::max(l.width(), r.width());
        let height = u32::max(l.height(), r.height());

        let z = vec![P::Subpixel::zero(); P::CHANNEL_COUNT as usize];
        let zeroed = P::from_slice(&z);

        let mut out = ImageBuffer::<P, _>::new(width, height);
        for h in 0..height {
            for w in 0..width {
                let pl = l.get_pixel_checked(w, h).unwrap_or(zeroed);
                let pr = r.get_pixel_checked(w, h).unwrap_or(zeroed);
                out[(w, h)] = pl.map2_with_alpha(
                    pr,
                    |l, r| if l < r { r - l } else { l - r },
                    |l, r| if l > r { l } else { r },
                );
            }
        }
        out
    }
}

pub struct Heatmap;

impl<P, C> DiffAlgo<ImageBuffer<P, C>> for Heatmap
where
    P: Pixel,
    C: Deref<Target = [P::Subpixel]>,
{
    type Diff<'a> = ImageBuffer<P, Vec<P::Subpixel>>
    where
        ImageBuffer<P, C>: 'a;

    fn diff<'a>(l: &'a ImageBuffer<P, C>, r: &'a ImageBuffer<P, C>) -> Self::Diff<'a> {
        let width = u32::max(l.width(), r.width());
        let height = u32::max(l.height(), r.height());

        let z = vec![P::Subpixel::zero(); P::CHANNEL_COUNT as usize];
        let zeroed = P::from_slice(&z);

        let mut out = ImageBuffer::<P, _>::new(width, height);

        for h in 0..height {
            for w in 0..width {
                let pl = l.get_pixel_checked(w, h).unwrap_or(zeroed);
                let pr = r.get_pixel_checked(w, h).unwrap_or(zeroed);
                if pl.channels_no_alpha() == pr.channels_no_alpha()
                    || (pl.alpha() == Some(&P::Subpixel::zero())
                        && pr.alpha() == Some(&P::Subpixel::zero()))
                {
                    out[(w, h)] = *pl;
                } else {
                    // TODO: Try to judge 'how different' the pixels are
                    let mut idx = 0;
                    out[(w, h)] = pl.map2_with_alpha(
                        pr,
                        |_, _| {
                            if idx == 0 {
                                idx += 1;
                                P::Subpixel::max_value()
                            } else {
                                P::Subpixel::zero()
                            }
                        },
                        |l, r| if l > r { l } else { r },
                    )
                }
            }
        }

        out
    }
}

pub struct RedGreen;

impl<P, C> DiffAlgo<ImageBuffer<P, C>> for RedGreen
where
    P: Pixel,
    C: Deref<Target = [P::Subpixel]>,
{
    type Diff<'a> = ImageBuffer<P, Vec<P::Subpixel>>
    where
        ImageBuffer<P, C>: 'a;

    fn diff<'a>(l: &'a ImageBuffer<P, C>, r: &'a ImageBuffer<P, C>) -> Self::Diff<'a> {
        let width = u32::max(l.width(), r.width());
        let height = u32::max(l.height(), r.height());

        let z = vec![P::Subpixel::zero(); P::CHANNEL_COUNT as usize];
        let zeroed = P::from_slice(&z);

        let mut out = ImageBuffer::<P, _>::new(width, height);

        for h in 0..height {
            for w in 0..width {
                let pl = l.get_pixel_checked(w, h).unwrap_or(zeroed);
                let pr = r.get_pixel_checked(w, h).unwrap_or(zeroed);
                // TODO: Try to judge 'how different' the pixels are

                let la = pl.alpha().copied().unwrap_or(Zero::zero());
                let ra = pr.alpha().copied().unwrap_or(Zero::zero());
                match la.partial_cmp(&ra).unwrap() {
                    Ordering::Equal if la == Zero::zero() => {
                        out[(w, h)] = *pl;
                        continue;
                    }
                    Ordering::Equal => (),
                    Ordering::Less => {
                        out[(w, h)] = pl.green();
                        continue;
                    }
                    Ordering::Greater => {
                        out[(w, h)] = pl.red();
                        continue;
                    }
                }

                if pl.channels_no_alpha() == pr.channels_no_alpha() {
                    out[(w, h)] = *pl;
                } else {
                    out[(w, h)] = pl.blue();
                }
            }
        }

        out
    }
}

mod sealed {
    use super::*;

    pub trait DiffPixel: Pixel {
        type Diff;
    }

    macro_rules! diff_pixels {
        (@ $pixel:ident, $ty:ty, $bigger:ty) => {
            impl DiffPixel for $pixel<$ty> {
                type Diff = [$bigger; <$pixel<$ty> as Pixel>::CHANNEL_COUNT as usize];
            }
        };
        ($($ty:ty => $bigger:ty),* $(,)?) => {
            $(
            diff_pixels!(@ Rgb, $ty, $bigger);
            diff_pixels!(@ Rgba, $ty, $bigger);
            diff_pixels!(@ Luma, $ty, $bigger);
            diff_pixels!(@ LumaA, $ty, $bigger);
            )*
        };
    }

    diff_pixels!(
        u8 => i16,
        i8 => i16,
        u16 => i32,
        i16 => i32,
        u32 => i64,
        i32 => i64,
        u64 => i128,
        i64 => i128,
        f32 => f32,
        f64 => f64,
    );
}

use sealed::DiffPixel;

pub struct PixelPatch;

impl<P, C> DiffAlgo<ImageBuffer<P, C>> for PixelPatch
where
    P: Pixel + DiffPixel,
    C: Deref<Target = [P::Subpixel]>,
{
    type Diff<'a> = Vec<Vec<P::Diff>>
    where
        ImageBuffer<P, C>: 'a;

    fn diff<'a>(l: &'a ImageBuffer<P, C>, r: &'a ImageBuffer<P, C>) -> Self::Diff<'a> {
        todo!()
    }
}

impl<P, C> DiffPatch<ImageBuffer<P, C>> for PixelPatch
where
    P: Pixel + DiffPixel,
    C: Deref<Target = [P::Subpixel]>,
{
}

impl<P, C> DiffAlgo<ImageBuffer<P, C>> for algo::Default
where
    P: Pixel,
    C: Deref<Target = [P::Subpixel]>,
{
    type Diff<'a> = ImageBuffer<P, Vec<P::Subpixel>>
    where
        ImageBuffer<P, C>: 'a;

    fn diff<'a>(l: &'a ImageBuffer<P, C>, r: &'a ImageBuffer<P, C>) -> Self::Diff<'a> {
        RedGreen::diff(l, r)
    }
}

impl<P: Pixel, C> Diffable for ImageBuffer<P, C> {
    type Diff<'a, A: DiffAlgo<Self::Item>> = A::Diff<'a>
    where
        Self: 'a;
    type Item = ImageBuffer<P, C>;

    fn diff<'a, A: DiffAlgo<Self::Item>>(&'a self, other: &'a Self) -> A::Diff<'a> {
        A::diff(self, other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::io::Reader;
    use image::ImageFormat;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::{Path, PathBuf};

    fn test_path(segments: &[&str]) -> PathBuf {
        let mut path = Path::new(env!("CARGO_MANIFEST_DIR")).to_owned();
        path.push("tests");
        for seg in segments {
            path.push(*seg)
        }
        path
    }

    fn read_rgba<P: AsRef<Path>>(path: P) -> image::RgbaImage {
        let file = BufReader::new(File::open(path).unwrap());
        Reader::with_format(file, ImageFormat::Png)
            .decode()
            .unwrap()
            .into_rgba8()
    }

    #[test]
    fn test_color_sub() {
        let img1 = test_path(&["assets", "img1.png"]);
        let img2 = test_path(&["assets", "img2.png"]);
        let out = test_path(&["assets", "color_sub.png"]);

        let img1 = read_rgba(img1);
        let img2 = read_rgba(img2);
        let out = read_rgba(out);

        let diff = img1.diff::<ColorSub>(&img2);
        assert_eq!(diff, out);
    }

    #[test]
    fn test_heatmap() {
        let img1 = test_path(&["assets", "img1.png"]);
        let img2 = test_path(&["assets", "img2.png"]);
        let out = test_path(&["assets", "heatmap.png"]);

        let img1 = read_rgba(img1);
        let img2 = read_rgba(img2);
        let out = read_rgba(out);

        let diff = img1.diff::<Heatmap>(&img2);
        assert_eq!(diff, out);
    }

    #[test]
    fn test_redgreen() {
        let img1 = test_path(&["assets", "img1.png"]);
        let img2 = test_path(&["assets", "img2.png"]);
        let out = test_path(&["assets", "red_green.png"]);

        let img1 = read_rgba(img1);
        let img2 = read_rgba(img2);
        let out = read_rgba(out);

        let diff = img1.diff::<RedGreen>(&img2);
        assert_eq!(diff, out);
    }
}
