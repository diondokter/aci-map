use std::time::Duration;

use aci_map::{AirLeveler, Map};
use criterion::{black_box, criterion_group, Criterion};

fn simulate_map<const WIDTH: usize, const HEIGHT: usize>(map: &mut Map<WIDTH, HEIGHT>) {
    map.simulate(0.05);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut map: Map<500, 500> = Map::new_default();

    for i in 0..500 {
        map.air_levelers.push(AirLeveler {
            x: i,
            y: i,
            target_air_pressure: 1.0,
        });
    }

    let mut g = c.benchmark_group("simulate");
    g.measurement_time(Duration::from_secs(60));
    g.throughput(criterion::Throughput::Elements(1));
    g.bench_function("500x500", |b| b.iter(|| simulate_map(black_box(&mut map))));
}

criterion_group!(benches, criterion_benchmark);
fn main() {
    std::thread::Builder::new()
        .name("TestThread".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            benches();
            Criterion::default().configure_from_args().final_summary();
        })
        .unwrap()
        .join()
        .unwrap();
}
