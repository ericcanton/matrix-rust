//! Compressed matrices.
//!
//! The storage is suitable for generic sparse matrices. Data are stored in one
//! of the following formats:
//!
//! * the [compressed-column][1] format or
//! * the [compressed-row][2] format.
//!
//! [1]: http://netlib.org/linalg/html_templates/node92.html
//! [2]: http://netlib.org/linalg/html_templates/node91.html

use std::{iter, mem};

use {Dense, Element, Matrix, Position, Size};

/// A compressed matrix.
#[derive(Clone, Debug, PartialEq)]
pub struct Compressed<T: Element> {
    /// The number of rows.
    pub rows: usize,
    /// The number of columns.
    pub columns: usize,
    /// The number of nonzero elements.
    pub nonzeros: usize,
    /// The storage format.
    pub format: Format,
    /// The values of the nonzero elements.
    pub values: Vec<T>,
    /// The indices of rows when `format = Column` or columns when `format =
    /// Row` of the nonzero elements.
    pub indices: Vec<usize>,
    /// The offsets of columns when `format = Column` or rows when `format =
    /// Row` such that the values and indices of the `i`th column when `format =
    /// Column` or the `i`th row when `format = Row` are stored starting from
    /// `values[j]` and `indices[j]`, respectively, where `j = offsets[i]`. The
    /// vector has one additional element, which is always equal to `nonzeros`.
    pub offsets: Vec<usize>,
}

/// A format of a compressed matrix.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    /// The compressed-column format.
    Column,
    /// The compressed-row format.
    Row,
}

/// A sparse iterator of a compressed matrix.
pub struct Iterator<'l, T: 'l + Element> {
    matrix: &'l Compressed<T>,
    taken: usize,
    major: usize,
}

macro_rules! debug_valid(
    ($matrix:ident) => (debug_assert!(
        $matrix.nonzeros == $matrix.values.len() &&
        $matrix.nonzeros == $matrix.indices.len() &&
        match $matrix.format {
            Format::Column => $matrix.columns + 1 == $matrix.offsets.len(),
            Format::Row => $matrix.rows + 1 == $matrix.offsets.len(),
        }
    ));
);

size!(Compressed);

impl<T: Element> Compressed<T> {
    /// Create a zero matrix.
    #[inline]
    pub fn new<S: Size>(size: S, format: Format) -> Self {
        Compressed::with_capacity(size, format, 0)
    }

    /// Create a zero matrix with a specific capacity.
    pub fn with_capacity<S: Size>(size: S, format: Format, capacity: usize) -> Self {
        let (rows, columns) = size.dimensions();
        Compressed {
            rows: rows,
            columns: columns,
            nonzeros: 0,
            format: format,
            values: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(capacity),
            offsets: match format {
                Format::Column => vec![0; columns + 1],
                Format::Row => vec![0; rows + 1],
            },
        }
    }

    /// Read an element.
    pub fn get<P: Position>(&self, position: P) -> T {
        let (mut i, mut j) = position.coordinates();
        debug_assert!(i < self.rows && j < self.columns);
        if let Format::Row = self.format {
            mem::swap(&mut i, &mut j);
        }
        for k in self.offsets[j]..self.offsets[j + 1] {
            if self.indices[k] == i {
                return self.values[k];
            }
            if self.indices[k] > i {
                break;
            }
        }
        T::zero()
    }

    /// Assign a value to an element.
    ///
    /// Note that the function treats zero values as any other.
    pub fn set<P: Position>(&mut self, position: P, value: T) {
        let (mut i, mut j) = position.coordinates();
        debug_assert!(i < self.rows && j < self.columns);
        if let Format::Row = self.format {
            mem::swap(&mut i, &mut j);
        }
        let mut k = self.offsets[j];
        while k < self.offsets[j + 1] {
            if self.indices[k] == i {
                self.values[k] = value;
                return;
            }
            if self.indices[k] > i {
                break;
            }
            k += 1;
        }
        self.nonzeros += 1;
        self.values.insert(k, value);
        self.indices.insert(k, i);
        for offset in &mut self.offsets[(j + 1)..] {
            *offset += 1;
        }
    }

    /// Return a sparse iterator.
    #[inline]
    pub fn iter<'l>(&'l self) -> Iterator<'l, T> {
        Iterator { matrix: self, taken: 0, major: 0 }
    }

    /// Resize the matrix.
    pub fn resize<S: Size>(&mut self, size: S) {
        let (rows, columns) = size.dimensions();
        if rows < self.rows || columns < self.columns {
            self.retain(|i, j, _| i < rows && j < columns);
        }
        let (from, into) = match self.format {
            Format::Column => (self.columns, columns),
            Format::Row => (self.rows, rows),
        };
        if from > into {
            self.offsets.truncate(into + 1);
        } else if from < into {
            self.offsets.extend(vec![self.nonzeros; into - from]);
        }
        self.columns = columns;
        self.rows = rows;
    }

    /// Retain the elements that satisfy a condition and discard the rest.
    pub fn retain<F>(&mut self, mut condition: F) where F: FnMut(usize, usize, &T) -> bool {
        let (mut k, mut major) = (0, 0);
        while k < self.indices.len() {
            while self.offsets[major + 1] <= k {
                major += 1;
            }
            let condition = match self.format {
                Format::Column => condition(self.indices[k], major, &self.values[k]),
                Format::Row => condition(major, self.indices[k], &self.values[k]),
            };
            if condition {
                k += 1;
                continue;
            }
            self.nonzeros -= 1;
            self.values.remove(k);
            self.indices.remove(k);
            for offset in &mut self.offsets[(major + 1)..] {
                *offset -= 1;
            }
        }
    }
}

impl<T: Element> Matrix for Compressed<T> {
    type Element = T;

    fn nonzeros(&self) -> usize {
        self.values.iter().fold(0, |sum, &value| if value.is_zero() { sum } else { sum + 1 })
    }

    fn transpose(&mut self) {
        let &mut Compressed { rows, columns, nonzeros, format, .. } = self;
        let mut matrix = Compressed::with_capacity((columns, rows), format, nonzeros);
        for (i, j, &value) in self.iter() {
            matrix.set((j, i), value);
        }
        *self = matrix;
    }

    #[inline]
    fn zero<S: Size>(size: S) -> Self {
        Compressed::new(size, Format::Column)
    }
}

impl<'l, T: Element> From<&'l Dense<T>> for Compressed<T> {
    fn from(dense: &'l Dense<T>) -> Self {
        let (rows, columns) = dense.dimensions();
        let mut matrix = Compressed::new((rows, columns), Format::Column);
        for (k, &value) in dense.values.iter().enumerate() {
            if !value.is_zero() {
                matrix.set((k % rows, k / rows), value);
            }
        }
        matrix
    }
}

impl<T: Element> From<Dense<T>> for Compressed<T> {
    #[inline]
    fn from(matrix: Dense<T>) -> Self {
        (&matrix).into()
    }
}

impl<'l, T: Element> From<&'l Compressed<T>> for Dense<T> {
    fn from(matrix: &'l Compressed<T>) -> Self {
        debug_valid!(matrix);

        let &Compressed {
            rows, columns, format, ref values, ref indices, ref offsets, ..
        } = matrix;

        let mut matrix = Dense::new((rows, columns));
        match format {
            Format::Row => for i in 0..rows {
                for k in offsets[i]..offsets[i + 1] {
                    matrix.values[indices[k] * rows + i] = values[k];
                }
            },
            Format::Column => for j in 0..columns {
                for k in offsets[j]..offsets[j + 1] {
                    matrix.values[j * rows + indices[k]] = values[k];
                }
            },
        }

        matrix
    }
}

impl<T: Element> From<Compressed<T>> for Dense<T> {
    fn from(matrix: Compressed<T>) -> Self {
        (&matrix).into()
    }
}

impl Format {
    /// Return the other format.
    #[inline]
    pub fn flip(&self) -> Self {
        match *self {
            Format::Column => Format::Row,
            Format::Row => Format::Column,
        }
    }
}

impl<'l, T: Element> iter::Iterator for Iterator<'l, T> {
    type Item = (usize, usize, &'l T);

    fn next(&mut self) -> Option<Self::Item> {
        let &mut Iterator { matrix, ref mut taken, ref mut major } = self;
        let k = *taken;
        if k == matrix.nonzeros {
            return None;
        }
        *taken += 1;
        while matrix.offsets[*major + 1] <= k {
            *major += 1;
        }
        Some(match matrix.format {
            Format::Column => (matrix.indices[k], *major, &matrix.values[k]),
            Format::Row => (*major, matrix.indices[k], &matrix.values[k]),
        })
    }
}

#[cfg(test)]
mod tests {
    use compressed::Format;
    use {Compressed, Dense, Matrix};

    macro_rules! new(
        ($rows:expr, $columns:expr, $nonzeros:expr, $format:expr,
         $values:expr, $indices:expr, $offsets:expr) => (
            Compressed { rows: $rows, columns: $columns, nonzeros: $nonzeros, format: $format,
                         values: $values, indices: $indices, offsets: $offsets }
        );
    );

    #[test]
    fn get() {
        let dense = Dense::from_vec(vec![
            0.0, 1.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 2.0, 3.0,
            0.0, 0.0, 0.0, 0.0, 4.0,
        ], (5, 3));

        let matrix: Compressed<_> = (&dense).into();
        assert_eq!(matrix.nonzeros, 4);

        for i in 0..5 {
            for j in 0..3 {
                assert_eq!(dense[(i, j)], matrix.get((i, j)));
            }
        }
    }

    #[test]
    fn set() {
        let mut dense = Dense::from_vec(vec![
            0.0, 1.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 2.0, 3.0,
            0.0, 0.0, 0.0, 0.0, 4.0,
        ], (5, 3));

        let mut matrix: Compressed<_> = (&dense).into();
        assert_eq!(matrix.nonzeros, 4);

        dense[(0, 0)] = 42.0;
        dense[(3, 1)] = 69.0;

        matrix.set((0, 0), 42.0);
        matrix.set((3, 1), 69.0);
        matrix.set((4, 0), 0.0);

        assert_eq!(matrix.nonzeros, 4 + 1 + (1 - 1) + 1);
        assert_eq!(dense, (&matrix).into());

        for i in 0..5 {
            for j in 0..3 {
                dense[(i, j)] = (j * 5 + i) as f64;
                matrix.set((i, j), (j * 5 + i) as f64);
            }
        }

        assert_eq!(matrix.nonzeros, 5 * 3);
        assert_eq!(dense, (&matrix).into());
    }

    #[test]
    fn nonzeros() {
        let matrix = new!(5, 7, 5, Format::Column, vec![1.0, 0.0, 3.0, 0.0, 5.0],
                          vec![1, 0, 3, 1, 4], vec![0, 0, 0, 1, 2, 2, 3, 5]);

        assert_eq!(matrix.nonzeros, 5);
        assert_eq!(matrix.nonzeros(), 3);
    }

    #[test]
    fn transpose() {
        let mut matrix = new!(5, 7, 5, Format::Column, vec![1.0, 2.0, 3.0, 4.0, 5.0],
                              vec![1, 0, 3, 1, 4], vec![0, 0, 0, 1, 2, 2, 3, 5]);

        matrix.transpose();

        assert_eq!(matrix, new!(7, 5, 5, Format::Column, vec![2.0, 1.0, 4.0, 3.0, 5.0],
                                vec![3, 2, 6, 5, 6], vec![0, 1, 3, 3, 4, 5]));
    }

    #[test]
    fn iter() {
        let matrix = new!(5, 7, 5, Format::Column, vec![1.0, 2.0, 3.0, 4.0, 5.0],
                          vec![1, 0, 3, 1, 4], vec![0, 0, 0, 1, 2, 2, 3, 5]);

        let result = matrix.iter().map(|(i, _, _)| i).collect::<Vec<_>>();
        assert_eq!(&result, &vec![1, 0, 3, 1, 4]);

        let result = matrix.iter().map(|(_, j, _)| j).collect::<Vec<_>>();
        assert_eq!(&result, &vec![2, 3, 5, 6, 6]);

        let result = matrix.iter().map(|(_, _, &value)| value).collect::<Vec<_>>();
        assert_eq!(&result, &vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    }

    #[test]
    fn resize_fewer_columns() {
        let mut matrix = new!(5, 7, 5, Format::Column, vec![1.0, 2.0, 3.0, 4.0, 5.0],
                              vec![1, 0, 3, 1, 4], vec![0, 0, 0, 1, 2, 2, 3, 5]);

        matrix.resize((5, 5));
        assert_eq!(matrix, new!(5, 5, 2, Format::Column, vec![1.0, 2.0],
                                vec![1, 0], vec![0, 0, 0, 1, 2, 2]));

        matrix.resize((5, 3));
        assert_eq!(matrix, new!(5, 3, 1, Format::Column, vec![1.0],
                                vec![1], vec![0, 0, 0, 1]));

        matrix.resize((5, 1));
        assert_eq!(matrix, new!(5, 1, 0, Format::Column, vec![],
                                vec![], vec![0, 0]));
    }

    #[test]
    fn resize_fewer_rows() {
        let mut matrix = new!(5, 7, 5, Format::Column, vec![1.0, 2.0, 3.0, 4.0, 5.0],
                              vec![1, 0, 3, 1, 4], vec![0, 0, 0, 1, 2, 2, 3, 5]);

        matrix.resize((3, 7));
        assert_eq!(matrix, new!(3, 7, 3, Format::Column, vec![1.0, 2.0, 4.0],
                                vec![1, 0, 1], vec![0, 0, 0, 1, 2, 2, 2, 3]));

        matrix.resize((1, 7));
        assert_eq!(matrix, new!(1, 7, 1, Format::Column, vec![2.0],
                                vec![0], vec![0, 0, 0, 0, 1, 1, 1, 1]));
    }

    #[test]
    fn resize_more_columns() {
        let mut matrix = new!(5, 7, 4, Format::Column, vec![1.0, 2.0, 3.0, 4.0],
                              vec![1, 1, 3, 4], vec![0, 0, 0, 1, 2, 2, 3, 4]);

        matrix.resize((5, 9));
        assert_eq!(matrix, new!(5, 9, 4, Format::Column, vec![1.0, 2.0, 3.0, 4.0],
                                vec![1, 1, 3, 4], vec![0, 0, 0, 1, 2, 2, 3, 4, 4, 4]));

        matrix.resize((5, 11));
        assert_eq!(matrix, new!(5, 11, 4, Format::Column, vec![1.0, 2.0, 3.0, 4.0],
                                vec![1, 1, 3, 4], vec![0, 0, 0, 1, 2, 2, 3, 4, 4, 4, 4, 4]));
    }

    #[test]
    fn resize_more_rows() {
        let mut matrix = new!(5, 7, 4, Format::Column, vec![1.0, 2.0, 3.0, 4.0],
                              vec![1, 1, 3, 4], vec![0, 0, 0, 1, 2, 2, 3, 4]);

        matrix.resize((7, 7));
        assert_eq!(matrix, new!(7, 7, 4, Format::Column, vec![1.0, 2.0, 3.0, 4.0],
                                vec![1, 1, 3, 4], vec![0, 0, 0, 1, 2, 2, 3, 4]));

        matrix.resize((9, 7));
        assert_eq!(matrix, new!(9, 7, 4, Format::Column, vec![1.0, 2.0, 3.0, 4.0],
                                vec![1, 1, 3, 4], vec![0, 0, 0, 1, 2, 2, 3, 4]));
    }

    #[test]
    fn from_dense() {
        let matrix = Dense::from_vec(vec![
            0.0, 1.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 2.0, 3.0,
            0.0, 0.0, 0.0, 0.0, 4.0,
        ], (5, 3));

        let matrix: Compressed<_> = matrix.into();

        assert_eq!(matrix, new!(5, 3, 4, Format::Column, vec![1.0, 2.0, 3.0, 4.0],
                                vec![1, 3, 4, 4], vec![0, 1, 3, 4]));
    }

    #[test]
    fn into_dense() {
        let matrix = new!(5, 3, 3, Format::Column, vec![1.0, 2.0, 3.0],
                          vec![0, 1, 2], vec![0, 1, 2, 3]);

        let matrix: Dense<_> = matrix.into();

        assert_eq!(&*matrix, &[
            1.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 2.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 3.0, 0.0, 0.0,
        ]);
    }
}
