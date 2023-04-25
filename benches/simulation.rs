use aci_map::Map;
use criterion::{black_box, criterion_group, Criterion};

fn simulate_map<const WIDTH: usize, const HEIGHT: usize>(map: &mut Map<WIDTH, HEIGHT>) {
    map.simulate(0.05);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut map: Map<500, 500> = Map::new_default();

    for _ in 0..100000 {
        map.insert_random_particle();
    }

    let mut g = c.benchmark_group("simulate");

    g.throughput(criterion::Throughput::Elements(100000));
    g.bench_function("500x500 @ 100000", |b| b.iter(|| simulate_map(black_box(&mut map))));
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
