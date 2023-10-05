use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    collisions::ColliderBundle,
    player::{AnimationBundle, MoveDirection},
};

#[derive(Component)]
pub struct Health(pub u32);

#[derive(Component, Default)]
pub struct Damage(pub i32);

impl Default for Health {
    fn default() -> Self {
        Self(100)
    }
}

#[derive(Bundle, Default)]
pub struct CreatureBundle {
    pub health: Health,
    pub damage: Damage,
    pub sprite: SpriteSheetBundle,
    pub move_direction: MoveDirection,
    pub character_controller: KinematicCharacterController,
    pub character_output: KinematicCharacterControllerOutput,
    pub collider_bundle: ColliderBundle,
    pub animation_bundle: AnimationBundle,
}
