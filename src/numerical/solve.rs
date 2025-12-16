use crate::numerical::complex::c64;

fn sparse_matmul(
    values: &[c64],
    column_indices: &[u32],
    row_pointers: &[u32],
    vector: &[c64],
) -> Vec<c64> {
    let rows = row_pointers.len() - 1;
    let mut results = vec![c64::new(0., 0.); rows];

    for i in 0..rows {
        let start = row_pointers[i] as usize;
        let end = row_pointers[i + 1] as usize;
        for j in start..end {
            results[i] += values[j] * vector[column_indices[j] as usize];
        }
    }

    results
}

pub fn solve(
    values: Vec<c64>,
    column_indices: Vec<u32>,
    row_pointers: Vec<u32>,
    guess: Vec<c64>,
    residuals: Vec<c64>,
    step: f64,
    iterations: u32,
) -> Vec<c64> {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::f64::EPSILON;

    use super::*;

    #[test]
    fn test_sparse_matrix_multiplication() {
        let values = [5, 1, 2, 3]
            .into_iter()
            .map(|x| c64::new(x as f64, 0.))
            .collect::<Vec<_>>();

        let column_indices = vec![1, 0, 2, 2];
        let row_pointers = vec![0, 1, 3, 4];

        let vector = [2, 4, 3]
            .into_iter()
            .map(|x| c64::new(x as f64, 0.))
            .collect::<Vec<_>>();

        let result = sparse_matmul(
            &values[..],
            &column_indices[..],
            &row_pointers[..],
            &vector[..],
        );

        assert!((result[0].re - 20.).abs() < EPSILON);
        assert!((result[1].re - 8.).abs() < EPSILON);
        assert!((result[2].re - 9.).abs() < EPSILON);

        assert!((result[0].im).abs() < EPSILON);
        assert!((result[1].im).abs() < EPSILON);
        assert!((result[2].im).abs() < EPSILON);
    }
}
