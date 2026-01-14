use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};

/// A grid iterator that yields (x, y, pixel_idx) tuples.
///
/// This iterator is designed to be completely inlined and optimized away
/// by the compiler, matching the performance of hand-written nested loops.
///
/// This is still ~25% slower than a simple squence of for loops,
/// but has the advantage of being presented as an iterator.
#[derive(Debug, Clone, Copy)]
pub struct GridIterator {
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    pixel_idx: usize,
}

impl GridIterator {
    #[inline]
    pub const fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            x: 0,
            y: 0,
            pixel_idx: 0,
        }
    }
}

impl Iterator for GridIterator {
    type Item = (usize, usize, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.y >= self.height {
            return None;
        }

        let result = (self.x, self.y, self.pixel_idx);

        self.pixel_idx += 1;
        self.x += 1;

        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
        }

        Some(result)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.width * self.height - self.pixel_idx;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for GridIterator {
    #[inline]
    fn len(&self) -> usize {
        self.width * self.height - self.pixel_idx
    }
}

impl From<GridIterator> for ParGridIterator {
    fn from(grid: GridIterator) -> Self {
        ParGridIterator::new(grid.width, grid.height)
    }
}

/// A parallel grid iterator that yields (x, y, pixel_idx) tuples.
///
/// This approach uses map_init to maintain x/y state per thread and avoid % or /
/// as much as possible.
///
/// ## Notes
///
/// Faster than the intuitive:
///
/// ```ignore
/// (0..self.width * self.height)
///         .into_par_iter()
///         .map(move |idx| {
///             let y = idx / width;
///             let x = idx % width;
///             (x, y, idx)
///         })
/// ```
///
/// Significantly faster than performing cartesian products / zip-like combinators:
///
/// ```ignore
/// (0..self.height).into_par_iter().flat_map(move |y| {
///        (0..width).into_par_iter().map(move |x| {
///            let idx = y * width + x;
///            (x, y, idx)
///        })
///    })
/// ```
///
/// And barely faster than the manual implementation of ParallelIterator, IndexedParallelIterator
/// rayon::iter::plumbing::Producer visible in the iterator benchmark file.
#[derive(Debug, Clone, Copy)]
pub struct ParGridIterator {
    width: usize,
    height: usize,
}

impl ParGridIterator {
    #[inline]
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    /// Convert into a parallel iterator using map_init
    /// Each thread initializes its own (x, y) state based on the first index it receives
    #[inline]
    pub fn par_iter(self) -> impl IndexedParallelIterator<Item = (usize, usize, usize)> {
        let width = self.width;
        (0..self.width * self.height).into_par_iter().map_init(
            || None,
            move |state: &mut Option<(usize, usize, usize)>, idx| {
                match state {
                    None => {
                        // First iteration for this thread: compute x, y from idx
                        let y = idx / width;
                        let x = idx % width;
                        *state = Some((x, y, idx));
                        (x, y, idx)
                    }
                    Some((x, y, last_idx)) => {
                        // Subsequent iterations: increment from previous state
                        let delta = idx - *last_idx;

                        // Fast path: consecutive indices
                        if delta == 1 {
                            let new_x = *x + 1;
                            if new_x >= width {
                                // Wrap to next row
                                *x = 0;
                                *y += 1;
                            } else {
                                *x = new_x;
                            }
                        } else {
                            // Slow path: non-consecutive (happens at chunk boundaries)
                            *y = idx / width;
                            *x = idx % width;
                        }

                        *last_idx = idx;
                        (*x, *y, idx)
                    }
                }
            },
        )
    }
}
