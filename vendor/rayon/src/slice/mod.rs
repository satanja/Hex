//! Parallel iterator types for [slices][std::slice]
//!
//! You will rarely need to interact with this module directly unless you need
//! to name one of the iterator types.
//!
//! [std::slice]: https://doc.rust-lang.org/stable/std/slice/

mod chunks;
mod mergesort;
mod quicksort;
mod rchunks;

mod test;

use self::mergesort::par_mergesort;
use self::quicksort::par_quicksort;
use crate::iter::plumbing::*;
use crate::iter::*;
use crate::split_producer::*;
use std::cmp;
use std::cmp::Ordering;
use std::fmt::{self, Debug};

pub use self::chunks::{Chunks, ChunksExact, ChunksExactMut, ChunksMut};
pub use self::rchunks::{RChunks, RChunksExact, RChunksExactMut, RChunksMut};

/// Parallel extensions for slices.
pub trait ParallelSlice<T: Sync> {
    /// Returns a plain slice, which is used to implement the rest of the
    /// parallel methods.
    fn as_parallel_slice(&self) -> &[T];

    /// Returns a parallel iterator over subslices separated by elements that
    /// match the separator.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    /// let smallest = [1, 2, 3, 0, 2, 4, 8, 0, 3, 6, 9]
    ///     .par_split(|i| *i == 0)
    ///     .map(|numbers| numbers.iter().min().unwrap())
    ///     .min();
    /// assert_eq!(Some(&1), smallest);
    /// ```
    fn par_split<P>(&self, separator: P) -> Split<'_, T, P>
    where
        P: Fn(&T) -> bool + Sync + Send,
    {
        Split {
            slice: self.as_parallel_slice(),
            separator,
        }
    }

    /// Returns a parallel iterator over all contiguous windows of length
    /// `window_size`. The windows overlap.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    /// let windows: Vec<_> = [1, 2, 3].par_windows(2).collect();
    /// assert_eq!(vec![[1, 2], [2, 3]], windows);
    /// ```
    fn par_windows(&self, window_size: usize) -> Windows<'_, T> {
        Windows {
            window_size,
            slice: self.as_parallel_slice(),
        }
    }

    /// Returns a parallel iterator over at most `chunk_size` elements of
    /// `self` at a time. The chunks do not overlap.
    ///
    /// If the number of elements in the iterator is not divisible by
    /// `chunk_size`, the last chunk may be shorter than `chunk_size`.  All
    /// other chunks will have that exact length.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    /// let chunks: Vec<_> = [1, 2, 3, 4, 5].par_chunks(2).collect();
    /// assert_eq!(chunks, vec![&[1, 2][..], &[3, 4], &[5]]);
    /// ```
    fn par_chunks(&self, chunk_size: usize) -> Chunks<'_, T> {
        assert!(chunk_size != 0, "chunk_size must not be zero");
        Chunks::new(chunk_size, self.as_parallel_slice())
    }

    /// Returns a parallel iterator over `chunk_size` elements of
    /// `self` at a time. The chunks do not overlap.
    ///
    /// If `chunk_size` does not divide the length of the slice, then the
    /// last up to `chunk_size-1` elements will be omitted and can be
    /// retrieved from the remainder function of the iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    /// let chunks: Vec<_> = [1, 2, 3, 4, 5].par_chunks_exact(2).collect();
    /// assert_eq!(chunks, vec![&[1, 2][..], &[3, 4]]);
    /// ```
    fn par_chunks_exact(&self, chunk_size: usize) -> ChunksExact<'_, T> {
        assert!(chunk_size != 0, "chunk_size must not be zero");
        ChunksExact::new(chunk_size, self.as_parallel_slice())
    }

    /// Returns a parallel iterator over at most `chunk_size` elements of `self` at a time,
    /// starting at the end. The chunks do not overlap.
    ///
    /// If the number of elements in the iterator is not divisible by
    /// `chunk_size`, the last chunk may be shorter than `chunk_size`.  All
    /// other chunks will have that exact length.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    /// let chunks: Vec<_> = [1, 2, 3, 4, 5].par_rchunks(2).collect();
    /// assert_eq!(chunks, vec![&[4, 5][..], &[2, 3], &[1]]);
    /// ```
    fn par_rchunks(&self, chunk_size: usize) -> RChunks<'_, T> {
        assert!(chunk_size != 0, "chunk_size must not be zero");
        RChunks::new(chunk_size, self.as_parallel_slice())
    }

    /// Returns a parallel iterator over `chunk_size` elements of `self` at a time,
    /// starting at the end. The chunks do not overlap.
    ///
    /// If `chunk_size` does not divide the length of the slice, then the
    /// last up to `chunk_size-1` elements will be omitted and can be
    /// retrieved from the remainder function of the iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    /// let chunks: Vec<_> = [1, 2, 3, 4, 5].par_rchunks_exact(2).collect();
    /// assert_eq!(chunks, vec![&[4, 5][..], &[2, 3]]);
    /// ```
    fn par_rchunks_exact(&self, chunk_size: usize) -> RChunksExact<'_, T> {
        assert!(chunk_size != 0, "chunk_size must not be zero");
        RChunksExact::new(chunk_size, self.as_parallel_slice())
    }
}

impl<T: Sync> ParallelSlice<T> for [T] {
    #[inline]
    fn as_parallel_slice(&self) -> &[T] {
        self
    }
}

/// Parallel extensions for mutable slices.
pub trait ParallelSliceMut<T: Send> {
    /// Returns a plain mutable slice, which is used to implement the rest of
    /// the parallel methods.
    fn as_parallel_slice_mut(&mut self) -> &mut [T];

    /// Returns a parallel iterator over mutable subslices separated by
    /// elements that match the separator.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    /// let mut array = [1, 2, 3, 0, 2, 4, 8, 0, 3, 6, 9];
    /// array.par_split_mut(|i| *i == 0)
    ///      .for_each(|slice| slice.reverse());
    /// assert_eq!(array, [3, 2, 1, 0, 8, 4, 2, 0, 9, 6, 3]);
    /// ```
    fn par_split_mut<P>(&mut self, separator: P) -> SplitMut<'_, T, P>
    where
        P: Fn(&T) -> bool + Sync + Send,
    {
        SplitMut {
            slice: self.as_parallel_slice_mut(),
            separator,
        }
    }

    /// Returns a parallel iterator over at most `chunk_size` elements of
    /// `self` at a time. The chunks are mutable and do not overlap.
    ///
    /// If the number of elements in the iterator is not divisible by
    /// `chunk_size`, the last chunk may be shorter than `chunk_size`.  All
    /// other chunks will have that exact length.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    /// let mut array = [1, 2, 3, 4, 5];
    /// array.par_chunks_mut(2)
    ///      .for_each(|slice| slice.reverse());
    /// assert_eq!(array, [2, 1, 4, 3, 5]);
    /// ```
    fn par_chunks_mut(&mut self, chunk_size: usize) -> ChunksMut<'_, T> {
        assert!(chunk_size != 0, "chunk_size must not be zero");
        ChunksMut::new(chunk_size, self.as_parallel_slice_mut())
    }

    /// Returns a parallel iterator over `chunk_size` elements of
    /// `self` at a time. The chunks are mutable and do not overlap.
    ///
    /// If `chunk_size` does not divide the length of the slice, then the
    /// last up to `chunk_size-1` elements will be omitted and can be
    /// retrieved from the remainder function of the iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    /// let mut array = [1, 2, 3, 4, 5];
    /// array.par_chunks_exact_mut(3)
    ///      .for_each(|slice| slice.reverse());
    /// assert_eq!(array, [3, 2, 1, 4, 5]);
    /// ```
    fn par_chunks_exact_mut(&mut self, chunk_size: usize) -> ChunksExactMut<'_, T> {
        assert!(chunk_size != 0, "chunk_size must not be zero");
        ChunksExactMut::new(chunk_size, self.as_parallel_slice_mut())
    }

    /// Returns a parallel iterator over at most `chunk_size` elements of `self` at a time,
    /// starting at the end. The chunks are mutable and do not overlap.
    ///
    /// If the number of elements in the iterator is not divisible by
    /// `chunk_size`, the last chunk may be shorter than `chunk_size`.  All
    /// other chunks will have that exact length.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    /// let mut array = [1, 2, 3, 4, 5];
    /// array.par_rchunks_mut(2)
    ///      .for_each(|slice| slice.reverse());
    /// assert_eq!(array, [1, 3, 2, 5, 4]);
    /// ```
    fn par_rchunks_mut(&mut self, chunk_size: usize) -> RChunksMut<'_, T> {
        assert!(chunk_size != 0, "chunk_size must not be zero");
        RChunksMut::new(chunk_size, self.as_parallel_slice_mut())
    }

    /// Returns a parallel iterator over `chunk_size` elements of `self` at a time,
    /// starting at the end. The chunks are mutable and do not overlap.
    ///
    /// If `chunk_size` does not divide the length of the slice, then the
    /// last up to `chunk_size-1` elements will be omitted and can be
    /// retrieved from the remainder function of the iterator.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    /// let mut array = [1, 2, 3, 4, 5];
    /// array.par_rchunks_exact_mut(3)
    ///      .for_each(|slice| slice.reverse());
    /// assert_eq!(array, [1, 2, 5, 4, 3]);
    /// ```
    fn par_rchunks_exact_mut(&mut self, chunk_size: usize) -> RChunksExactMut<'_, T> {
        assert!(chunk_size != 0, "chunk_size must not be zero");
        RChunksExactMut::new(chunk_size, self.as_parallel_slice_mut())
    }

    /// Sorts the slice in parallel.
    ///
    /// This sort is stable (i.e. does not reorder equal elements) and `O(n log n)` worst-case.
    ///
    /// When applicable, unstable sorting is preferred because it is generally faster than stable
    /// sorting and it doesn't allocate auxiliary memory.
    /// See [`par_sort_unstable`](#method.par_sort_unstable).
    ///
    /// # Current implementation
    ///
    /// The current algorithm is an adaptive merge sort inspired by
    /// [timsort](https://en.wikipedia.org/wiki/Timsort).
    /// It is designed to be very fast in cases where the slice is nearly sorted, or consists of
    /// two or more sorted sequences concatenated one after another.
    ///
    /// Also, it allocates temporary storage the same size as `self`, but for very short slices a
    /// non-allocating insertion sort is used instead.
    ///
    /// In order to sort the slice in parallel, the slice is first divided into smaller chunks and
    /// all chunks are sorted in parallel. Then, adjacent chunks that together form non-descending
    /// or descending runs are concatenated. Finally, the remaining chunks are merged together using
    /// parallel subdivision of chunks and parallel merge operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    ///
    /// let mut v = [-5, 4, 1, -3, 2];
    ///
    /// v.par_sort();
    /// assert_eq!(v, [-5, -3, 1, 2, 4]);
    /// ```
    fn par_sort(&mut self)
    where
        T: Ord,
    {
        par_mergesort(self.as_parallel_slice_mut(), T::lt);
    }

    /// Sorts the slice in parallel with a comparator function.
    ///
    /// This sort is stable (i.e. does not reorder equal elements) and `O(n log n)` worst-case.
    ///
    /// When applicable, unstable sorting is preferred because it is generally faster than stable
    /// sorting and it doesn't allocate auxiliary memory.
    /// See [`par_sort_unstable_by`](#method.par_sort_unstable_by).
    ///
    /// # Current implementation
    ///
    /// The current algorithm is an adaptive merge sort inspired by
    /// [timsort](https://en.wikipedia.org/wiki/Timsort).
    /// It is designed to be very fast in cases where the slice is nearly sorted, or consists of
    /// two or more sorted sequences concatenated one after another.
    ///
    /// Also, it allocates temporary storage the same size as `self`, but for very short slices a
    /// non-allocating insertion sort is used instead.
    ///
    /// In order to sort the slice in parallel, the slice is first divided into smaller chunks and
    /// all chunks are sorted in parallel. Then, adjacent chunks that together form non-descending
    /// or descending runs are concatenated. Finally, the remaining chunks are merged together using
    /// parallel subdivision of chunks and parallel merge operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    ///
    /// let mut v = [5, 4, 1, 3, 2];
    /// v.par_sort_by(|a, b| a.cmp(b));
    /// assert_eq!(v, [1, 2, 3, 4, 5]);
    ///
    /// // reverse sorting
    /// v.par_sort_by(|a, b| b.cmp(a));
    /// assert_eq!(v, [5, 4, 3, 2, 1]);
    /// ```
    fn par_sort_by<F>(&mut self, compare: F)
    where
        F: Fn(&T, &T) -> Ordering + Sync,
    {
        par_mergesort(self.as_parallel_slice_mut(), |a, b| {
            compare(a, b) == Ordering::Less
        });
    }

    /// Sorts the slice in parallel with a key extraction function.
    ///
    /// This sort is stable (i.e. does not reorder equal elements) and `O(n log n)` worst-case.
    ///
    /// When applicable, unstable sorting is preferred because it is generally faster than stable
    /// sorting and it doesn't allocate auxiliary memory.
    /// See [`par_sort_unstable_by_key`](#method.par_sort_unstable_by_key).
    ///
    /// # Current implementation
    ///
    /// The current algorithm is an adaptive merge sort inspired by
    /// [timsort](https://en.wikipedia.org/wiki/Timsort).
    /// It is designed to be very fast in cases where the slice is nearly sorted, or consists of
    /// two or more sorted sequences concatenated one after another.
    ///
    /// Also, it allocates temporary storage the same size as `self`, but for very short slices a
    /// non-allocating insertion sort is used instead.
    ///
    /// In order to sort the slice in parallel, the slice is first divided into smaller chunks and
    /// all chunks are sorted in parallel. Then, adjacent chunks that together form non-descending
    /// or descending runs are concatenated. Finally, the remaining chunks are merged together using
    /// parallel subdivision of chunks and parallel merge operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    ///
    /// let mut v = [-5i32, 4, 1, -3, 2];
    ///
    /// v.par_sort_by_key(|k| k.abs());
    /// assert_eq!(v, [1, 2, -3, 4, -5]);
    /// ```
    fn par_sort_by_key<B, F>(&mut self, f: F)
    where
        B: Ord,
        F: Fn(&T) -> B + Sync,
    {
        par_mergesort(self.as_parallel_slice_mut(), |a, b| f(a).lt(&f(b)));
    }

    /// Sorts the slice in parallel, but may not preserve the order of equal elements.
    ///
    /// This sort is unstable (i.e. may reorder equal elements), in-place (i.e. does not allocate),
    /// and `O(n log n)` worst-case.
    ///
    /// # Current implementation
    ///
    /// The current algorithm is based on Orson Peters' [pattern-defeating quicksort][pdqsort],
    /// which is a quicksort variant designed to be very fast on certain kinds of patterns,
    /// sometimes achieving linear time. It is randomized but deterministic, and falls back to
    /// heapsort on degenerate inputs.
    ///
    /// It is generally faster than stable sorting, except in a few special cases, e.g. when the
    /// slice consists of several concatenated sorted sequences.
    ///
    /// All quicksorts work in two stages: partitioning into two halves followed by recursive
    /// calls. The partitioning phase is sequential, but the two recursive calls are performed in
    /// parallel.
    ///
    /// [pdqsort]: https://github.com/orlp/pdqsort
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    ///
    /// let mut v = [-5, 4, 1, -3, 2];
    ///
    /// v.par_sort_unstable();
    /// assert_eq!(v, [-5, -3, 1, 2, 4]);
    /// ```
    fn par_sort_unstable(&mut self)
    where
        T: Ord,
    {
        par_quicksort(self.as_parallel_slice_mut(), T::lt);
    }

    /// Sorts the slice in parallel with a comparator function, but may not preserve the order of
    /// equal elements.
    ///
    /// This sort is unstable (i.e. may reorder equal elements), in-place (i.e. does not allocate),
    /// and `O(n log n)` worst-case.
    ///
    /// # Current implementation
    ///
    /// The current algorithm is based on Orson Peters' [pattern-defeating quicksort][pdqsort],
    /// which is a quicksort variant designed to be very fast on certain kinds of patterns,
    /// sometimes achieving linear time. It is randomized but deterministic, and falls back to
    /// heapsort on degenerate inputs.
    ///
    /// It is generally faster than stable sorting, except in a few special cases, e.g. when the
    /// slice consists of several concatenated sorted sequences.
    ///
    /// All quicksorts work in two stages: partitioning into two halves followed by recursive
    /// calls. The partitioning phase is sequential, but the two recursive calls are performed in
    /// parallel.
    ///
    /// [pdqsort]: https://github.com/orlp/pdqsort
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    ///
    /// let mut v = [5, 4, 1, 3, 2];
    /// v.par_sort_unstable_by(|a, b| a.cmp(b));
    /// assert_eq!(v, [1, 2, 3, 4, 5]);
    ///
    /// // reverse sorting
    /// v.par_sort_unstable_by(|a, b| b.cmp(a));
    /// assert_eq!(v, [5, 4, 3, 2, 1]);
    /// ```
    fn par_sort_unstable_by<F>(&mut self, compare: F)
    where
        F: Fn(&T, &T) -> Ordering + Sync,
    {
        par_quicksort(self.as_parallel_slice_mut(), |a, b| {
            compare(a, b) == Ordering::Less
        });
    }

    /// Sorts the slice in parallel with a key extraction function, but may not preserve the order
    /// of equal elements.
    ///
    /// This sort is unstable (i.e. may reorder equal elements), in-place (i.e. does not allocate),
    /// and `O(n log n)` worst-case.
    ///
    /// # Current implementation
    ///
    /// The current algorithm is based on Orson Peters' [pattern-defeating quicksort][pdqsort],
    /// which is a quicksort variant designed to be very fast on certain kinds of patterns,
    /// sometimes achieving linear time. It is randomized but deterministic, and falls back to
    /// heapsort on degenerate inputs.
    ///
    /// It is generally faster than stable sorting, except in a few special cases, e.g. when the
    /// slice consists of several concatenated sorted sequences.
    ///
    /// All quicksorts work in two stages: partitioning into two halves followed by recursive
    /// calls. The partitioning phase is sequential, but the two recursive calls are performed in
    /// parallel.
    ///
    /// [pdqsort]: https://github.com/orlp/pdqsort
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon::prelude::*;
    ///
    /// let mut v = [-5i32, 4, 1, -3, 2];
    ///
    /// v.par_sort_unstable_by_key(|k| k.abs());
    /// assert_eq!(v, [1, 2, -3, 4, -5]);
    /// ```
    fn par_sort_unstable_by_key<B, F>(&mut self, f: F)
    where
        B: Ord,
        F: Fn(&T) -> B + Sync,
    {
        par_quicksort(self.as_parallel_slice_mut(), |a, b| f(a).lt(&f(b)));
    }
}

impl<T: Send> ParallelSliceMut<T> for [T] {
    #[inline]
    fn as_parallel_slice_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<'data, T: Sync + 'data> IntoParallelIterator for &'data [T] {
    type Item = &'data T;
    type Iter = Iter<'data, T>;

    fn into_par_iter(self) -> Self::Iter {
        Iter { slice: self }
    }
}

impl<'data, T: Send + 'data> IntoParallelIterator for &'data mut [T] {
    type Item = &'data mut T;
    type Iter = IterMut<'data, T>;

    fn into_par_iter(self) -> Self::Iter {
        IterMut { slice: self }
    }
}

/// Parallel iterator over immutable items in a slice
#[derive(Debug)]
pub struct Iter<'data, T: Sync> {
    slice: &'data [T],
}

impl<'data, T: Sync> Clone for Iter<'data, T> {
    fn clone(&self) -> Self {
        Iter { ..*self }
    }
}

impl<'data, T: Sync + 'data> ParallelIterator for Iter<'data, T> {
    type Item = &'data T;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<'data, T: Sync + 'data> IndexedParallelIterator for Iter<'data, T> {
    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: Consumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn len(&self) -> usize {
        self.slice.len()
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        callback.callback(IterProducer { slice: self.slice })
    }
}

struct IterProducer<'data, T: Sync> {
    slice: &'data [T],
}

impl<'data, T: 'data + Sync> Producer for IterProducer<'data, T> {
    type Item = &'data T;
    type IntoIter = ::std::slice::Iter<'data, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.slice.iter()
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.slice.split_at(index);
        (IterProducer { slice: left }, IterProducer { slice: right })
    }
}

/// Parallel iterator over immutable overlapping windows of a slice
#[derive(Debug)]
pub struct Windows<'data, T: Sync> {
    window_size: usize,
    slice: &'data [T],
}

impl<'data, T: Sync> Clone for Windows<'data, T> {
    fn clone(&self) -> Self {
        Windows { ..*self }
    }
}

impl<'data, T: Sync + 'data> ParallelIterator for Windows<'data, T> {
    type Item = &'data [T];

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<'data, T: Sync + 'data> IndexedParallelIterator for Windows<'data, T> {
    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: Consumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn len(&self) -> usize {
        assert!(self.window_size >= 1);
        self.slice.len().saturating_sub(self.window_size - 1)
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        callback.callback(WindowsProducer {
            window_size: self.window_size,
            slice: self.slice,
        })
    }
}

struct WindowsProducer<'data, T: Sync> {
    window_size: usize,
    slice: &'data [T],
}

impl<'data, T: 'data + Sync> Producer for WindowsProducer<'data, T> {
    type Item = &'data [T];
    type IntoIter = ::std::slice::Windows<'data, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.slice.windows(self.window_size)
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let left_index = cmp::min(self.slice.len(), index + (self.window_size - 1));
        let left = &self.slice[..left_index];
        let right = &self.slice[index..];
        (
            WindowsProducer {
                window_size: self.window_size,
                slice: left,
            },
            WindowsProducer {
                window_size: self.window_size,
                slice: right,
            },
        )
    }
}

/// Parallel iterator over mutable items in a slice
#[derive(Debug)]
pub struct IterMut<'data, T: Send> {
    slice: &'data mut [T],
}

impl<'data, T: Send + 'data> ParallelIterator for IterMut<'data, T> {
    type Item = &'data mut T;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.len())
    }
}

impl<'data, T: Send + 'data> IndexedParallelIterator for IterMut<'data, T> {
    fn drive<C>(self, consumer: C) -> C::Result
    where
        C: Consumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn len(&self) -> usize {
        self.slice.len()
    }

    fn with_producer<CB>(self, callback: CB) -> CB::Output
    where
        CB: ProducerCallback<Self::Item>,
    {
        callback.callback(IterMutProducer { slice: self.slice })
    }
}

struct IterMutProducer<'data, T: Send> {
    slice: &'data mut [T],
}

impl<'data, T: 'data + Send> Producer for IterMutProducer<'data, T> {
    type Item = &'data mut T;
    type IntoIter = ::std::slice::IterMut<'data, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.slice.iter_mut()
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.slice.split_at_mut(index);
        (
            IterMutProducer { slice: left },
            IterMutProducer { slice: right },
        )
    }
}

/// Parallel iterator over slices separated by a predicate
pub struct Split<'data, T, P> {
    slice: &'data [T],
    separator: P,
}

impl<'data, T, P: Clone> Clone for Split<'data, T, P> {
    fn clone(&self) -> Self {
        Split {
            separator: self.separator.clone(),
            ..*self
        }
    }
}

impl<'data, T: Debug, P> Debug for Split<'data, T, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Split").field("slice", &self.slice).finish()
    }
}

impl<'data, T, P> ParallelIterator for Split<'data, T, P>
where
    P: Fn(&T) -> bool + Sync + Send,
    T: Sync,
{
    type Item = &'data [T];

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let producer = SplitProducer::new(self.slice, &self.separator);
        bridge_unindexed(producer, consumer)
    }
}

/// Implement support for `SplitProducer`.
impl<'data, T, P> Fissile<P> for &'data [T]
where
    P: Fn(&T) -> bool,
{
    fn length(&self) -> usize {
        self.len()
    }

    fn midpoint(&self, end: usize) -> usize {
        end / 2
    }

    fn find(&self, separator: &P, start: usize, end: usize) -> Option<usize> {
        self[start..end].iter().position(separator)
    }

    fn rfind(&self, separator: &P, end: usize) -> Option<usize> {
        self[..end].iter().rposition(separator)
    }

    fn split_once(self, index: usize) -> (Self, Self) {
        let (left, right) = self.split_at(index);
        (left, &right[1..]) // skip the separator
    }

    fn fold_splits<F>(self, separator: &P, folder: F, skip_last: bool) -> F
    where
        F: Folder<Self>,
        Self: Send,
    {
        let mut split = self.split(separator);
        if skip_last {
            split.next_back();
        }
        folder.consume_iter(split)
    }
}

/// Parallel iterator over mutable slices separated by a predicate
pub struct SplitMut<'data, T, P> {
    slice: &'data mut [T],
    separator: P,
}

impl<'data, T: Debug, P> Debug for SplitMut<'data, T, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SplitMut")
            .field("slice", &self.slice)
            .finish()
    }
}

impl<'data, T, P> ParallelIterator for SplitMut<'data, T, P>
where
    P: Fn(&T) -> bool + Sync + Send,
    T: Send,
{
    type Item = &'data mut [T];

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        let producer = SplitProducer::new(self.slice, &self.separator);
        bridge_unindexed(producer, consumer)
    }
}

/// Implement support for `SplitProducer`.
impl<'data, T, P> Fissile<P> for &'data mut [T]
where
    P: Fn(&T) -> bool,
{
    fn length(&self) -> usize {
        self.len()
    }

    fn midpoint(&self, end: usize) -> usize {
        end / 2
    }

    fn find(&self, separator: &P, start: usize, end: usize) -> Option<usize> {
        self[start..end].iter().position(separator)
    }

    fn rfind(&self, separator: &P, end: usize) -> Option<usize> {
        self[..end].iter().rposition(separator)
    }

    fn split_once(self, index: usize) -> (Self, Self) {
        let (left, right) = self.split_at_mut(index);
        (left, &mut right[1..]) // skip the separator
    }

    fn fold_splits<F>(self, separator: &P, folder: F, skip_last: bool) -> F
    where
        F: Folder<Self>,
        Self: Send,
    {
        let mut split = self.split_mut(separator);
        if skip_last {
            split.next_back();
        }
        folder.consume_iter(split)
    }
}