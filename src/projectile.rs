use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::player::{Player, PlayerAtlases};

#[derive(Clone, Debug, Default, Bundle)]
pub struct ProjectileBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderMassProperties,
}

#[derive(Component)]
pub struct Projectile;

#[derive(Resource, Default)]
pub struct ProjectilesGlobalAmount {
    pub queue: VecDeque<Entity>,
}

const MAX_PROJECTILES: usize = 1000;

pub fn despawn_projectiles(
    mut commands: Commands,
    mut projectiles_amount: ResMut<ProjectilesGlobalAmount>,
) {
    if projectiles_amount.queue.len() > MAX_PROJECTILES && projectiles_amount.queue.len() > 0 {
        if let Some(entity) = projectiles_amount.queue.pop_back() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn shoot_projectile(
    input: Res<Input<MouseButton>>,
    assets: Res<PlayerAtlases>,
    mut projectiles_amount: ResMut<ProjectilesGlobalAmount>,
    mut commands: Commands,
    player_query: Query<(&Transform, &TextureAtlasSprite), With<Player>>,
) {
    for (origin_transform, origin_sprite) in &player_query {
        if input.just_pressed(MouseButton::Left) {
            let is_flipped = origin_sprite.flip_x;
            let impulse = if is_flipped {
                Vec2::new(-1000., 0.)
            } else {
                Vec2::new(1000., 0.)
            };
            let mut origin = Vec3::from(origin_transform.translation);
            origin.x += match is_flipped {
                true => -0.5,
                false => 0.5,
            };
            projectiles_amount.queue.push_front(
                commands
                    .spawn((
                        ProjectileBundle {
                            rigid_body: RigidBody::Dynamic,
                            collider: Collider::cuboid(5., 5.),
                            density: ColliderMassProperties::Mass(1.0),
                            ..Default::default()
                        },
                        ExternalImpulse {
                            impulse,
                            torque_impulse: 0.0,
                        },
                        Projectile,
                        SpriteSheetBundle {
                            texture_atlas: assets.run.clone(),
                            transform: Transform {
                                translation: origin,
                                scale: Vec3::splat(1.4),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    ))
                    .id(),
            );
        }
    }
}
