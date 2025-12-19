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

fn vec_sub(a: &[c64], b: &[c64]) -> Vec<c64> {
    a.iter()
        .copied()
        .zip(b.iter().copied())
        .map(|(a, b)| a - b)
        .collect()
}

fn vec_add(a: &[c64], b: &[c64]) -> Vec<c64> {
    a.iter()
        .copied()
        .zip(b.iter().copied())
        .map(|(a, b)| a + b)
        .collect()
}

fn vec_dot(a: &[c64], b: &[c64]) -> c64 {
    let mut acc = c64::ZERO;

    a.iter()
        .copied()
        .zip(b.iter().copied())
        .map(|(a, b)| a * b.conj())
        .for_each(|z| acc += z);

    acc
}

fn vec_mul(a: &[c64], k: c64) -> Vec<c64> {
    a.iter().copied().map(|a| k * a).collect()
}

fn vec_norm(a: &[c64]) -> f64 {
    a.iter()
        .map(|&a| a.re * a.re + a.im * a.im)
        .sum::<f64>()
        .sqrt()
}

fn vec_add_in_place(a: &mut [c64], b: &[c64]) {
    for (a, &b) in a.iter_mut().zip(b) {
        *a += b;
    }
}

#[inline]
fn diag(values: &[c64], row_pointers: &[u32], column_indices: &[u32]) -> impl Iterator<Item = c64> {
    row_pointers
        .array_windows()
        .enumerate()
        .filter_map(|(row, &[start, end])| {
            let start = start as usize;
            let end = end as usize;

            column_indices[start..end]
                .iter()
                .zip(&values[start..end])
                .find_map(
                    |(&col, &val)| {
                        if col as usize == row { Some(val) } else { None }
                    },
                )
        })
}

// BiCGSTAB
pub fn solve(
    values: &[c64],
    column_indices: &[u32],
    row_pointers: &[u32],
    mut x: Vec<c64>,
    b: &[c64],
    max_iters: u32,
    tol: f64,
) -> Vec<c64> {
    let a_x0 = sparse_matmul(&values, &column_indices, &row_pointers, &x);
    let mut r = vec_sub(&b, &a_x0);

    let r_hat = r.clone();

    let mut p = r.clone();

    let mut rho_old = vec_dot(&r_hat, &r);

    let small = 1e-30f64;

    for _iter in 0..max_iters {
        let a_p = sparse_matmul(&values, &column_indices, &row_pointers, &p);

        let denom_alpha = vec_dot(&r_hat, &a_p);
        if denom_alpha.norm() < small {
            break;
        }
        let alpha = rho_old / denom_alpha;

        let alpha_a_p = vec_mul(&a_p, alpha);
        let s = vec_sub(&r, &alpha_a_p);

        if vec_norm(&s) < tol {
            let alpha_p = vec_mul(&p, alpha);
            vec_add_in_place(&mut x, &alpha_p);
            break;
        }

        let a_s = sparse_matmul(&values, &column_indices, &row_pointers, &s);

        let denom_omega = vec_dot(&a_s, &a_s);
        if denom_omega.norm() < small {
            break;
        }

        let omega = vec_dot(&a_s, &s) / denom_omega;

        let alpha_p = vec_mul(&p, alpha);
        let omega_s = vec_mul(&s, omega);
        vec_add_in_place(&mut x, &alpha_p);
        vec_add_in_place(&mut x, &omega_s);

        let omega_a_s = vec_mul(&a_s, omega);
        let r_new = vec_sub(&s, &omega_a_s);

        let rho_new = vec_dot(&r_hat, &r_new);
        if rho_new.norm() < small {
            break;
        }

        if omega == c64::new(0.0, 0.0) {
            break;
        }
        let beta = (rho_new / rho_old) * (alpha / omega);

        let omega_a_p = vec_mul(&a_p, omega);
        let p_minus = vec_sub(&p, &omega_a_p);
        let beta_term = vec_mul(&p_minus, beta);
        p = vec_add(&r_new, &beta_term);

        r = r_new;
        rho_old = rho_new;
    }

    x
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

    #[test]
    fn test_solve_real_numbers() {
        let values = vec![
            c64::new(4.0, 0.0),
            c64::new(1.0, 0.0),
            c64::new(1.0, 0.0),
            c64::new(3.0, 0.0),
            c64::new(1.0, 0.0),
            c64::new(1.0, 0.0),
            c64::new(2.0, 0.0),
        ];

        let column_indices = vec![0, 1, 0, 1, 2, 1, 2];

        let row_pointers = vec![0, 2, 5, 7];
        let b = vec![c64::new(1.0, 0.0), c64::new(2.0, 0.0), c64::new(3.0, 0.0)];
        let x0 = vec![c64::new(0.0, 0.0); 3];

        let x = solve(&values, &column_indices, &row_pointers, x0, &b, 1000, 1e-8);

        let ax = sparse_matmul(&values, &column_indices, &row_pointers, &x);
        let residual: Vec<c64> = b
            .iter()
            .zip(ax.iter())
            .map(|(&bi, &axi)| bi - axi)
            .collect();

        assert!(
            residual.iter().copied().map(|c| c.norm()).sum::<f64>() / (residual.len() as f64)
                < 1e-8
        );
    }
}
