use bevy::prelude::*;
use bevy_ecs_ldtk::{prelude::*, utils::ldtk_pixel_coords_to_translation_pivoted};
use bevy_rapier2d::prelude::*;

use crate::creature::{CreatureBundle, Damage, Health};

#[derive(Component, Default)]
pub struct Enemy;

pub struct SeePlayer(pub bool);

#[derive(Default)]
enum EnemyType {
    #[default]
    Common,
    Strong,
    Boss,
}

impl EnemyType {
    fn get_random_enemy() -> Self {
        match rand::random::<f32>() {
            x if x < 0.7 => Self::Common,
            x if x < 0.9 && x >= 0.7 => Self::Strong,
            x if x >= 0.9 => Self::Boss,
            _ => Self::default(),
        }
    }
}

#[derive(Bundle, Default)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub creature_bundle: CreatureBundle,
    pub patrol: Patrol,
}

impl LdtkEntity for EnemyBundle {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        layer_instance: &LayerInstance,
        tileset: Option<&Handle<Image>>,
        tileset_definition: Option<&TilesetDefinition>,
        _asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) -> Self {
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
        let sprite_sheet_bundle = SpriteSheetBundle {
            sprite: TextureAtlasSprite::default(),
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_xyz(
                (entity_instance.px.x + layer_instance.px_total_offset_x) as f32,
                (entity_instance.px.y + layer_instance.px_total_offset_y) as f32,
                1.0,
            ),
            ..Default::default()
        };
        Self {
            enemy: Enemy,
            creature_bundle: CreatureBundle {
                health: Health(100),
                damage: Damage(5),
                sprite: sprite_sheet_bundle,
                collider_bundle: entity_instance.into(),
                ..Default::default()
            },
            patrol: Patrol::from((entity_instance, layer_instance)),
        }
    }
}

#[derive(Component, Default)]
pub struct Patrol {
    pub points: Vec<Vec2>,
    pub index: usize,
    pub forward: bool,
}

impl From<(&EntityInstance, &LayerInstance)> for Patrol {
    fn from(value: (&EntityInstance, &LayerInstance)) -> Self {
        let (entity_instance, layer_instance) = value;
        let mut points = Vec::new();
        points.push(ldtk_pixel_coords_to_translation_pivoted(
            entity_instance.px,
            layer_instance.c_hei * layer_instance.grid_size,
            IVec2::new(entity_instance.width, entity_instance.height),
            entity_instance.pivot,
        ));

        let ldtk_patrol_points = entity_instance
            .iter_points_field("patrol")
            .expect("patrol field should be correclty typed");

        for ldtk_point in ldtk_patrol_points {
            // The +1 is necessary here due to the pivot of the entities in the sample
            // file.
            // The patrols set up in the file look flat and grounded,
            // but technically they're not if you consider the pivot,
            // which is at the bottom-center for the skulls.
            let pixel_coords = (ldtk_point.as_vec2() + Vec2::new(0.5, 1.))
                * Vec2::splat(layer_instance.grid_size as f32);

            points.push(ldtk_pixel_coords_to_translation_pivoted(
                pixel_coords.as_ivec2(),
                layer_instance.c_hei * layer_instance.grid_size,
                IVec2::new(entity_instance.width, entity_instance.height),
                entity_instance.pivot,
            ));
        }

        Patrol {
            points,
            index: 1,
            forward: true,
        }
    }
}

pub fn spawn_enemy(mut commands: Commands) {
    let mut entity = commands.spawn_empty();
    entity.insert(Enemy);
    entity.insert(Patrol::default());
    let enemy_type = EnemyType::get_random_enemy();
    match enemy_type {
        EnemyType::Common => entity.insert(CreatureBundle {
            sprite: SpriteSheetBundle {
                ..Default::default()
            },
            health: Health(100),
            damage: Damage(10),
            ..Default::default()
        }),
        EnemyType::Strong => entity.insert(CreatureBundle {
            sprite: SpriteSheetBundle {
                ..Default::default()
            },
            health: Health(150),
            damage: Damage(30),
            ..Default::default()
        }),
        EnemyType::Boss => entity.insert(CreatureBundle {
            sprite: SpriteSheetBundle {
                ..Default::default()
            },
            health: Health(300),
            damage: Damage(50),
            ..Default::default()
        }),
    };
    commands.spawn((Enemy, Patrol::default()));
}

pub fn patrol(mut query: Query<(&mut Transform, &mut Velocity, &mut Patrol)>) {
    for (mut transform, mut velocity, mut patrol) in &mut query {
        if patrol.points.len() <= 1 {
            continue;
        }

        let mut new_velocity =
            (patrol.points[patrol.index] - transform.translation.truncate()).normalize() * 75.;

        if new_velocity.dot(velocity.linvel) < 0. {
            if patrol.index == 0 {
                patrol.forward = true;
            } else if patrol.index == patrol.points.len() - 1 {
                patrol.forward = false;
            }

            transform.translation.x = patrol.points[patrol.index].x;
            transform.translation.y = patrol.points[patrol.index].y;

            if patrol.forward {
                patrol.index += 1;
            } else {
                patrol.index -= 1;
            }

            new_velocity =
                (patrol.points[patrol.index] - transform.translation.truncate()).normalize() * 75.;
        }

        velocity.linvel = new_velocity;
    }
}
