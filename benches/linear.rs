use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use electrocute::{LinearEquations, c64};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

const SEED: u64 = 42;

fn random_solvable_system(n: u32, nnz_per_row: usize, rng: &mut impl Rng) -> LinearEquations {
    let mut coords = Vec::new();

    for i in 0..n {
        coords.push((i, i));

        for _ in 0..nnz_per_row {
            let mut j = rng.gen_range(0..n);
            if j == i {
                j = (j + 1) % n;
            }
            coords.push((i, j));
        }
    }

    let mut le = LinearEquations::from_coordinates(coords);

    for i in 0..n {
        let start = le.row_pointers[i as usize] as usize;
        let end = le.row_pointers[i as usize + 1] as usize;

        let mut row_sum = 0.0;

        for k in start..end {
            let j = le.column_indices[k];
            if j != i {
                let v = rng.gen_range(-1.0..1.0);
                le.a[k] = c64::new(v, 0.0);
                row_sum += v.abs();
            }
        }

        let diag = le.value_map[&(i, i)];
        le.a[diag] = c64::new(row_sum + rng.gen_range(0.5..2.0), 0.0);
    }

    for i in 0..n {
        le.set_b(i, c64::new(rng.gen_range(-1.0..1.0), 0.0));
    }

    le
}

fn bench_linear_solve(c: &mut Criterion) {
    let mut group = c.benchmark_group("linear_equations");

    for &size in &[500u32, 1000, 2500] {
        group.bench_with_input(BenchmarkId::new("init", size), &size, |b, &n| {
            let mut rng = ChaCha8Rng::seed_from_u64(SEED);

            b.iter(|| black_box(random_solvable_system(n, 5, &mut rng)));
        });

        group.bench_with_input(BenchmarkId::new("solve", size), &size, |b, &n| {
            let mut rng = ChaCha8Rng::seed_from_u64(SEED);
            let mut le = random_solvable_system(n, 5, &mut rng);

            b.iter(|| {
                black_box(le.solve());
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_linear_solve);
criterion_main!(benches);
