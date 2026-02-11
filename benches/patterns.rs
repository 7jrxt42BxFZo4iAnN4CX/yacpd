//! Benchmarks for candlestick pattern detection.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use yacpd::prelude::*;

/// Simple test bar structure
#[derive(Debug, Clone, Copy)]
struct TestBar {
  o: f64,
  h: f64,
  l: f64,
  c: f64,
}

impl OHLCV for TestBar {
  fn open(&self) -> f64 {
    self.o
  }

  fn high(&self) -> f64 {
    self.h
  }

  fn low(&self) -> f64 {
    self.l
  }

  fn close(&self) -> f64 {
    self.c
  }

  fn volume(&self) -> f64 {
    1000.0
  }
}

/// Generate realistic random bars
fn generate_bars(n: usize) -> Vec<TestBar> {
  let mut bars = Vec::with_capacity(n);
  let mut price = 100.0;

  for i in 0..n {
    let change = ((i * 7 + 13) % 100) as f64 / 50.0 - 1.0; // Deterministic "random"
    let volatility = 2.0 + ((i * 3) % 10) as f64 / 5.0;

    let o = price;
    let c = price + change;
    let h = o.max(c) + volatility * 0.5;
    let l = o.min(c) - volatility * 0.5;

    bars.push(TestBar { o, h, l, c });
    price = c;
  }

  bars
}

fn bench_single_pattern(c: &mut Criterion) {
  let bars = generate_bars(1000);

  let engine =
    EngineBuilder::new().add(BuiltinDetector::Doji(DojiDetector::with_defaults())).build().unwrap();

  c.bench_function("scan_doji_1000_bars", |b| {
    b.iter(|| {
      let _ = black_box(engine.scan(black_box(&bars)));
    })
  });
}

fn bench_all_patterns(c: &mut Criterion) {
  let bars = generate_bars(1000);

  let engine = EngineBuilder::new().with_all_defaults().build().unwrap();

  c.bench_function("scan_all_patterns_1000_bars", |b| {
    b.iter(|| {
      let _ = black_box(engine.scan(black_box(&bars)));
    })
  });
}

fn bench_scaling(c: &mut Criterion) {
  let engine = EngineBuilder::new().with_all_defaults().build().unwrap();

  let mut group = c.benchmark_group("scaling");

  for size in [100, 500, 1000, 5000, 10000].iter() {
    let bars = generate_bars(*size);

    group.bench_with_input(BenchmarkId::new("scan", size), size, |b, _| {
      b.iter(|| {
        let _ = black_box(engine.scan(black_box(&bars)));
      })
    });
  }

  group.finish();
}

fn bench_parallel_scan(c: &mut Criterion) {
  let bars1 = generate_bars(1000);
  let bars2 = generate_bars(1000);
  let bars3 = generate_bars(1000);
  let bars4 = generate_bars(1000);

  let engine = EngineBuilder::new().with_all_defaults().build().unwrap();

  let instruments: Vec<(&str, &[TestBar])> =
    vec![("SYM1", &bars1), ("SYM2", &bars2), ("SYM3", &bars3), ("SYM4", &bars4)];

  c.bench_function("parallel_scan_4_instruments", |b| {
    b.iter(|| {
      let _ = black_box(scan_parallel(black_box(&engine), black_box(instruments.clone())));
    })
  });
}

fn bench_context_computation(c: &mut Criterion) {
  let bars = generate_bars(1000);

  let engine = EngineBuilder::new().with_all_defaults().build().unwrap();

  c.bench_function("compute_contexts_1000_bars", |b| {
    b.iter(|| {
      let _ = black_box(engine.compute_contexts(black_box(&bars)));
    })
  });
}

fn bench_scan_at(c: &mut Criterion) {
  let bars = generate_bars(1000);

  let engine = EngineBuilder::new().with_all_defaults().build().unwrap();

  let contexts = engine.compute_contexts(&bars);

  c.bench_function("scan_at_single_bar", |b| {
    b.iter(|| {
      let _ =
        black_box(engine.scan_at(black_box(&bars), black_box(500), black_box(&contexts[500])));
    })
  });
}

criterion_group!(
  benches,
  bench_single_pattern,
  bench_all_patterns,
  bench_scaling,
  bench_parallel_scan,
  bench_context_computation,
  bench_scan_at,
);

criterion_main!(benches);
