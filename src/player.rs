use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::LdtkEntity;
use bevy_rapier2d::prelude::*;

use crate::{
    collisions::ColliderBundle,
    creature::{CreatureBundle, Damage, Health},
    ladder::Climber,
};

#[derive(Component, Default, Debug)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Default, Debug)]
pub struct AnimationTimer(pub Timer);

#[derive(Component, Default, Debug)]
pub struct Player;

#[derive(Component, Default, Debug)]
pub enum MoveDirection {
    Up,
    Right,
    Down,
    Left,
    #[default]
    Idle,
}

#[derive(Bundle, Default)]
pub struct AnimationBundle {
    pub animation_indices: AnimationIndices,
    pub animation_timer: AnimationTimer,
}

#[derive(Resource)]
pub struct PlayerAtlases {
    pub idle: Handle<TextureAtlas>,
    pub run: Handle<TextureAtlas>,
    pub fall: Handle<TextureAtlas>,
}

pub fn animate_sprite(
    mut query: Query<
        (
            &mut AnimationTimer,
            &AnimationIndices,
            &mut TextureAtlasSprite,
        ),
        With<Player>,
    >,
    time: Res<Time>,
) {
    for (mut timer, indicies, mut sprite) in &mut query {
        let AnimationTimer(timer) = &mut *timer;
        timer.tick(time.delta());
        if timer.just_finished() {
            if sprite.index >= indicies.last {
                sprite.index = 0;
                continue;
            }
            sprite.index = if sprite.index == indicies.last {
                indicies.first
            } else {
                sprite.index + 1
            }
        };
    }
}

pub fn scale_player(mut q: Query<&mut Transform, Added<Player>>) {
    if let Ok(mut player_transform) = q.get_single_mut() {
        player_transform.scale = Vec3::splat(1.5);
    }
}

pub fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let run_texture_handle = asset_server.load("atlas/player_run.png");
    let idle_texture_handle = asset_server.load("atlas/player_idle.png");
    let fall_texture_handle = asset_server.load("atlas/player_fall.png");

    let run_texture_atlas =
        TextureAtlas::from_grid(run_texture_handle, [16.0, 16.0].into(), 1, 4, None, None);
    let idle_texture_atlas =
        TextureAtlas::from_grid(idle_texture_handle, [16.0, 16.0].into(), 1, 4, None, None);
    let fall_texture_atlas =
        TextureAtlas::from_grid(fall_texture_handle, [16.0, 16.0].into(), 1, 2, None, None);

    let run_texture_atlas_handle = texture_atlases.add(run_texture_atlas);
    let idle_texture_atlas_handle = texture_atlases.add(idle_texture_atlas);
    let fall_texture_atlas_handle = texture_atlases.add(fall_texture_atlas);

    commands.insert_resource(PlayerAtlases {
        idle: idle_texture_atlas_handle,
        run: run_texture_atlas_handle,
        fall: fall_texture_atlas_handle,
    });
}

pub fn update_animation_state(
    assets: Res<PlayerAtlases>,
    mut query: Query<
        (
            &mut TextureAtlasSprite,
            &MoveDirection,
            &mut AnimationIndices,
            &mut Handle<TextureAtlas>,
        ),
        With<Player>,
    >,
) {
    for (mut sprite, direction, mut indicies, mut texture) in &mut query {
        match *direction {
            MoveDirection::Right => {
                sprite.flip_x = false;
                indicies.last = 3;
                if sprite.index > 3 {
                    sprite.index = 0;
                }
                *texture = assets.run.clone();
            }
            MoveDirection::Left => {
                sprite.flip_x = true;
                indicies.last = 3;
                if sprite.index > 3 {
                    sprite.index = 0;
                }
                *texture = assets.run.clone();
            }
            MoveDirection::Idle => {
                indicies.last = 3;
                if sprite.index > 3 {
                    sprite.index = 0;
                }
                *texture = assets.idle.clone();
            }
            MoveDirection::Up | MoveDirection::Down => {
                indicies.last = 1;
                if sprite.index > 1 {
                    sprite.index = 0;
                }
                *texture = assets.fall.clone();
            }
        };
    }
}

pub fn movement(
    input: Res<Input<KeyCode>>,
    mut query: Query<
        (
            &mut KinematicCharacterController,
            &KinematicCharacterControllerOutput,
            &mut Climber,
            &mut MoveDirection,
        ),
        With<Player>,
    >,
) {
    for (mut controller, output, mut climber, mut direction) in &mut query {
        let right = if input.pressed(KeyCode::E) || input.pressed(KeyCode::Right) {
            1.
        } else {
            0.
        };
        let left = if input.pressed(KeyCode::A) || input.pressed(KeyCode::Left) {
            1.
        } else {
            0.
        };
        let mut transition_vector = output.effective_translation;

        transition_vector.x = (right - left) * 4.;

        if !output.grounded {
            *direction = MoveDirection::Up;
        } else if left == 1. {
            *direction = MoveDirection::Left;
        } else if right == 1. {
            *direction = MoveDirection::Right;
        } else {
            *direction = MoveDirection::Idle;
        }

        if climber.intersecting_climbables.is_empty() {
            climber.climbing = false;
        } else if input.just_pressed(KeyCode::Comma) || input.just_pressed(KeyCode::O) {
            climber.climbing = true;
        }

        if climber.climbing {
            let up = if input.pressed(KeyCode::Comma) {
                1.
            } else {
                0.
            };
            let down = if input.pressed(KeyCode::O) { 1. } else { 0. };

            transition_vector.y = (up - down) * 2.;
        }

        if (input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Up))
            && (output.grounded || climber.climbing)
        {
            transition_vector.y += 10.;
        } else {
            if !climber.climbing {
                transition_vector.y -= 1.;
            }
        }
        controller.translation = Some(transition_vector);
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub creature_bundle: CreatureBundle,
    pub player: Player,
    pub climber: Climber,
}

impl LdtkEntity for PlayerBundle {
    fn bundle_entity(
        entity_instance: &bevy_ecs_ldtk::EntityInstance,
        layer_instance: &bevy_ecs_ldtk::prelude::LayerInstance,
        tileset: Option<&Handle<Image>>,
        tileset_definition: Option<&bevy_ecs_ldtk::prelude::TilesetDefinition>,
        _asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) -> Self {
        let animation_indices = AnimationIndices { first: 0, last: 3 };
        let tileset_definition = tileset_definition.unwrap();
        let texture_atlas = TextureAtlas::from_grid(
            tileset.unwrap().clone(),
            Vec2::new(
                entity_instance.tile.unwrap().w as f32,
                entity_instance.tile.unwrap().h as f32,
            ),
            1,
            1,
            Some(Vec2::splat(tileset_definition.padding as f32)),
            Some(Vec2::new(
                entity_instance.tile.unwrap().x as f32,
                entity_instance.tile.unwrap().y as f32,
            )),
        );
        let texture_atlas_handle = texture_atlases.add(texture_atlas);

        let transform = Transform {
            translation: Vec3::new(
                (entity_instance.px.x + layer_instance.px_total_offset_x) as f32,
                (entity_instance.px.y + layer_instance.px_total_offset_y) as f32,
                1.0,
            ),
            rotation: Quat::from_rotation_x(22.),
            scale: Vec3::splat(4.),
        };
        let sprite_sheet_bundle = SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: texture_atlas_handle,
            transform,
            ..Default::default()
        };

        Self {
            climber: Climber::default(),
            creature_bundle: CreatureBundle {
                animation_bundle: AnimationBundle {
                    animation_timer: AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
                    animation_indices,
                },
                health: Health(100),
                damage: Damage(20),
                sprite: sprite_sheet_bundle,
                character_controller: KinematicCharacterController {
                    slide: true,
                    apply_impulse_to_dynamic_bodies: true,
                    filter_flags: QueryFilterFlags::from_bits(24).unwrap(),
                    ..default()
                },
                collider_bundle: ColliderBundle {
                    rigid_body: RigidBody::KinematicPositionBased,
                    collider: Collider::cuboid(4., 8.),
                    rotation_constraints: LockedAxes::ROTATION_LOCKED,
                    ..Default::default()
                },
                ..Default::default()
            },
            player: Player,
        }
    }
}
