use aci_map::{
    air::{AirLeveler, OxygenUser},
    liquids::{LiquidData, LiquidLeveler},
    objects::environment_object::EnvironmentObject,
    Map, MapObject,
};
use criterion::{black_box, criterion_group, Criterion};

fn simulate_map(map: &mut dyn MapObject) {
    map.perform_simulation_tick(0.05);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut map: Map<500, 500> = Map::new_default();

    map.objects_mut()
        .push_object::<EnvironmentObject>(AirLeveler {
            x: 0,
            y: 0,
            nitrogen: 0.79,
            oxygen: 0.00,
            fumes: 0.0,
        });
    map.objects_mut()
        .push_object::<EnvironmentObject>(AirLeveler {
            x: 9,
            y: 9,
            nitrogen: 0.79,
            oxygen: 0.21,
            fumes: 0.00,
        });
    map.objects_mut()
        .push_object::<EnvironmentObject>(OxygenUser {
            x: 50,
            y: 50,
            change_per_sec: 0.001,
        });

    map.objects_mut()
        .push_object::<EnvironmentObject>(LiquidLeveler {
            x: 99,
            y: 0,
            target: LiquidData::Water { level: 1.0 },
        });
    map.objects_mut()
        .push_object::<EnvironmentObject>(LiquidLeveler {
            x: 99,
            y: 9,
            target: LiquidData::Lava { level: 1.0 },
        });

    for (x, y) in map
        .all_tile_coords()
        .filter(|(x, y)| *x > 90 && *x < 120 && *y < 20)
    {
        map.tiles[x][y].ground_level = -1.1;
    }

    let mut g = c.benchmark_group("simulate");
    g.warm_up_time(std::time::Duration::from_secs(15));
    g.throughput(criterion::Throughput::Elements(1));
    g.bench_function("500x500", |b| b.iter(|| simulate_map(black_box(&mut map))));
}

criterion_group!(benches, criterion_benchmark);
fn main() {
    rayon::ThreadPoolBuilder::new()
        .stack_size(64 * 1024 * 1024)
        .build_global()
        .unwrap();

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
