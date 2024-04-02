
use image::{ImageBuffer, Pixel};
use crate::algo::DiffAlgo;
use crate::Diffable;

pub struct ColorSub;

impl<P: Pixel, C> DiffAlgo<ImageBuffer<P, C>> for ColorSub {
    type Diff<'a> = ImageBuffer<P, C>
    where
        ImageBuffer<P, C>: 'a;

    fn diff<'a>(l: &'a ImageBuffer<P, C>, r: &'a ImageBuffer<P, C>) -> Self::Diff<'a> {
        todo!()
    }
}

pub struct Heatmap;

impl<P: Pixel, C> DiffAlgo<ImageBuffer<P, C>> for Heatmap {
    type Diff<'a> = ImageBuffer<P, C>
    where
        ImageBuffer<P, C>: 'a;

    fn diff<'a>(l: &'a ImageBuffer<P, C>, r: &'a ImageBuffer<P, C>) -> Self::Diff<'a> {
        todo!()
    }
}

impl<P: Pixel, C> Diffable for ImageBuffer<P, C> {
    type Item = ImageBuffer<P, C>;

    fn diff<'a, A: DiffAlgo<Self::Item>>(&'a self, other: &'a Self) -> A::Diff<'a> {
        A::diff(self, other)
    }
}
