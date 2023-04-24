use glam::{vec2, Vec2};

#[derive(Debug, Clone)]
pub struct Map<const WIDTH: usize, const HEIGHT: usize> {
    air_pressures: [[f32; HEIGHT]; WIDTH],
    particles: Vec<Particle>,
    pub air_levelers: Vec<AirLeveler>,
}

impl<const WIDTH: usize, const HEIGHT: usize> Map<WIDTH, HEIGHT> {
    const AIR_PRESSURE_PER_PARTICLE: f32 = 0.001;

    pub const fn new_default() -> Self {
        Self {
            air_pressures: [[0.0; HEIGHT]; WIDTH],
            particles: Vec::new(),
            air_levelers: Vec::new(),
        }
    }

    fn all_tile_coords() -> impl Iterator<Item = (usize, usize)> {
        (0..WIDTH)
            .map(|x| (0..HEIGHT).map(move |y| (x, y)))
            .flatten()
    }

    pub fn collect_air_pressure_map(&self) -> [[f32; HEIGHT]; WIDTH] {
        self.air_pressures
    }

    fn neighbour_tile_coords(
        target_tile_x: usize,
        target_tile_y: usize,
    ) -> impl Iterator<Item = (usize, usize)> + Clone {
        let has_neg_x_neighbour = target_tile_x > 0;
        let has_neg_y_neighbour = target_tile_y > 0;
        let has_pos_x_neighbour = target_tile_x < WIDTH - 1;
        let has_pos_y_neighbour = target_tile_y < HEIGHT - 1;

        [
            (has_neg_x_neighbour && has_neg_y_neighbour)
                .then(|| (target_tile_x - 1, target_tile_y - 1)),
            (has_neg_x_neighbour).then(|| (target_tile_x - 1, target_tile_y)),
            (has_neg_x_neighbour && has_pos_y_neighbour)
                .then(|| (target_tile_x - 1, target_tile_y + 1)),
            (has_neg_y_neighbour).then(|| (target_tile_x, target_tile_y - 1)),
            (has_pos_y_neighbour).then(|| (target_tile_x, target_tile_y + 1)),
            (has_pos_x_neighbour && has_neg_y_neighbour)
                .then(|| (target_tile_x + 1, target_tile_y - 1)),
            (has_pos_x_neighbour).then(|| (target_tile_x + 1, target_tile_y)),
            (has_pos_x_neighbour && has_pos_y_neighbour)
                .then(|| (target_tile_x + 1, target_tile_y + 1)),
        ]
        .into_iter()
        .filter_map(|t| t)
    }

    pub fn simulate(&mut self, delta_time: f32) {
        self.update_particles_existance();
        self.update_air_pressure_map();
        self.move_particles(delta_time);
    }

    fn update_particles_existance(&mut self) {
        for leveler in self.air_levelers.iter() {
            let air_pressure_difference =
                leveler.target_air_pressure - self.air_pressures[leveler.x][leveler.y];
            let num_particles_required =
                (air_pressure_difference / Self::AIR_PRESSURE_PER_PARTICLE) as isize;

            if num_particles_required >= 0 {
                for _ in 0..num_particles_required {
                    self.particles.push(Particle {
                        location: Vec2::new(
                            leveler.x as f32 + rand::random::<f32>(),
                            leveler.y as f32 + rand::random::<f32>(),
                        ),
                        velocity: Vec2::ZERO,
                    })
                }
            } else {
                let particles_to_remove = self
                    .particles
                    .iter()
                    .enumerate()
                    .filter(|(_, particle)| {
                        particle.location.x as usize == leveler.x
                            && particle.location.y as usize == leveler.y
                    })
                    .map(|(index, _)| index)
                    .take(num_particles_required.unsigned_abs())
                    .collect::<Vec<_>>();

                for index in particles_to_remove.into_iter().rev() {
                    self.particles.remove(index);
                }
            }
        }
    }

    fn update_air_pressure_map(&mut self) {
        self.air_pressures = [[0.0; HEIGHT]; WIDTH];

        for particle in self.particles.iter() {
            let (x, y) = particle.tile_location::<WIDTH, HEIGHT>();
            self.air_pressures[x][y] += Self::AIR_PRESSURE_PER_PARTICLE;
        }
    }

    fn move_particles(&mut self, delta_time: f32) {
        const VELOCITY_CHANGE_RATE: f32 = 1.0;

        for particle in self.particles.iter_mut() {
            let (x, y) = particle.tile_location::<WIDTH, HEIGHT>();

            let current_tile_pressure = self.air_pressures[x][y];
            let neighbours =
                Self::neighbour_tile_coords(x, y).map(|(x, y)| (x, y, self.air_pressures[x][y]));

            for (nx, ny, n_pressure) in neighbours {
                let pressure_difference = current_tile_pressure - n_pressure;
                let neighbour_center = vec2(nx as f32 + 0.5, ny as f32 + 0.5);
                let neighbour_direction = (neighbour_center - particle.location).normalize();
                let proximity_strenght = 1.0 / (neighbour_center - particle.location).length();
                particle.velocity += neighbour_direction
                    * pressure_difference
                    * proximity_strenght
                    * VELOCITY_CHANGE_RATE
                    * delta_time;
            }

            particle.location += particle.velocity * delta_time;
            particle.location = particle
                .location
                .clamp(Vec2::ZERO, vec2(WIDTH as f32, HEIGHT as f32));
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Default for Map<WIDTH, HEIGHT> {
    fn default() -> Self {
        Self::new_default()
    }
}

#[derive(Debug, Clone)]
pub struct AirLeveler {
    pub x: usize,
    pub y: usize,
    pub target_air_pressure: f32,
}

#[derive(Debug, Clone)]
pub struct Particle {
    location: glam::Vec2,
    velocity: glam::Vec2,
}

impl Particle {
    fn tile_location<const WIDTH: usize, const HEIGHT: usize>(&self) -> (usize, usize) {
        (
            (self.location.x as usize).clamp(0, WIDTH - 1),
            (self.location.y as usize).clamp(0, HEIGHT - 1),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, path::Path};

    use super::*;

    #[test]
    fn neighbours() {
        let neighbours = Map::<10, 10>::neighbour_tile_coords(0, 0).collect::<Vec<_>>();

        assert!(neighbours.contains(&(0, 1)));
        assert!(neighbours.contains(&(1, 1)));
        assert!(neighbours.contains(&(1, 0)));
        assert_eq!(neighbours.len(), 3);

        let neighbours = Map::<10, 10>::neighbour_tile_coords(9, 9).collect::<Vec<_>>();

        assert!(neighbours.contains(&(8, 9)));
        assert!(neighbours.contains(&(8, 8)));
        assert!(neighbours.contains(&(9, 8)));
        assert_eq!(neighbours.len(), 3);

        let neighbours = Map::<10, 10>::neighbour_tile_coords(5, 5).collect::<Vec<_>>();

        assert!(neighbours.contains(&(4, 4)));
        assert!(neighbours.contains(&(4, 5)));
        assert!(neighbours.contains(&(4, 6)));
        assert!(neighbours.contains(&(5, 4)));
        assert!(neighbours.contains(&(5, 6)));
        assert!(neighbours.contains(&(6, 4)));
        assert!(neighbours.contains(&(6, 5)));
        assert!(neighbours.contains(&(6, 6)));
        assert_eq!(neighbours.len(), 8);

        let neighbours = dbg!(Map::<10, 1>::neighbour_tile_coords(1, 0).collect::<Vec<_>>());

        assert!(neighbours.contains(&(0, 0)));
        assert!(neighbours.contains(&(2, 0)));
        assert_eq!(neighbours.len(), 2);
    }

    fn create_map_gif<const WIDTH: usize, const HEIGHT: usize>(
        path: impl AsRef<Path>,
        map: &mut Map<WIDTH, HEIGHT>,
        iterations: usize,
        frame_every_nth: usize,
        mut data_step: impl FnMut(&Map<WIDTH, HEIGHT>) -> [[f32; HEIGHT]; WIDTH],
        max_value: f32,
        delta_time: f32,
    ) {
        let mut image = File::create(path).unwrap();
        let mut encoder = gif::Encoder::new(&mut image, WIDTH as u16, HEIGHT as u16, &[]).unwrap();
        encoder.set_repeat(gif::Repeat::Infinite).unwrap();

        for i in 0..iterations {
            if i % frame_every_nth == 0 {
                let data = data_step(&map);

                let mut pixels = vec![0; WIDTH * HEIGHT * 3];
                for (i, (x, y)) in Map::<WIDTH, HEIGHT>::all_tile_coords().enumerate() {
                    if data[x][y].is_nan() {
                        continue;
                    }
                    pixels[i * 3 + 0] =
                        ((1.0 - data[x][y] / max_value).powf(0.1) * 255.0).clamp(0.0, 255.0) as u8;
                    pixels[i * 3 + 1] =
                        ((data[x][y] / max_value).powf(0.1) * 255.0).clamp(0.0, 255.0) as u8;
                    pixels[i * 3 + 2] = if data[x][y] > max_value { 255 } else { 0 };
                }
                encoder
                    .write_frame(&gif::Frame::from_rgb(
                        WIDTH as u16,
                        HEIGHT as u16,
                        &mut pixels,
                    ))
                    .unwrap();
            }

            map.simulate(delta_time)
        }
    }

    #[test]
    fn air_pressure() {
        std::thread::Builder::new()
            .name("TestThread".into())
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let mut map = Map::<10, 10>::new_default();
                // map.air_levelers.push(AirLeveler {
                //     x: 9,
                //     y: 0,
                //     target_air_pressure: 1.00,
                // });
                // map.air_levelers.push(AirLeveler {
                //     x: 0,
                //     y: 9,
                //     target_air_pressure: 1.00,
                // });
                map.air_levelers.push(AirLeveler {
                    x: 5,
                    y: 5,
                    target_air_pressure: 1.00,
                });

                create_map_gif(
                    "target/air_pressure.gif",
                    &mut map.clone(),
                    10000,
                    10,
                    |map| map.collect_air_pressure_map(),
                    1.0,
                    0.05,
                );
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn all_tile_coords() {
        let coords = Map::<1, 2>::all_tile_coords().collect::<Vec<_>>();
        assert_eq!(coords, vec![(0, 0), (0, 1)])
    }
}
