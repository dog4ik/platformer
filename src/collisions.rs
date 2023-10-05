use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    creature::{Damage, Health},
    enemy::Enemy,
    ladder::{Climbable, Climber},
    player::Player,
};

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderMassProperties,
}

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct SensorBundle {
    pub collider: Collider,
    pub sensor: Sensor,
    pub active_events: ActiveEvents,
    pub rotation_constraints: LockedAxes,
}

impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> ColliderBundle {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;

        match entity_instance.identifier.as_str() {
            "Player" => ColliderBundle {
                collider: Collider::cuboid(16., 16.),
                rigid_body: RigidBody::Dynamic,
                friction: Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                rotation_constraints,
                ..Default::default()
            },
            "Mob" => ColliderBundle {
                collider: Collider::cuboid(10., 10.),
                rigid_body: RigidBody::KinematicVelocityBased,
                rotation_constraints,
                ..Default::default()
            },
            "Chest" => ColliderBundle {
                collider: Collider::cuboid(8., 8.),
                rigid_body: RigidBody::Dynamic,
                rotation_constraints,
                gravity_scale: GravityScale(1.0),
                friction: Friction::new(0.5),
                density: ColliderMassProperties::Density(15.0),
                ..Default::default()
            },
            _ => ColliderBundle::default(),
        }
    }
}

impl From<IntGridCell> for SensorBundle {
    fn from(int_grid_cell: IntGridCell) -> Self {
        // ladder
        if int_grid_cell.value == 2 {
            SensorBundle {
                collider: Collider::cuboid(8., 8.),
                sensor: Sensor,
                active_events: ActiveEvents::CONTACT_FORCE_EVENTS,
                rotation_constraints: LockedAxes::ROTATION_LOCKED,
                ..Default::default()
            }
        } else {
            SensorBundle::default()
        }
    }
}

pub fn detect_climb_range(
    mut player: Query<(Entity, &mut Climber), With<Player>>,
    climbables: Query<Entity, With<Climbable>>,
    rapier_context: Res<RapierContext>,
) {
    for (player, mut climber) in &mut player {
        for climbable in &climbables {
            let is_intersecting = rapier_context.intersection_pair(player, climbable);
            if is_intersecting.is_some() {
                climber.climbing = true;
                climber.intersecting_climbables.insert(climbable);
            } else {
                climber.intersecting_climbables.remove(&climbable);
                if climber.intersecting_climbables.is_empty() {
                    climber.climbing = false;
                }
            }
        }
    }
}

pub fn detect_player_damage(
    mut player: Query<(&mut Health, &Transform, &Collider), With<Player>>,
    enemies: Query<(&Transform, &Collider, &Damage), With<Enemy>>,
) {
    if let Ok((mut health, player_transform, player_collider)) = player.get_single_mut() {
        let player_collider = player_collider.as_cuboid().unwrap();
        let player_dimentions = player_collider.half_extents() * Vec2::splat(2.5);
        for (enemy_transform, enemy_collider, enemy_damage) in &enemies {
            let enemy_collider = enemy_collider.as_cuboid().unwrap();
            let enemy_dimentions = enemy_collider.half_extents() * Vec2::splat(2.5);

            let collision = collide(
                player_transform.translation,
                player_dimentions,
                enemy_transform.translation,
                enemy_dimentions,
            );
            if let Some(collision) = collision {
                match collision {
                    Collision::Top => (),
                    _ => health.0 -= enemy_damage.0 as u32,
                };
            }
        }
    }
}

pub fn ignore_gravity_if_climbing(
    mut query: Query<(&Climber, &mut GravityScale), Changed<Climber>>,
) {
    for (climber, mut gravity_scale) in &mut query {
        if climber.climbing {
            gravity_scale.0 = 0.0;
        } else {
            gravity_scale.0 = 1.0;
        }
    }
}
