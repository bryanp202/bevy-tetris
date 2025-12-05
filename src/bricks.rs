use bevy::{
    color::{Color, Hsla, LinearRgba},
    ecs::resource::Resource,
    math::{IVec2, UVec2, Vec3},
};

pub const BRICK_SIZE: f32 = 16.0;
const GRID_WIDTH: usize = 32;
const GRID_HEIGHT: usize = 64;
const GAME_OVER_CUTOFF: usize = 3;
#[derive(Resource)]
/// Collision and color data of the tetris bricks
///
/// _____________________
/// |(0, 0)|(0, 1)|(0, 2)|
/// |(1, 0)|(1, 1)|(1, 2)|
/// |(2, 0)|(2, 1)|(2, 2)|
/// | ...  | ...  | ...  |
/// _____________________
pub struct BrickGrid {
    data: [[TetrominoType; GRID_WIDTH]; GRID_HEIGHT],
}

impl Default for BrickGrid {
    fn default() -> Self {
        Self {
            data: std::array::from_fn(|_| std::array::from_fn(|_| TetrominoType::default())),
        }
    }
}

impl BrickGrid {
    /// Places a new tetromino and returns completed rows
    /// Returns a brick is out of bands and the number of completed rows
    ///
    /// **PLEASE MAKE IT MORE EFFICIENT WITH CHECKING IS_ROW_CLEARED()**
    pub fn place(&mut self, tetromino: TetrominoData) -> (bool, usize) {
        let mut rows = 0;
        let mut cleared_rows = [usize::MAX; 4];

        for UVec2 { x, y } in tetromino.bricks {
            let x = x as usize;
            let y = y as usize;
            self.data[y][x] = tetromino.kind;

            if self.is_row_cleared(y as usize) {
                cleared_rows[rows] = y as usize;
                rows += 1;
            }
        }

        let mut row_offset = 0;
        for y in rows..GRID_HEIGHT {
            if y == cleared_rows[row_offset] {
                row_offset += 1;
            } else {
                for x in 0..GRID_WIDTH {
                    self.data[y][x] = self.data[y - row_offset][x];
                }
            }
        }
        for y in 0..rows {
            for x in 0..GRID_WIDTH {
                self.data[y][x] = TetrominoType::Empty;
            }
        }

        let game_over = tetromino
            .bricks
            .iter()
            .map(|&UVec2 { x: _, y }| y as usize)
            .all(|y| y + rows > GAME_OVER_CUTOFF);

        (game_over, rows)
    }

    fn is_row_cleared(&self, y: usize) -> bool {
        self.data[y]
            .iter()
            .all(|&brick_type| brick_type != TetrominoType::Empty)
    }
}

pub fn grid_to_transform(pos: UVec2) -> Vec3 {
    Vec3::new(
        pos.x as f32 * BRICK_SIZE - GRID_WIDTH as f32 * BRICK_SIZE / 2.0,
        pos.y as f32 * -BRICK_SIZE + GRID_WIDTH as f32 * BRICK_SIZE / 2.0,
        0.0,
    )
}

#[derive(Clone, Copy, Default, PartialEq)]
pub enum TetrominoType {
    #[default]
    Empty,
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl TetrominoType {
    pub fn as_color(&self) -> Color {
        match self {
            Self::Empty => Color::LinearRgba(LinearRgba::new(0.0, 0.0, 0.0, 0.0)),
            Self::I => Color::Hsla(Hsla::hsl(166.0, 0.84, 0.68)),
            Self::J => Color::Hsla(Hsla::hsl(191.0, 1.0, 0.19)),
            Self::L => Color::Hsla(Hsla::hsl(26.0, 0.93, 0.78)),
            Self::O => Color::Hsla(Hsla::hsl(56.0, 1.0, 0.5)),
            Self::S => Color::Hsla(Hsla::hsl(355.0, 0.5, 0.54)),
            Self::T => Color::Hsla(Hsla::hsl(272.0, 0.61, 0.34)),
            Self::Z => Color::Hsla(Hsla::hsl(145.0, 0.63, 0.49)),
        }
    }
}

pub struct TetrominoData {
    offset: IVec2,
    pub bricks: [UVec2; 4],
    pub kind: TetrominoType,
}

impl TetrominoData {
    pub fn new(kind: TetrominoType) -> Self {
        Self {
            offset: IVec2 { x: 0, y: 0 },
            bricks: match kind {
                TetrominoType::Empty => panic!("Should never call tetromino new with empty type"),
                TetrominoType::I => [
                    UVec2 { x: 2, y: 0 },
                    UVec2 { x: 2, y: 1 },
                    UVec2 { x: 2, y: 2 },
                    UVec2 { x: 2, y: 3 },
                ],
                TetrominoType::J => [
                    UVec2 { x: 2, y: 0 },
                    UVec2 { x: 2, y: 1 },
                    UVec2 { x: 2, y: 2 },
                    UVec2 { x: 1, y: 2 },
                ],
                TetrominoType::L => [
                    UVec2 { x: 1, y: 0 },
                    UVec2 { x: 1, y: 1 },
                    UVec2 { x: 1, y: 2 },
                    UVec2 { x: 2, y: 2 },
                ],
                TetrominoType::O => [
                    UVec2 { x: 1, y: 1 },
                    UVec2 { x: 1, y: 2 },
                    UVec2 { x: 2, y: 1 },
                    UVec2 { x: 2, y: 2 },
                ],
                TetrominoType::S => [
                    UVec2 { x: 1, y: 2 },
                    UVec2 { x: 2, y: 2 },
                    UVec2 { x: 2, y: 1 },
                    UVec2 { x: 3, y: 1 },
                ],
                TetrominoType::Z => [
                    UVec2 { x: 1, y: 1 },
                    UVec2 { x: 2, y: 1 },
                    UVec2 { x: 2, y: 2 },
                    UVec2 { x: 3, y: 2 },
                ],
                TetrominoType::T => [
                    UVec2 { x: 1, y: 2 },
                    UVec2 { x: 2, y: 2 },
                    UVec2 { x: 2, y: 1 },
                    UVec2 { x: 3, y: 2 },
                ],
            },
            kind,
        }
    }
}
