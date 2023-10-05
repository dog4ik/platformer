use bevy::{asset::ChangeWatcher, prelude::*, window::close_on_esc};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use platformer::{
    camera::{camera_fit_inside_current_level, setup_camera},
    collisions::{detect_climb_range, detect_player_damage},
    enemy::{patrol, EnemyBundle},
    inventory::{consume_selected_item, Inventory},
    items::{pickup_item, AppleBundle, MeatBundle, PillsBundle},
    ladder::LadderBundle,
    map::{setup_map, spawn_wall_collision, update_level_selection, WallBundle},
    player::{
        animate_sprite, movement, scale_player, setup_player, update_animation_state, PlayerBundle,
    },
    projectile::{despawn_projectiles, shoot_projectile, ProjectilesGlobalAmount},
    ui::{
        setup_ui, slot_buttons_system, update_health_ui, update_inventory_ui, update_selected_slot,
    },
};

struct Game;
struct PsysicsPlugin;
struct MapPlugin;
struct HelperPlugin;
struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_health_ui,
                update_inventory_ui,
                update_selected_slot,
                slot_buttons_system,
            ),
        );
    }
}

impl Plugin for HelperPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // FrameTimeDiagnosticsPlugin::default(),
            // LogDiagnosticsPlugin::default(),
        ));
    }
}

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(LdtkPlugin::default())
            .insert_resource(LevelSelection::Uid(0))
            .insert_resource(LdtkSettings {
                level_spawn_behavior: LevelSpawnBehavior::UseZeroTranslation,
                ..Default::default()
            });
    }
}

impl Plugin for PsysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(50.0),
            RapierDebugRenderPlugin::default(),
        ))
        .insert_resource(RapierConfiguration {
            gravity: Vec2::new(0.0, -1000.0),
            ..Default::default()
        });
    }
}

impl Plugin for Game {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_player,))
            .insert_resource(ProjectilesGlobalAmount::default())
            .insert_resource(Inventory::default())
            .add_systems(
                Update,
                (
                    scale_player,
                    movement,
                    update_animation_state,
                    animate_sprite,
                    camera_fit_inside_current_level,
                    spawn_wall_collision,
                    shoot_projectile,
                    despawn_projectiles,
                    detect_climb_range,
                    patrol,
                    update_level_selection,
                    detect_player_damage,
                    pickup_item,
                    consume_selected_item,
                ),
            )
            .register_ldtk_int_cell::<WallBundle>(1)
            .register_ldtk_entity::<EnemyBundle>("Mob")
            .register_ldtk_entity::<PlayerBundle>("Player")
            .register_ldtk_entity::<AppleBundle>("Apple")
            .register_ldtk_entity::<MeatBundle>("Meat")
            .register_ldtk_entity::<PillsBundle>("Pills")
            .register_ldtk_int_cell::<LadderBundle>(2)
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
            HelperPlugin,
            UiPlugin,
        ))
        .add_systems(Update, close_on_esc)
        .add_systems(Startup, (setup_camera, setup_map, setup_ui))
        .run();
}
