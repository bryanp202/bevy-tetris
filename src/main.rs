#![feature(variant_count)]

mod bricks;
mod input;

use std::time::Duration;

use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{
    bricks::{
        BRICK_SIZE, BrickGrid, TetrominoData, TetrominoOrientation, TetrominoType,
        grid_to_transform,
    },
    input::{InputFlags, LogicalInputs, Rotation, Shift, TetrisCloneInputPlugin},
};

const SCORE_COLOR: Color = Color::WHITE;
const SCOREBOARD_FONT_SIZE: f32 = 32.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TetrisClonePlugin)
        .add_plugins(TetrisCloneInputPlugin)
        .run();
}

pub struct TetrisClonePlugin;

impl Plugin for TetrisClonePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Scoreboard::default())
            .insert_resource(BrickGrid::default())
            .insert_resource(RngRes::default())
            .add_systems(Startup, setup)
            .add_systems(Update, (handle_input, zoom_ctrl))
            .add_systems(FixedUpdate, active_gravity)
            .add_observer(tetromino_move_observer)
            .add_observer(tetromino_place_observer)
            .add_observer(score_observer);
    }
}

fn setup(mut rng: ResMut<RngRes>, mut commands: Commands) {
    commands.spawn((MainCamera, Camera2d));
    spawn_tetromino(&mut rng, &mut commands);
    make_grid(&mut commands);

    commands.spawn((
        Text::new("Score: "),
        TextFont {
            font_size: SCOREBOARD_FONT_SIZE,
            ..default()
        },
        ScoreUi,
        TextColor(SCORE_COLOR),
        Node {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        },
        children![(
            TextSpan::new("0"),
            TextFont {
                font_size: SCOREBOARD_FONT_SIZE,
                ..default()
            },
            TextColor(SCORE_COLOR),
        )],
    ));

    commands.spawn((
        Text::new("Cleared Rows: "),
        TextFont {
            font_size: SCOREBOARD_FONT_SIZE,
            ..default()
        },
        ClearedRowsUi,
        TextColor(SCORE_COLOR),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(SCOREBOARD_FONT_SIZE * 3.0),
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        },
        children![(
            TextSpan::new("0"),
            TextFont {
                font_size: SCOREBOARD_FONT_SIZE,
                ..default()
            },
            TextColor(SCORE_COLOR),
        )],
    ));
}

fn make_grid(commands: &mut Commands) {
    for row in 0..21 {
        let mut transform = grid_to_transform(IVec2::new(5, row), IVec2::default()).with_z(1.0);
        transform.x -= BRICK_SIZE * 0.5;
        transform.y -= BRICK_SIZE * 0.5;
        commands.spawn((
            GridLine,
            Sprite::from_color(Color::BLACK, Vec2::new(BRICK_SIZE * 10.0, 1.0)),
            Transform::from_translation(transform),
        ));
    }

    for column in 0..11 {
        let mut transform = grid_to_transform(IVec2::new(column, 10), IVec2::default()).with_z(1.0);
        transform.x -= BRICK_SIZE * 0.5;
        transform.y -= BRICK_SIZE * 0.5;
        commands.spawn((
            GridLine,
            Sprite::from_color(Color::BLACK, Vec2::new(1.0, BRICK_SIZE * 20.0)),
            Transform::from_translation(transform),
        ));
    }
}

#[derive(Component)]
struct GridLine;

fn tetromino_move_observer(
    moved: On<TetrominoMoveEvent>,
    tetrominos: Query<(&Tetromino, &Children)>,
    mut bricks: Query<&mut Transform, (With<Active>, With<Brick>)>,
) {
    let Ok((tetromino, child_bricks)) = tetrominos.get(moved.0) else {
        return;
    };

    for (brick_id, &brick_offset) in child_bricks.iter().zip(tetromino.get_brick_data().iter()) {
        let Ok(mut transform) = bricks.get_mut(brick_id) else {
            continue;
        };

        transform.translation = grid_to_transform(tetromino.offset, brick_offset);
    }
}

fn tetromino_place_observer(
    placed: On<TetrominoPlaceEvent>,
    tetrominos: Query<(Entity, &Tetromino)>,
    inactive_bricks: Query<Entity, (With<Brick>, Without<Active>)>,
    mut brick_grid: ResMut<BrickGrid>,
    mut rng: ResMut<RngRes>,
    mut commands: Commands,
) {
    let Ok((tetromino_entity, tetromino)) = tetrominos.get(placed.0) else {
        return;
    };

    let cleared_rows = brick_grid.place(tetromino.offset, tetromino.kind, tetromino.orient);

    if cleared_rows > 0 {
        commands.trigger(RowClearEvent(cleared_rows));
    }

    commands.entity(tetromino_entity).despawn();
    for inactive_brick_entity in &inactive_bricks {
        commands.entity(inactive_brick_entity).despawn();
    }

    brick_grid.spawn_inactive_bricks(&mut commands);

    spawn_tetromino(&mut rng, &mut commands);
}

fn score_observer(
    cleared_rows: On<RowClearEvent>,
    mut score: ResMut<Scoreboard>,
    score_root: Single<Entity, (With<ScoreUi>, With<Text>)>,
    rows_root: Single<Entity, (With<ClearedRowsUi>, With<Text>)>,
    mut writer: TextUiWriter,
) {
    score.cleared_rows += cleared_rows.0;
    score.score += match cleared_rows.0 {
        1 => 100,
        2 => 300,
        3 => 500,
        4 => 800,
        _ => 0,
    };

    *writer.text(*score_root, 1) = score.score.to_string();
    *writer.text(*rows_root, 1) = score.cleared_rows.to_string();
}

fn zoom_ctrl(
    keyboard: Res<ButtonInput<KeyCode>>,
    camera: Single<&mut Projection, With<MainCamera>>,
) {
    let mut proj = camera.into_inner();

    if let Projection::Orthographic(ref mut project2d) = *proj {
        if keyboard.just_pressed(KeyCode::ArrowUp) {
            project2d.scale *= 1.0 / 1.1;
        }
        if keyboard.just_pressed(KeyCode::ArrowDown) {
            project2d.scale *= 1.1;
        }
    }
}

fn spawn_tetromino(rng: &mut RngRes, commands: &mut Commands) {
    let tetromino = TetrominoData::new(rng);
    let sprite = Sprite::from_color(tetromino.kind.as_color(), Vec2::new(BRICK_SIZE, BRICK_SIZE));

    commands
        .spawn((
            Tetromino {
                offset: tetromino.offset,
                kind: tetromino.kind,
                orient: tetromino.orient,
            },
            GravityTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
            Visibility::Visible,
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .with_children(|p| {
            for brick in TetrominoData::get_data(tetromino.kind, tetromino.orient) {
                let transform = grid_to_transform(tetromino.offset - brick, IVec2::default());
                p.spawn((
                    Active,
                    Brick,
                    sprite.clone(),
                    Transform::from_xyz(transform.x, transform.y, transform.z),
                ));
            }
        });
}

fn handle_input(
    inputs: Res<LogicalInputs>,
    brick_grid: Res<BrickGrid>,
    tetrominos: Query<(Entity, &mut Tetromino, &mut GravityTimer)>,
    mut commands: Commands,
) {
    for (entity, mut tetromino, mut gravity_timer) in tetrominos {
        let new_offset = match inputs.shift() {
            Shift::None => None,
            Shift::Left => Some(tetromino.shift_left_offset()),
            Shift::Right => Some(tetromino.shift_right_offset()),
        };

        let new_orient = match inputs.rotation() {
            Rotation::None => None,
            Rotation::CW => Some(tetromino.rot_cw()),
            Rotation::CCW => Some(tetromino.rot_ccw()),
        };

        if inputs.flags().contains(InputFlags::FastFall) {
            gravity_timer
                .0
                .set_duration(Duration::from_secs_f32(0.0625));
        } else {
            gravity_timer.0.set_duration(Duration::from_secs_f32(0.5));
        }

        let moved = tetromino.check_move(&brick_grid, new_offset, new_orient);

        if inputs.flags().contains(InputFlags::Drop) {
            let mut drop_offset = Some(tetromino.fall_offset());
            while tetromino.check_move(&brick_grid, drop_offset, new_orient) {
                drop_offset = Some(tetromino.fall_offset());
            }
            commands.trigger(TetrominoPlaceEvent(entity));
        } else if moved {
            commands.trigger(TetrominoMoveEvent(entity));
        }
    }
}

fn active_gravity(
    time: Res<Time>,
    brick_grid: Res<BrickGrid>,
    tetrominos: Query<(Entity, &mut Tetromino, &mut GravityTimer)>,
    mut commands: Commands,
) {
    for (entity, mut tetromino, mut gravity_timer) in tetrominos {
        if !gravity_timer.0.tick(time.delta()).just_finished() {
            continue;
        }

        let new_offset = Some(tetromino.fall_offset());
        let orient = None;
        if tetromino.check_move(&brick_grid, new_offset, orient) {
            commands.trigger(TetrominoMoveEvent(entity));
        } else {
            commands.trigger(TetrominoPlaceEvent(entity));
        }
    }
}

#[derive(Component)]
struct MainCamera;

#[derive(Event)]
struct TetrominoMoveEvent(Entity);

#[derive(Event)]
struct TetrominoPlaceEvent(Entity);

#[derive(Event)]
struct RowClearEvent(usize);

#[derive(Component)]
struct Tetromino {
    offset: IVec2,
    kind: TetrominoType,
    orient: TetrominoOrientation,
}

impl Tetromino {
    pub fn check_move(
        &mut self,
        brick_grid: &BrickGrid,
        new_offset: Option<IVec2>,
        new_orient: Option<TetrominoOrientation>,
    ) -> bool {
        if new_offset.is_none() && new_orient.is_none() {
            return false;
        }

        let new_offset = new_offset.unwrap_or(self.offset);
        let new_orient = new_orient.unwrap_or(self.orient);

        if brick_grid.is_clear(new_offset, self.kind, new_orient) {
            self.offset = new_offset;
            self.orient = new_orient;
            true
        } else {
            false
        }
    }

    fn get_brick_data(&self) -> [IVec2; 4] {
        TetrominoData::get_data(self.kind, self.orient)
    }

    fn fall_offset(&mut self) -> IVec2 {
        self.offset.with_y(self.offset.y - 1)
    }

    fn shift_left_offset(&mut self) -> IVec2 {
        self.offset.with_x(self.offset.x - 1)
    }

    fn shift_right_offset(&mut self) -> IVec2 {
        self.offset.with_x(self.offset.x + 1)
    }

    fn rot_cw(&mut self) -> TetrominoOrientation {
        self.orient.rot_cw()
    }

    fn rot_ccw(&mut self) -> TetrominoOrientation {
        self.orient.rot_ccw()
    }
}

#[derive(Component)]
struct Brick;

#[derive(Component)]
struct Active;

#[derive(Resource, Default)]
pub struct Scoreboard {
    time: usize,
    cleared_rows: usize,
    score: usize,
}

#[derive(Resource)]
pub struct RngRes {
    pub gener: ChaCha8Rng,
}

impl Default for RngRes {
    fn default() -> Self {
        Self {
            gener: ChaCha8Rng::seed_from_u64(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|dur| dur.as_secs())
                    .unwrap_or(1126541),
            ),
        }
    }
}

#[derive(Component, Default)]
pub struct GravityTimer(Timer);

#[derive(Component)]
pub struct ClearedRowsUi;

#[derive(Component)]
pub struct ScoreUi;
