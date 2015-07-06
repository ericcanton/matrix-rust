use {DenseMatrix, Element};

/// A band matrix.
///
/// Data are stored in the [format][1] adopted by [LAPACK][2].
///
/// [1]: http://www.netlib.org/lapack/lug/node124.html
/// [2]: http://www.netlib.org/lapack
#[derive(Clone, Debug)]
pub struct BandMatrix<T> {
    /// The number of rows.
    pub rows: usize,
    /// The number of columns.
    pub columns: usize,
    /// The number of superdiagonals.
    pub superdiagonals: usize,
    /// The number of subdiagonals.
    pub subdiagonals: usize,
    /// The values of the diagonal elements such that the first row corresponds
    /// to the uppermost superdiagonal while the last row corresponds to the
    /// lowest supdiagonal.
    pub data: Vec<T>,
}

impl<T> From<BandMatrix<T>> for DenseMatrix<T> where T: Element {
    fn from(band: BandMatrix<T>) -> DenseMatrix<T> {
        let BandMatrix { rows, columns, superdiagonals, subdiagonals, ref data } = band;

        let diagonals = superdiagonals + 1 + subdiagonals;
        debug_assert_eq!(data.len(), diagonals * columns);

        let mut dense = DenseMatrix {
            rows: rows,
            columns: columns,
            data: vec![Element::zero(); rows * columns],
        };

        for k in 1..(superdiagonals + 1) {
            for j in k..columns {
                let i = j - k;
                if i >= rows { break; }
                dense.data[j * rows + i] = data[j * diagonals + superdiagonals - k];
            }
        }
        for i in 0..columns {
            if i >= rows || i >= columns { break; }
            dense.data[i * rows + i] = data[i * diagonals + superdiagonals];
        }
        for k in 1..(subdiagonals + 1) {
            for j in 0..columns {
                let i = j + k;
                if i >= rows { break; }
                dense.data[j * rows + i] = data[j * diagonals + superdiagonals + k];
            }
        }

        dense
    }
}

#[cfg(test)]
mod tests {
    use {BandMatrix, DenseMatrix};

    #[test]
    fn into_tall_dense() {
        let band = BandMatrix {
            rows: 7,
            columns: 4,
            superdiagonals: 2,
            subdiagonals: 2,
            data: vec![
                0.0,  0.0,  1.0,  4.0,  8.0,
                0.0,  2.0,  5.0,  9.0, 12.0,
                3.0,  6.0, 10.0, 13.0, 15.0,
                7.0, 11.0, 14.0, 16.0, 17.0,
            ],
        };

        let dense: DenseMatrix<f64> = band.into();

        assert_eq!(&dense[..], &[
            1.0, 4.0,  8.0,  0.0,  0.0,  0.0, 0.0,
            2.0, 5.0,  9.0, 12.0,  0.0,  0.0, 0.0,
            3.0, 6.0, 10.0, 13.0, 15.0,  0.0, 0.0,
            0.0, 7.0, 11.0, 14.0, 16.0, 17.0, 0.0,
        ]);
    }

    #[test]
    fn into_wide_dense() {
        let band = BandMatrix {
            rows: 4,
            columns: 7,
            superdiagonals: 2,
            subdiagonals: 2,
            data: vec![
                 0.0,  0.0,  1.0,  4.0,  8.0,
                 0.0,  2.0,  5.0,  9.0, 13.0,
                 3.0,  6.0, 10.0, 14.0,  0.0,
                 7.0, 11.0, 15.0,  0.0,  0.0,
                12.0, 16.0,  0.0,  0.0,  0.0,
                17.0,  0.0,  0.0,  0.0,  0.0,
                 0.0,  0.0,  0.0,  0.0,  0.0,
            ],
        };

        let dense: DenseMatrix<f64> = band.into();

        assert_eq!(&dense[..], &[
            1.0, 4.0,  8.0,  0.0,
            2.0, 5.0,  9.0, 13.0,
            3.0, 6.0, 10.0, 14.0,
            0.0, 7.0, 11.0, 15.0,
            0.0, 0.0, 12.0, 16.0,
            0.0, 0.0,  0.0, 17.0,
            0.0, 0.0,  0.0,  0.0,
        ]);
    }
}
