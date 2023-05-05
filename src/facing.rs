/// A cardinal direction something can be facing to.
///
/// When used for rotation (for example in buildings), the North facing is the identity rotation.
///
/// The coord system has the 0,0 at the North-West.
/// So going north is -y, going east is +x, going south is +y, going west is -x.
#[derive(Debug, Clone, Copy, num_enum::UnsafeFromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum Facing {
    North,
    East,
    South,
    West,
}

impl Facing {
    pub fn move_coords_in_direction<const WIDTH: usize, const HEIGHT: usize>(
        &self,
        x: usize,
        y: usize,
    ) -> Option<(usize, usize)> {
        match self {
            Facing::North => (y > 0).then(|| (x, y - 1)),
            Facing::East => (x < WIDTH - 1).then(|| (x + 1, y)),
            Facing::South => (y < HEIGHT - 1).then(|| (x, y + 1)),
            Facing::West => (x > 0).then(|| (x - 1, y)),
        }
    }

    /// Rotates a facing. The default is North.
    ///
    /// So East rotate East = South.
    pub fn rotate(self, applied: Facing) -> Self {
        use num_enum::UnsafeFromPrimitive;
        let new_discriminant = (self as u8 + applied as u8) % 4;
        unsafe { Self::unchecked_transmute_from(new_discriminant) }
    }

    /// Rotate the given coords according to the facing.
    /// They will be rotated relative to 0,0
    pub fn rotate_isize_coords(&self, x: isize, y: isize) -> (isize, isize) {
        match self {
            Facing::North => (x, y),
            Facing::East => (-y, x),
            Facing::South => (-x, -y),
            Facing::West => (y, -x),
        }
    }

    /// Rotate the given coords according to the facing.
    /// They will be rotated relative to 0.5,0.5 (which is the middle of tile 0,0)
    pub fn rotate_f32_coords(&self, mut x: f32, mut y: f32) -> (f32, f32) {
        x -= 0.5;
        y -= 0.5;

        let (x, y) = match self {
            Facing::North => (x, y),
            Facing::East => (-y, x),
            Facing::South => (-x, -y),
            Facing::West => (y, -x),
        };

        (x + 0.5, y + 0.5)
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn facing_rotation() {
        assert_eq!(Facing::North.rotate(Facing::North), Facing::North);
        assert_eq!(Facing::North.rotate(Facing::East), Facing::East);
        assert_eq!(Facing::North.rotate(Facing::South), Facing::South);
        assert_eq!(Facing::North.rotate(Facing::West), Facing::West);

        assert_eq!(Facing::East.rotate(Facing::North), Facing::East);
        assert_eq!(Facing::East.rotate(Facing::East), Facing::South);
        assert_eq!(Facing::East.rotate(Facing::South), Facing::West);
        assert_eq!(Facing::East.rotate(Facing::West), Facing::North);

        assert_eq!(Facing::South.rotate(Facing::North), Facing::South);
        assert_eq!(Facing::South.rotate(Facing::East), Facing::West);
        assert_eq!(Facing::South.rotate(Facing::South), Facing::North);
        assert_eq!(Facing::South.rotate(Facing::West), Facing::East);

        assert_eq!(Facing::West.rotate(Facing::North), Facing::West);
        assert_eq!(Facing::West.rotate(Facing::East), Facing::North);
        assert_eq!(Facing::West.rotate(Facing::South), Facing::East);
        assert_eq!(Facing::West.rotate(Facing::West), Facing::South);
    }

    #[test]
    #[rustfmt::skip]
    fn facing_move_coords_in_direction() {
        assert_eq!(Facing::North.move_coords_in_direction::<5, 10>(2, 0), None);
        assert_eq!(Facing::North.move_coords_in_direction::<5, 10>(0, 1), Some((0, 0)));
        assert_eq!(Facing::North.move_coords_in_direction::<5, 10>(4, 9), Some((4, 8)));

        assert_eq!(Facing::East.move_coords_in_direction::<5, 10>(4, 2), None);
        assert_eq!(Facing::East.move_coords_in_direction::<5, 10>(3, 1), Some((4, 1)));
        assert_eq!(Facing::East.move_coords_in_direction::<5, 10>(0, 9), Some((1, 9)));

        assert_eq!(Facing::South.move_coords_in_direction::<5, 10>(4, 9), None);
        assert_eq!(Facing::South.move_coords_in_direction::<5, 10>(0, 8), Some((0, 9)));
        assert_eq!(Facing::South.move_coords_in_direction::<5, 10>(2, 0), Some((2, 1)));

        assert_eq!(Facing::West.move_coords_in_direction::<5, 10>(0, 6), None);
        assert_eq!(Facing::West.move_coords_in_direction::<5, 10>(1, 1), Some((0, 1)));
        assert_eq!(Facing::West.move_coords_in_direction::<5, 10>(4, 9), Some((3, 9)));
    }

    #[test]
    fn facing_rotate_isize() {
        assert_eq!(Facing::North.rotate_isize_coords(0, 0), (0, 0));
        assert_eq!(Facing::East.rotate_isize_coords(0, 0), (0, 0));
        assert_eq!(Facing::South.rotate_isize_coords(0, 0), (0, 0));
        assert_eq!(Facing::West.rotate_isize_coords(0, 0), (0, 0));

        assert_eq!(Facing::North.rotate_isize_coords(2, 1), (2, 1));
        assert_eq!(Facing::East.rotate_isize_coords(2, 1), (-1, 2));
        assert_eq!(Facing::South.rotate_isize_coords(2, 1), (-2, -1));
        assert_eq!(Facing::West.rotate_isize_coords(2, 1), (1, -2));
    }

    #[test]
    fn facing_rotate_f32() {
        assert_relative_eq!(Facing::North.rotate_f32_coords(0.7, 0.6).0, (0.7, 0.6).0);
        assert_relative_eq!(Facing::North.rotate_f32_coords(0.7, 0.6).1, (0.7, 0.6).1);
        assert_relative_eq!(Facing::East.rotate_f32_coords(0.7, 0.6).0, (0.4, 0.7).0);
        assert_relative_eq!(Facing::East.rotate_f32_coords(0.7, 0.6).1, (0.4, 0.7).1);
        assert_relative_eq!(Facing::South.rotate_f32_coords(0.7, 0.6).0, (0.3, 0.4).0);
        assert_relative_eq!(Facing::South.rotate_f32_coords(0.7, 0.6).1, (0.3, 0.4).1);
        assert_relative_eq!(Facing::West.rotate_f32_coords(0.7, 0.6).0, (0.6, 0.3).0);
        assert_relative_eq!(Facing::West.rotate_f32_coords(0.7, 0.6).1, (0.6, 0.3).1);

        assert_relative_eq!(Facing::North.rotate_f32_coords(2.5, 1.5).0, (2.5, 1.5).0);
        assert_relative_eq!(Facing::North.rotate_f32_coords(2.5, 1.5).1, (2.5, 1.5).1);
        assert_relative_eq!(Facing::East.rotate_f32_coords(2.5, 1.5).0, (-0.5, 2.5).0);
        assert_relative_eq!(Facing::East.rotate_f32_coords(2.5, 1.5).1, (-0.5, 2.5).1);
        assert_relative_eq!(Facing::South.rotate_f32_coords(2.5, 1.5).0, (-1.5, -0.5).0);
        assert_relative_eq!(Facing::South.rotate_f32_coords(2.5, 1.5).1, (-1.5, -0.5).1);
        assert_relative_eq!(Facing::West.rotate_f32_coords(2.5, 1.5).0, (1.5, -1.5).0);
        assert_relative_eq!(Facing::West.rotate_f32_coords(2.5, 1.5).1, (1.5, -1.5).1);
    }
}
