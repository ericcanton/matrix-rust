use std::ops::{Deref, DerefMut};

use {BandMatrix, DenseMatrix, Element};

/// A diagonal matrix.
#[derive(Clone, Debug)]
pub struct DiagonalMatrix<T> {
    /// The number of rows.
    pub rows: usize,
    /// The number of columns.
    pub columns: usize,
    /// The values of the diagonal elements.
    pub data: Vec<T>,
}

impl<T> From<DiagonalMatrix<T>> for BandMatrix<T> where T: Element {
    #[inline]
    fn from(diagonal: DiagonalMatrix<T>) -> BandMatrix<T> {
        BandMatrix {
            rows: diagonal.rows,
            columns: diagonal.columns,
            superdiagonals: 0,
            subdiagonals: 0,
            data: diagonal.data,
        }
    }
}

impl<T> From<DiagonalMatrix<T>> for DenseMatrix<T> where T: Element {
    #[inline]
    fn from(diagonal: DiagonalMatrix<T>) -> DenseMatrix<T> {
        <DiagonalMatrix<T> as Into<BandMatrix<T>>>::into(diagonal).into()
    }
}

impl<T> Deref for DiagonalMatrix<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.data.deref()
    }
}

impl<T> DerefMut for DiagonalMatrix<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.data.deref_mut()
    }
}
