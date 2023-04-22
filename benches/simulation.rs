use aci_map::{AirLeveler, Map};
use criterion::{black_box, criterion_group, Criterion};

fn simulate_map<const WIDTH: usize, const HEIGHT: usize>(map: &mut Map<WIDTH, HEIGHT>) {
    map.simulate(0.05);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut map: Map<500, 500> = Map::new_default();

    map.air_levelers.push(AirLeveler {
        x: 0,
        y: 0,
        air_pressure: 0.8,
        oxygen: 0.00,
    });
    map.air_levelers.push(AirLeveler {
        x: 9,
        y: 9,
        air_pressure: 1.0,
        oxygen: 0.21,
    });

    let mut g = c.benchmark_group("simulate");

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
