use bevy::{
    color::{Color, Hsla},
    ecs::{component::Component, resource::Resource, system::Commands},
    math::{IVec2, Vec2, Vec3},
    sprite::Sprite,
    transform::components::Transform,
};

use crate::{Brick, RngRes};

use rand::Rng;

pub const BRICK_SIZE: f32 = 32.0;
const GRID_WIDTH: usize = 10;
const GRID_HEIGHT: usize = 20;
const GRID_LINE_THICKNESS: f32 = 4.0;
const GRID_LINE_COLOR: Color = Color::hsl(0.0, 0.0, 0.03);
const GAME_OVER_CUTOFF: isize = 0;

pub fn init_grid(commands: &mut Commands) {
    for row in 0..21 {
        let mut transform = grid_to_transform(
            IVec2::new(GRID_WIDTH.div_ceil(2) as i32, row),
            IVec2::default(),
        )
        .with_z(1.0);
        transform.x -= BRICK_SIZE * 0.5;
        transform.y -= BRICK_SIZE * 0.5;
        commands.spawn((
            GridLine,
            Sprite::from_color(
                GRID_LINE_COLOR,
                Vec2::new(BRICK_SIZE * GRID_WIDTH as f32, GRID_LINE_THICKNESS),
            ),
            Transform::from_translation(transform),
        ));
    }

    for column in 0..11 {
        let mut transform = grid_to_transform(
            IVec2::new(column, GRID_HEIGHT.div_ceil(2) as i32),
            IVec2::default(),
        )
        .with_z(1.0);
        transform.x -= BRICK_SIZE * 0.5;
        transform.y -= BRICK_SIZE * 0.5;
        commands.spawn((
            GridLine,
            Sprite::from_color(
                GRID_LINE_COLOR,
                Vec2::new(GRID_LINE_THICKNESS, BRICK_SIZE * GRID_HEIGHT as f32),
            ),
            Transform::from_translation(transform),
        ));
    }
}

#[derive(Component)]
struct GridLine;

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
    data: [[Option<TetrominoType>; GRID_WIDTH]; GRID_HEIGHT],
}

impl Default for BrickGrid {
    fn default() -> Self {
        Self {
            data: std::array::from_fn(|_| std::array::from_fn(|_| None)),
        }
    }
}

impl BrickGrid {
    pub fn spawn_inactive_bricks(&self, commands: &mut Commands) {
        for (y, row) in self.data.iter().enumerate() {
            for (x, maybe_kind) in row.iter().enumerate() {
                let &Some(kind) = maybe_kind else {
                    continue;
                };
                let sprite = Sprite::from_color(kind.as_color(), Vec2::new(BRICK_SIZE, BRICK_SIZE));
                let transform = grid_to_transform(IVec2::new(x as i32, y as i32), IVec2::default());

                commands.spawn((
                    Brick,
                    sprite,
                    Transform::from_xyz(transform.x, transform.y, transform.z),
                ));
            }
        }
    }

    pub fn is_clear(
        &self,
        offset: IVec2,
        kind: TetrominoType,
        orient: TetrominoOrientation,
    ) -> bool {
        let bricks = TetrominoData::get_data(kind, orient);
        for brick in &bricks {
            if offset.x - brick.x < 0
                || offset.x - brick.x >= GRID_WIDTH as i32
                || offset.y - brick.y < 0
                || offset.y - brick.y >= GRID_HEIGHT as i32
            {
                return false;
            }

            let x = usize::try_from(offset.x - brick.x).expect("Should never be negative");
            let y = usize::try_from(offset.y - brick.y).expect("Should never be negative");

            if self.data[y][x] != None {
                return false;
            }
        }
        true
    }

    /// Places a new tetromino and returns completed rows
    /// Returns (number of completed rows)
    ///
    /// **PLEASE MAKE IT MORE EFFICIENT WITH CHECKING IS_ROW_CLEARED()**
    pub fn place(
        &mut self,
        offset: IVec2,
        kind: TetrominoType,
        orient: TetrominoOrientation,
    ) -> usize {
        let bricks = TetrominoData::get_data(kind, orient);
        let mut rows = 0;
        let mut cleared_rows = [usize::MAX; 5];

        for brick in bricks {
            let x = usize::try_from(offset.x - brick.x)
                .expect("Should never place a brick in a negative column");
            let y = usize::try_from(offset.y - brick.y)
                .expect("Should never place a brick in a negative row");
            self.data[y][x] = Some(kind);

            if self.is_row_cleared(y) {
                cleared_rows[rows] = y;
                rows += 1;
            }
        }

        let mut target = 0;
        let mut row_offset = 0;
        for y in 0..GRID_HEIGHT - rows {
            if y == cleared_rows[row_offset] {
                row_offset += 1;
                continue;
            }
            self.data[target] = self.data[y];
            target += 1;
        }
        for y in target..GRID_HEIGHT {
            self.data[y].fill(None);
        }

        rows
    }

    fn is_row_cleared(&self, y: usize) -> bool {
        self.data[y].iter().all(|&brick_type| brick_type.is_some())
    }
}

pub fn grid_to_transform(pos: IVec2, offset: IVec2) -> Vec3 {
    Vec3::new(
        ((pos.x - offset.x) as f32 - GRID_WIDTH as f32 * 0.5 + 0.5) * BRICK_SIZE,
        ((pos.y - offset.y) as f32 - GRID_HEIGHT as f32 * 0.5 + 0.5) * BRICK_SIZE,
        0.0,
    )
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum TetrominoType {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

impl TetrominoType {
    pub fn as_color(self) -> Color {
        match self {
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

#[derive(Clone, Copy, Default)]
#[repr(u8)]
pub enum TetrominoOrientation {
    #[default]
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}

impl TetrominoOrientation {
    pub fn rot_cw(self) -> Self {
        match self {
            Self::Zero => Self::Ninety,
            Self::Ninety => Self::OneEighty,
            Self::OneEighty => Self::TwoSeventy,
            Self::TwoSeventy => Self::Zero,
        }
    }

    pub fn rot_ccw(self) -> Self {
        match self {
            Self::Zero => Self::TwoSeventy,
            Self::Ninety => Self::Zero,
            Self::OneEighty => Self::Ninety,
            Self::TwoSeventy => Self::OneEighty,
        }
    }
}

pub struct TetrominoData {
    pub offset: IVec2,
    pub kind: TetrominoType,
    pub orient: TetrominoOrientation,
}

impl TetrominoData {
    pub fn new(rng: &mut RngRes) -> Self {
        // By using variant_count in the range, it is safe to assume that the generated
        // value will never be out of range
        let kind = unsafe {
            std::mem::transmute(
                rng.gener
                    .random_range(0..std::mem::variant_count::<TetrominoType>() as u8),
            )
        };

        Self {
            offset: IVec2::new(5, GRID_HEIGHT as i32),
            orient: TetrominoOrientation::default(),
            kind,
        }
    }

    pub fn get_data(kind: TetrominoType, orientation: TetrominoOrientation) -> [IVec2; 4] {
        TETROMINO_OFFSETS[kind as usize][orientation as usize]
    }
}

const TETROMINO_OFFSETS: [[[IVec2; 4]; 4]; std::mem::variant_count::<TetrominoType>()] = [
    [
        // I
        [
            IVec2::new(0, 1),
            IVec2::new(1, 1),
            IVec2::new(2, 1),
            IVec2::new(3, 1),
        ],
        [
            IVec2::new(1, 3),
            IVec2::new(1, 2),
            IVec2::new(1, 1),
            IVec2::new(1, 0),
        ],
        [
            IVec2::new(0, 1),
            IVec2::new(1, 1),
            IVec2::new(2, 1),
            IVec2::new(3, 1),
        ],
        [
            IVec2::new(1, 3),
            IVec2::new(1, 2),
            IVec2::new(1, 1),
            IVec2::new(1, 0),
        ],
    ],
    [
        // J
        [
            IVec2::new(2, 2),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
            IVec2::new(2, 1),
        ],
        [
            IVec2::new(1, 2),
            IVec2::new(1, 1),
            IVec2::new(1, 0),
            IVec2::new(2, 0),
        ],
        [
            IVec2::new(0, 1),
            IVec2::new(1, 1),
            IVec2::new(2, 1),
            IVec2::new(0, 0),
        ],
        [
            IVec2::new(1, 2),
            IVec2::new(0, 2),
            IVec2::new(1, 1),
            IVec2::new(1, 0),
        ],
    ],
    [
        // L
        [
            IVec2::new(0, 2),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
            IVec2::new(2, 1),
        ],
        [
            IVec2::new(1, 2),
            IVec2::new(2, 2),
            IVec2::new(1, 1),
            IVec2::new(1, 0),
        ],
        [
            IVec2::new(2, 0),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
            IVec2::new(2, 1),
        ],
        [
            IVec2::new(1, 2),
            IVec2::new(1, 1),
            IVec2::new(0, 0),
            IVec2::new(1, 0),
        ],
    ],
    [
        // O
        [
            IVec2::new(0, 2),
            IVec2::new(1, 2),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
        ],
        [
            IVec2::new(0, 2),
            IVec2::new(1, 2),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
        ],
        [
            IVec2::new(0, 2),
            IVec2::new(1, 2),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
        ],
        [
            IVec2::new(0, 2),
            IVec2::new(1, 2),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
        ],
    ],
    [
        // S
        [
            IVec2::new(0, 2),
            IVec2::new(1, 2),
            IVec2::new(1, 1),
            IVec2::new(2, 1),
        ],
        [
            IVec2::new(1, 2),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
            IVec2::new(0, 0),
        ],
        [
            IVec2::new(0, 2),
            IVec2::new(1, 2),
            IVec2::new(1, 1),
            IVec2::new(2, 1),
        ],
        [
            IVec2::new(1, 2),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
            IVec2::new(0, 0),
        ],
    ],
    [
        // T
        [
            IVec2::new(1, 2),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
            IVec2::new(2, 1),
        ],
        [
            IVec2::new(1, 2),
            IVec2::new(1, 1),
            IVec2::new(2, 1),
            IVec2::new(1, 0),
        ],
        [
            IVec2::new(0, 1),
            IVec2::new(1, 1),
            IVec2::new(2, 1),
            IVec2::new(1, 0),
        ],
        [
            IVec2::new(1, 2),
            IVec2::new(1, 1),
            IVec2::new(0, 1),
            IVec2::new(1, 0),
        ],
    ],
    [
        // Z
        [
            IVec2::new(1, 2),
            IVec2::new(2, 2),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
        ],
        [
            IVec2::new(0, 2),
            IVec2::new(1, 1),
            IVec2::new(0, 1),
            IVec2::new(1, 0),
        ],
        [
            IVec2::new(1, 2),
            IVec2::new(2, 2),
            IVec2::new(0, 1),
            IVec2::new(1, 1),
        ],
        [
            IVec2::new(0, 2),
            IVec2::new(1, 1),
            IVec2::new(0, 1),
            IVec2::new(1, 0),
        ],
    ],
];
