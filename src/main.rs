mod bricks;
mod input;

use std::time::Duration;

use bevy::prelude::*;

use crate::{
    bricks::{BRICK_SIZE, BrickGrid, TetrominoData, TetrominoType, grid_to_transform},
    input::{InputFlags, LogicalInputs, Rotation, Shift, TetrisCloneInputPlugin},
};

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
            .add_systems(Startup, setup)
            .add_systems(Update, (handle_input, zoom_ctrl))
            .add_systems(FixedUpdate, active_gravity)
            .add_observer(tetromino_move_observer);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((MainCamera, Camera2d));

    spawn_tetromino(&mut commands, TetrominoType::T);
}

fn tetromino_move_observer(
    moved: On<TetrominoMoveEvent>,
    tetrominos: Query<(&GridOffset, &TetrominoBricks)>,
    mut bricks: Query<(&GridPosition, &mut Transform), (With<Active>, With<Brick>)>,
) {
    let Ok((tetromino_pos, tetromino_bricks)) = tetrominos.get(moved.0) else {
        return;
    };

    for &brick_id in &tetromino_bricks.0 {
        let Ok((pos, mut transform)) = bricks.get_mut(brick_id) else {
            continue;
        };

        transform.translation = pos.as_transform(tetromino_pos.0);
    }
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

fn spawn_tetromino(commands: &mut Commands, kind: TetrominoType) {
    let tetronimo = TetrominoData::new(kind);

    let mut tetromino_bricks = Vec::new();

    let sprite = Sprite::from_color(tetronimo.kind.as_color(), Vec2::new(BRICK_SIZE, BRICK_SIZE));
    for brick in tetronimo.bricks {
        let grid_pos = GridPosition(
            brick
        );
        let transform = grid_pos.as_transform(IVec2 { x: 0, y: 0 });
        let entity = commands.spawn((
            Active,
            Brick,
            grid_pos,
            sprite.clone(),
            Transform::from_xyz(transform.x, transform.y, transform.z),
        ));

        tetromino_bricks.push(entity.id());
    }

    commands.spawn((
        Tetromino,
        TetrominoBricks(tetromino_bricks),
        GridOffset(IVec2 { x: 0, y: 0 }),
        GravityTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
    ));
}

fn handle_input(
    inputs: Res<LogicalInputs>,
    tetrominos: Query<(Entity, &mut GridOffset, &mut GravityTimer), With<Tetromino>>,
    mut bricks: Query<&mut GridPosition, (With<Active>, With<Brick>, Without<Tetromino>)>,
    mut commands: Commands,
) {
    for (entity, mut tetromino_pos, mut gravity_timer) in tetrominos {
        let shifted = match inputs.shift() {
            Shift::Left => {
                tetromino_pos.0.x = tetromino_pos.0.x.saturating_sub(1);
                true
            },
            Shift::Right => {
                tetromino_pos.0.x = tetromino_pos.0.x + 1;
                true
            }
            Shift::None => false,
        };

        let rotated = match inputs.rotation() {
            Rotation::CW => {
                for mut brick in &mut bricks {
                    brick.rotate_cw();
                }
                true
            },
            Rotation::CCW => {
                for mut brick in &mut bricks {
                    brick.rotate_ccw();
                }
                true
            },
            Rotation::None => false,
        };

        if inputs.flags().contains(InputFlags::FastFall) {
            gravity_timer.0.set_duration(Duration::from_secs_f32(0.1));
        } else {
            gravity_timer.0.set_duration(Duration::from_secs_f32(0.5));
        }

        if shifted | rotated {
            commands.trigger(TetrominoMoveEvent(entity));
        }
    }
}

fn active_gravity(
    time: Res<Time>,
    brick_grid: Res<BrickGrid>,
    tetrominos: Query<(Entity, &mut GridOffset, &mut GravityTimer), With<Tetromino>>,
    mut commands: Commands,
) {
    for (entity, mut tetromino_pos, mut gravity_timer) in tetrominos {
        if !gravity_timer.0.tick(time.delta()).just_finished() {
            continue;
        }
        tetromino_pos.fall();

        commands.trigger(TetrominoMoveEvent(entity));
    }
}

#[derive(Component)]
struct MainCamera;

#[derive(Event)]
struct TetrominoMoveEvent(Entity);

#[derive(Component)]
struct Tetromino;

#[derive(Component)]
#[relationship(relationship_target = TetrominoBricks)]
pub struct BrickOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = BrickOf)]
pub struct TetrominoBricks(Vec<Entity>);

#[derive(Component)]
struct Brick;

#[derive(Component)]
struct Active;

#[derive(Component)]
struct GridOffset(IVec2);

impl GridOffset {
    fn fall(&mut self) {
        self.0.y += 1;
    }
}

#[derive(Component)]
struct GridPosition(UVec2);

impl GridPosition {
    fn rotate_cw(&mut self) {
        self.0 = UVec2 { x: 3 - self.0.y, y: self.0.x };
    }

    fn rotate_ccw(&mut self) {
        self.0 = UVec2 { x: self.0.y, y: 3 - self.0.x };
    }

    fn as_transform(&self, offset: IVec2) -> Vec3 {
        grid_to_transform(self.0.saturating_add_signed(offset))
    }
}

#[derive(Resource, Default)]
pub struct Scoreboard {
    time: usize,
    cleared_rows: usize,
    score: usize,
}

#[derive(Component, Default)]
pub struct GravityTimer(Timer);
