use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    creature::{Damage, Health},
    enemy::Enemy,
    items::EntitiesResource,
    particles::spawn_splash_particles,
    player::Player,
};

#[derive(Clone, Debug, Default, Bundle)]
pub struct ProjectileBundle {
    pub projectile: Projectile,
    pub ccd: Ccd,
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderMassProperties,
    pub damage: Damage,
}

#[derive(Component, Default, Debug, Clone)]
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
    if projectiles_amount.queue.len() > MAX_PROJECTILES {
        if let Some(entity) = projectiles_amount.queue.pop_back() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn projectiles_collisions(
    rapier_context: Res<RapierContext>,
    mut commands: Commands,
    projectiles: Query<(Entity, &Damage), With<Projectile>>,
    mut enemies: Query<(Entity, &mut Health, &Transform), With<Enemy>>,
) {
    for (projectile, Damage(damage)) in &projectiles {
        for (enemy, mut enemy_health, enemy_position) in &mut enemies {
            if let Some(contact) = rapier_context.contact_pair(projectile, enemy) {
                if let Some((_, view)) = contact.find_deepest_contact() {
                    let contact = view.local_p1();
                    let mut position = enemy_position.clone();
                    let mut impulse = ExternalImpulse::default();
                    if contact.x > 0. {
                        position.translation.x -= 10.;
                        impulse.impulse = Vec2::new(-1000.0, 0.);
                    } else {
                        position.translation.x += 10.;
                        impulse.impulse = Vec2::new(1000.0, 0.);
                    };

                    spawn_splash_particles(&mut commands, 10, position, Some(impulse));
                    enemy_health.0 -= damage;
                }
            }
        }
        if let Some(_contact) = rapier_context.contacts_with(projectile).next() {
            commands.entity(projectile).despawn();
        }
    }
}

pub fn shoot_projectile(
    input: Res<Input<MouseButton>>,
    assets: Res<EntitiesResource>,
    mut projectiles_amount: ResMut<ProjectilesGlobalAmount>,
    mut commands: Commands,
    player_query: Query<(&Transform, &TextureAtlasSprite), With<Player>>,
) {
    for (origin_transform, origin_sprite) in &player_query {
        if input.just_pressed(MouseButton::Left) {
            let is_flipped = origin_sprite.flip_x;

            let strength = 800.;
            let impulse = if is_flipped {
                Vec2::new(-strength, 0.)
            } else {
                Vec2::new(strength, 0.)
            };

            let mut origin = Vec3::from(origin_transform.translation);
            origin.x += match is_flipped {
                true => -10.,
                false => 10.,
            };
            let rotation = match is_flipped {
                true => Quat::from_rotation_z(0.7853982),
                false => Quat::from_rotation_z(-2.356194),
            };

            let fireball = assets.entities.get("FireBall").expect("fireball to exist");
            projectiles_amount.queue.push_front(
                commands
                    .spawn((
                        ProjectileBundle {
                            rigid_body: RigidBody::Dynamic,
                            collider: Collider::cuboid(5., 5.),
                            density: ColliderMassProperties::Mass(1.0),
                            gravity_scale: GravityScale(0.2),
                            ccd: Ccd::enabled(),
                            damage: Damage(20),
                            ..Default::default()
                        },
                        ExternalImpulse {
                            impulse,
                            torque_impulse: 0.0,
                        },
                        SpriteSheetBundle {
                            texture_atlas: fireball.texture_atlas.clone(),
                            sprite: TextureAtlasSprite::new(fireball.index),
                            transform: Transform {
                                translation: origin,
                                scale: Vec3::splat(0.5),
                                rotation,
                            },
                            ..Default::default()
                        },
                    ))
                    .id(),
            );
        }
    }
}
