#[cfg(not(bench))]
compile_error!("benchmarks must be run as `RUSTFLAGS=\"--cfg bench\" cargo bench --all-features`");

use criterion::{black_box, criterion_group, criterion_main, Criterion};

extern crate lispizzle;

fn create(qty: usize) -> String {
    let mut v = String::new();
    for i in 0..qty {
        if i % 2 == 0 {
            v.push('β');
        } else {
            v.push('b');
        }
    }
    v
}

#[inline(never)]
fn algo_count(v: &str) {
    black_box(lispizzle::parser::reader::util::count_chars(v.as_bytes()));
}

#[inline(never)]
fn naive_count(v: &str) {
    black_box(v.chars().count());
}

#[inline(never)]
fn algo_skip(v: &str, qty: usize) {
    let len = lispizzle::parser::reader::util::skip_chars(v.as_bytes(), qty);
    black_box(len);
}

#[inline(never)]
fn naive_skip(v: &str, qty: usize) {
    let mut len = None;
    for (i, (l, _)) in v.char_indices().enumerate() {
        if i == qty {
            len = Some(l);
            break;
        }
    }
    black_box(len.map(|len| &v[len..]));
}

#[inline(never)]
fn algo_skip_count(v: &str, qty: usize) {
    let len = lispizzle::parser::reader::util::skip_chars_count_nl(v.as_bytes(), qty);
    black_box(len);
}

#[inline(never)]
fn naive_skip_count(v: &str, qty: usize) {
    let mut count = 0;
    let mut len = None;
    for (i, (l, c)) in v.char_indices().enumerate() {
        if i == qty {
            len = Some(l);
            break;
        }
        if c == '\n' {
            count += 1;
        }
    }
    black_box(len.map(|l| (&v[l..], count)));
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let v = create(300_001);

    {
        let mut g = c.benchmark_group("count");

        g.bench_with_input("algo", &v, |b, v| b.iter(|| algo_count(v)));
        g.bench_with_input("naive", &v, |b, v| b.iter(|| naive_count(v)));
    }

    {
        let mut g = c.benchmark_group("skip 100");

        g.bench_with_input("algo", &v, |b, v| b.iter(|| algo_skip(v, black_box(100))));
        g.bench_with_input("naive", &v, |b, v| b.iter(|| naive_skip(v, black_box(100))));
    }
    {
        let mut g = c.benchmark_group("skip 300_000");

        g.bench_with_input("algo", &v, |b, v| {
            b.iter(|| algo_skip(v, black_box(300_000)))
        });
        g.bench_with_input("naive", &v, |b, v| {
            b.iter(|| naive_skip(v, black_box(300_000)))
        });
    }

    {
        let mut g = c.benchmark_group("skip-count 100");

        g.bench_with_input("algo", &v, |b, v| {
            b.iter(|| algo_skip_count(v, black_box(100)))
        });
        g.bench_with_input("naive", &v, |b, v| {
            b.iter(|| naive_skip_count(v, black_box(100)))
        });
    }
    {
        let mut g = c.benchmark_group("skip-count 300_000");

        g.bench_with_input("algo", &v, |b, v| {
            b.iter(|| algo_skip_count(v, black_box(300_000)))
        });
        g.bench_with_input("naive", &v, |b, v| {
            b.iter(|| naive_skip_count(v, black_box(300_000)))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
