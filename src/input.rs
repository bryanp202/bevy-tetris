use bevy::prelude::*;
use bitflags::bitflags;

pub struct TetrisCloneInputPlugin;

impl Plugin for TetrisCloneInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LogicalInputs::default())
            .insert_resource(KeyBindings::default())
            .add_systems(PreUpdate, parse_inputs);
    }
}

#[derive(Resource)]
struct KeyBindings {
    shift_left: KeyCode,
    shift_right: KeyCode,
    rot_cw: KeyCode,
    rot_ccw: KeyCode,
    drop: KeyCode,
    fast_fall: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            shift_left: KeyCode::KeyA,
            shift_right: KeyCode::KeyD,
            rot_cw: KeyCode::KeyE,
            rot_ccw: KeyCode::KeyQ,
            drop: KeyCode::Space,
            fast_fall: KeyCode::KeyS,
        }
    }
}

#[derive(Resource, Default)]
pub struct LogicalInputs {
    shift: Shift,
    rotation: Rotation,
    flags: InputFlags,
}

impl LogicalInputs {
    pub fn shift(&self) -> Shift {
        self.shift
    }

    pub fn rotation(&self) -> Rotation {
        self.rotation
    }

    pub fn flags(&self) -> InputFlags {
        self.flags
    }
}

bitflags! {
    #[derive(Clone, Copy, Default)]
    pub struct InputFlags: u8 {
        const None =        0b0000_0000;
        const Drop =        0b0000_0001;
        const FastFall =    0b0000_0010;
    }
}

#[derive(Clone, Copy, Default)]
pub enum Shift {
    #[default]
    None,
    Left,
    Right,
}

#[derive(Clone, Copy, Default)]
pub enum Rotation {
    #[default]
    None,
    CW,
    CCW,
}

fn parse_inputs(
    keyboard: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    mut inputs: ResMut<LogicalInputs>,
) {
    let shift_left = keyboard.just_pressed(key_bindings.shift_left);
    let shift_right = keyboard.just_pressed(key_bindings.shift_right);
    let shift = match (shift_left, shift_right) {
        (true, true) | (false, false) => Shift::None,
        (true, false) => Shift::Left,
        (false, true) => Shift::Right,
    };

    let rot_cw = keyboard.just_pressed(key_bindings.rot_cw);
    let rot_ccw = keyboard.just_pressed(key_bindings.rot_ccw);
    let rot = match (rot_cw, rot_ccw) {
        (true, true) | (false, false) => Rotation::None,
        (true, false) => Rotation::CW,
        (false, true) => Rotation::CCW,
    };

    let drop = if keyboard.just_pressed(key_bindings.drop) {
        InputFlags::Drop
    } else {
        InputFlags::None
    };

    let fast_fall = if keyboard.pressed(key_bindings.fast_fall) {
        InputFlags::FastFall
    } else {
        InputFlags::None
    };

    inputs.shift = shift;
    inputs.rotation = rot;
    inputs.flags = drop | fast_fall;
}
