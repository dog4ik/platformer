use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::collisions::{Climber, ColliderBundle, GroundDetection};

#[derive(Component, Default)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Default)]
pub struct AnimationTimer(pub Timer);

#[derive(Component, Default)]
pub struct Player;

#[derive(Component, Default)]
pub enum MoveDirection {
    Up,
    Right,
    Down,
    Left,
    #[default]
    Idle,
}

#[derive(Bundle, Default)]
struct AnimationBundle {
    animation_indices: AnimationIndices,
    animation_timer: AnimationTimer,
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
            sprite.index = if sprite.index == indicies.last {
                indicies.first
            } else {
                sprite.index + 1
            }
        };
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
        idle: idle_texture_atlas_handle.clone(),
        run: run_texture_atlas_handle,
        fall: fall_texture_atlas_handle,
    });

    let animation_indices = AnimationIndices { first: 0, last: 3 };

    // TODO: bundles!
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: idle_texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform {
                translation: (150., 150., 100.).into(),
                scale: Vec3::splat(1.4),
                ..Default::default()
            },
            ..Default::default()
        },
        AnimationBundle {
            animation_timer: AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
            animation_indices,
        },
        Player,
        Climber::default(),
        GroundDetection::default(),
        MoveDirection::default(),
        Collider::cuboid(8., 8.),
        RigidBody::KinematicPositionBased,
        KinematicCharacterController {
            slide: true,
            ..default()
        },
        KinematicCharacterControllerOutput::default(),
    ));
}

pub fn update_animation_state(
    assets: Res<PlayerAtlases>,
    mut query: Query<
        (
            &mut TextureAtlasSprite,
            &MoveDirection,
            &mut Handle<TextureAtlas>,
        ),
        With<Player>,
    >,
) {
    for (mut sprite, direction, mut texture) in &mut query {
        match *direction {
            MoveDirection::Right => {
                sprite.flip_x = false;
                *texture = assets.run.clone();
            }
            MoveDirection::Left => {
                sprite.flip_x = true;
                *texture = assets.run.clone();
            }
            MoveDirection::Idle => {
                *texture = assets.idle.clone();
            }
            _ => todo!(),
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

        if left == 1. {
            *direction = MoveDirection::Left;
        } else if right == 1. {
            *direction = MoveDirection::Right;
        } else {
            *direction = MoveDirection::Idle;
        }

        if climber.intersecting_climbables.is_empty() {
            climber.climbing = false;
        } else if input.just_pressed(KeyCode::W) || input.just_pressed(KeyCode::S) {
            climber.climbing = true;
        }

        if climber.climbing {
            let up = if input.pressed(KeyCode::W) { 1. } else { 0. };
            let down = if input.pressed(KeyCode::S) { 1. } else { 0. };

            transition_vector.y += (up - down) * 2.;
        }

        if (input.just_pressed(KeyCode::Space) || input.just_pressed(KeyCode::Up))
            && (output.grounded || climber.climbing)
        {
            transition_vector.y += 10.;
            climber.climbing = false;
        } else {
            transition_vector.y -= 1.;
        }
        controller.translation = Some(transition_vector);
    }
}

#[derive(Bundle)]
pub struct PlayerBundle {
    pub sprite_sheet_bundle: SpriteSheetBundle,
    pub animation_indices: AnimationIndices,
    pub collider_bundle: ColliderBundle,
    pub player: Player,
    pub climber: Climber,
    pub ground_detection: GroundDetection,
}
