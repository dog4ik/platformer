use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::collisions::{ColliderBundle, GameCollisions};

#[derive(Component, Default)]
pub struct ParticleGroup;

#[derive(Component, Default)]
pub struct Paticle;

#[derive(Bundle)]
pub struct ParticleBundle {
    pub collider: ColliderBundle,
    pub sprite: SpriteBundle,
    pub impulse: ExternalImpulse,
    pub paticle: Paticle,
}

pub fn spawn_splash_particles(
    commands: &mut Commands,
    amount: usize,
    mut position: Transform,
    impusle: Option<ExternalImpulse>,
) {
    position.scale = Vec3::splat(1.);
    let impulse = impusle.unwrap_or_default();
    commands
        .spawn((
            ParticleGroup,
            TransformBundle {
                local: position.into(),
                ..Default::default()
            },
            VisibilityBundle::default(),
        ))
        .with_children(|parent| {
            let mut generator = rand::thread_rng();
            for _ in 0..amount {
                let mut local_position = Transform::from_xyz(0., 0., 0.);
                let rand_x = generator.gen_range(-10.0..10.);
                let rand_y = generator.gen_range(-10.0..10.);
                local_position.translation.y += rand_y;
                local_position.translation.x += rand_x;
                parent.spawn(ParticleBundle {
                    collider: ColliderBundle {
                        collider: Collider::ball(1.),
                        rigid_body: RigidBody::Dynamic,
                        collision_groups: GameCollisions::Environment.into(),
                        ..Default::default()
                    },
                    impulse,
                    paticle: Paticle,
                    sprite: SpriteBundle {
                        sprite: Sprite {
                            color: Color::RED,
                            custom_size: Some(Vec2::new(1.5, 1.5)),
                            ..default()
                        },
                        transform: local_position,
                        ..default()
                    },
                });
            }
        });
}

impl Default for ParticleBundle {
    fn default() -> Self {
        Self {
            collider: ColliderBundle {
                collider: Collider::round_cuboid(1., 1., 0.5),
                rigid_body: RigidBody::Dynamic,
                gravity_scale: GravityScale(0.5),
                ..Default::default()
            },
            paticle: Paticle,
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::new(1., 1.)),
                    ..default()
                },
                ..default()
            },
            impulse: ExternalImpulse::default(),
        }
    }
}
