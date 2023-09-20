use bevy::{asset::ChangeWatcher, prelude::*, window::close_on_esc};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use platformer::{
    camera::{camera_fit_inside_current_level, setup_camera},
    collisions::{ground_detection, spawn_ground_sensor, update_on_ground},
    map::{setup_map, spawn_wall_collision, WallBundle},
    player::{animate_sprite, movement, setup_player, update_animation_state},
};

struct Game;
struct PsysicsPlugin;
struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin::default())
            .insert_resource(LevelSelection::Uid(0));
    }
}

impl Plugin for PsysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
        ))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -200.0),
            ..Default::default()
        });
    }
}

impl Plugin for Game {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_player,))
            .add_systems(
                Update,
                (
                    animate_sprite,
                    update_animation_state,
                    camera_fit_inside_current_level,
                    movement,
                    spawn_ground_sensor,
                    spawn_wall_collision,
                    update_on_ground,
                    ground_detection,
                ),
            )
            .register_ldtk_int_cell::<WallBundle>(1)
            .register_ldtk_int_cell::<WallBundle>(3);
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    watch_for_changes: ChangeWatcher::with_delay(std::time::Duration::from_millis(
                        200,
                    )),
                    ..default()
                }),
            Game,
            PsysicsPlugin,
            MapPlugin,
        ))
        .add_systems(Update, close_on_esc)
        .add_systems(Startup, (setup_camera, setup_map))
        .run();
}
