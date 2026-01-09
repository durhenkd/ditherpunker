#![allow(unused)]

pub(crate) mod bench_utils;
use bench_utils::*;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use ditherpunker::utils::iterator::{GridIterator, ParGridIterator as ParGridIteratorMapInit};
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct ParGridIterator {
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    pixel_idx: usize,
    remaining: usize,
}

impl ParGridIterator {
    #[inline]
    pub const fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            x: 0,
            y: 0,
            pixel_idx: 0,
            remaining: width * height,
        }
    }

    /// Create an iterator starting at a specific pixel index
    #[inline]
    const fn from_index(width: usize, height: usize, start_idx: usize, len: usize) -> Self {
        let y = start_idx / width;
        let x = start_idx % width;
        Self {
            width,
            height,
            x,
            y,
            pixel_idx: start_idx,
            remaining: len,
        }
    }
}

impl rayon::iter::ParallelIterator for ParGridIterator {
    type Item = (usize, usize, usize);

    #[inline]
    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
    {
        rayon::iter::plumbing::bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

impl rayon::iter::IndexedParallelIterator for ParGridIterator {
    #[inline]
    fn len(&self) -> usize {
        self.remaining
    }

    #[inline]
    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::Consumer<Self::Item>,
    {
        rayon::iter::plumbing::bridge(self, consumer)
    }

    #[inline]
    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: rayon::iter::plumbing::ProducerCallback<Self::Item>,
    {
        callback.callback(ParGridProducer {
            width: self.width,
            height: self.height,
            x: self.x,
            y: self.y,
            pixel_idx: self.pixel_idx,
            remaining: self.remaining,
        })
    }
}

/// Producer for ParGridIterator that handles the actual splitting and iteration
#[derive(Debug, Clone)]
struct ParGridProducer {
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    pixel_idx: usize,
    remaining: usize,
}

impl rayon::iter::plumbing::Producer for ParGridProducer {
    type Item = (usize, usize, usize);
    type IntoIter = GridIteratorProducer;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        GridIteratorProducer {
            width: self.width,
            x: self.x,
            y: self.y,
            pixel_idx: self.pixel_idx,
            remaining: self.remaining,
        }
    }

    #[inline]
    fn split_at(self, index: usize) -> (Self, Self) {
        let left_remaining = index;
        let right_remaining = self.remaining - index;

        let right_pixel_idx = self.pixel_idx + index;
        let right_y = right_pixel_idx / self.width;
        let right_x = right_pixel_idx % self.width;

        let left = ParGridProducer {
            width: self.width,
            height: self.height,
            x: self.x,
            y: self.y,
            pixel_idx: self.pixel_idx,
            remaining: left_remaining,
        };

        let right = ParGridProducer {
            width: self.width,
            height: self.height,
            x: right_x,
            y: right_y,
            pixel_idx: right_pixel_idx,
            remaining: right_remaining,
        };

        (left, right)
    }
}

/// The actual iterator used by the producer
#[derive(Debug, Clone)]
struct GridIteratorProducer {
    width: usize,
    x: usize,
    y: usize,
    pixel_idx: usize,
    remaining: usize,
}

impl Iterator for GridIteratorProducer {
    type Item = (usize, usize, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let result = (self.x, self.y, self.pixel_idx);

        self.pixel_idx += 1;
        self.x += 1;
        self.remaining -= 1;

        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
        }

        Some(result)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for GridIteratorProducer {
    #[inline]
    fn len(&self) -> usize {
        self.remaining
    }
}

impl DoubleEndedIterator for GridIteratorProducer {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        self.remaining -= 1;
        let last_idx = self.pixel_idx + self.remaining;
        let last_y = last_idx / self.width;
        let last_x = last_idx % self.width;

        Some((last_x, last_y, last_idx))
    }
}

/// Simple parallel iterator using range.map() approach
/// This is simpler but uses division/modulo per pixel
#[derive(Debug, Clone, Copy)]
pub struct ParGridIteratorSimple {
    width: usize,
    height: usize,
}

impl ParGridIteratorSimple {
    #[inline]
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    /// Convert into a parallel iterator using map
    #[inline]
    pub fn iter(self) -> impl ParallelIterator<Item = (usize, usize, usize)> {
        let width = self.width;
        (0..self.width * self.height)
            .into_par_iter()
            .map(move |idx| {
                let y = idx / width;
                let x = idx % width;
                (x, y, idx)
            })
    }
}

/// Parallel iterator using Cartesian product approach (zip of ranges)
/// This is the simplest approach - no division, modulo, or state tracking needed!
/// Just compute index as y * width + x
#[derive(Debug, Clone, Copy)]
pub struct ParGridIteratorZip {
    width: usize,
    height: usize,
}

impl ParGridIteratorZip {
    #[inline]
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    /// Convert into a parallel iterator using Cartesian product
    /// This iterates y first (outer), then x (inner), computing idx = y * width + x
    #[inline]
    pub fn iter(self) -> impl ParallelIterator<Item = (usize, usize, usize)> {
        let width = self.width;
        (0..self.height).into_par_iter().flat_map(move |y| {
            (0..width).into_par_iter().map(move |x| {
                let idx = y * width + x;
                (x, y, idx)
            })
        })
    }
}

fn do_iterator<It: Iterator<Item = (usize, usize, usize)>>(it: It) {
    let mut sum = std::hint::black_box(0_i32);
    for (x, y, idx) in it {
        sum += std::hint::black_box(x) as i32
            + std::hint::black_box(y) as i32
            + std::hint::black_box(idx) as i32;
    }
}

fn do_iteration(width: usize, height: usize) {
    let mut idx = 0_usize;
    let mut sum = std::hint::black_box(0_i32);
    for y in 0..height {
        for x in 0..width {
            sum += std::hint::black_box(x) as i32
                + std::hint::black_box(y) as i32
                + std::hint::black_box(idx) as i32;
            idx += 1;
        }
    }
}

fn do_par_iterator(par_it: ParGridIterator) {
    let _sum: i32 = par_it
        .map(|(x, y, idx)| {
            std::hint::black_box(x) as i32
                + std::hint::black_box(y) as i32
                + std::hint::black_box(idx) as i32
        })
        .sum();
}

fn do_par_iterator_simple(par_it: ParGridIteratorSimple) {
    let _sum: i32 = par_it
        .iter()
        .map(|(x, y, idx)| {
            std::hint::black_box(x) as i32
                + std::hint::black_box(y) as i32
                + std::hint::black_box(idx) as i32
        })
        .sum();
}

fn do_par_iterator_map_init(par_it: ParGridIteratorMapInit) {
    let _sum: i32 = par_it
        .par_iter()
        .map(|(x, y, idx)| {
            std::hint::black_box(x) as i32
                + std::hint::black_box(y) as i32
                + std::hint::black_box(idx) as i32
        })
        .sum();
}

fn do_par_iterator_zip(par_it: ParGridIteratorZip) {
    let _sum: i32 = par_it
        .iter()
        .map(|(x, y, idx)| {
            std::hint::black_box(x) as i32
                + std::hint::black_box(y) as i32
                + std::hint::black_box(idx) as i32
        })
        .sum();
}

const IT_SIZES: [usize; 5] = [10, 100, 300, 500, 1080];

fn bench_grid_iterator(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
) {
    for size in IT_SIZES {
        group.bench_with_input(
            BenchmarkId::new("iterator_grid", size),
            &size,
            |b, &size| {
                b.iter(|| do_iterator(GridIterator::new(size, size)));
            },
        );
    }
}

fn bench_iteration(group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    for size in IT_SIZES {
        group.bench_with_input(
            BenchmarkId::new("iterator_plain", size),
            &size,
            |b, &size| {
                b.iter(|| do_iteration(size, size));
            },
        );
    }
}

fn bench_par_iterator_direct(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
) {
    for size in IT_SIZES {
        group.bench_with_input(BenchmarkId::new("par_direct", size), &size, |b, &size| {
            b.iter(|| do_par_iterator(ParGridIterator::new(size, size)));
        });
    }
}

fn bench_par_iterator_simple(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
) {
    for size in IT_SIZES {
        group.bench_with_input(
            BenchmarkId::new("par_range_map", size),
            &size,
            |b, &size| {
                b.iter(|| do_par_iterator_simple(ParGridIteratorSimple::new(size, size)));
            },
        );
    }
}

fn bench_par_iterator_map_init(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
) {
    for size in IT_SIZES {
        group.bench_with_input(BenchmarkId::new("par_map_init", size), &size, |b, &size| {
            b.iter(|| do_par_iterator_map_init(ParGridIteratorMapInit::new(size, size)));
        });
    }
}

fn bench_par_iterator_zip(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
) {
    for size in IT_SIZES {
        group.bench_with_input(BenchmarkId::new("par_zip", size), &size, |b, &size| {
            b.iter(|| do_par_iterator_zip(ParGridIteratorZip::new(size, size)));
        });
    }
}

fn sequential_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("sequential");
    bench_grid_iterator(&mut group);
    bench_iteration(&mut group);
    group.finish();
}

fn parallel_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel");
    bench_par_iterator_direct(&mut group);
    bench_par_iterator_simple(&mut group);
    bench_par_iterator_map_init(&mut group);
    bench_par_iterator_zip(&mut group);
    group.finish();
}

criterion_group!(benches, sequential_benchmark, parallel_benchmark);
criterion_main!(benches);
