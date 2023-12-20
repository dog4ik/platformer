use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_ldtk::{
    ldtk::FieldDefinition,
    prelude::{FieldValue, LdtkEntity, TilesetDefinition},
    LdtkAsset,
};
use bevy_rapier2d::prelude::*;

use crate::{
    collisions::{ColliderBundle, GameCollisions},
    inventory::{Inventory, InventoryItem},
    player::Player,
};

#[derive(Component, Default)]
pub struct Item(String);

#[derive(Bundle, Default)]
pub struct ItemBundle {
    pub collider: ColliderBundle,
    pub sprite: SpriteSheetBundle,
    pub item_type: ItemType,
    pub item: Item,
}

#[derive(Component, Debug)]
pub struct TileInformation {
    pub tileset_identifier: i32,
    pub texture_atlas: Handle<TextureAtlas>,
    pub index: usize,
    pub item_type: Option<ItemType>,
}

#[derive(Debug, Resource, Default)]
pub struct EntitiesResource {
    pub entities: HashMap<String, TileInformation>,
}

impl EntitiesResource {
    pub fn spawn_item(&self, commands: &mut Commands, tile_name: String, position: Transform) {
        if let Some(item) = self.entities.get(&tile_name) {
            if let Some(item_type) = &item.item_type {
                commands.spawn(ItemBundle {
                    collider: ColliderBundle {
                        collider: Collider::cuboid(10., 5.),
                        rigid_body: RigidBody::Dynamic,
                        collision_groups: GameCollisions::Item.into(),
                        ..Default::default()
                    },
                    sprite: SpriteSheetBundle {
                        sprite: TextureAtlasSprite::new(item.index),
                        texture_atlas: item.texture_atlas.clone(),
                        transform: position.with_scale(Vec3::splat(0.4)),
                        ..Default::default()
                    },
                    item_type: item_type.clone(),
                    item: Item(tile_name),
                });
            }
        }
    }
}

pub fn generate_item_type(
    defenitions: &Vec<FieldDefinition>,
    tags: &Vec<String>,
) -> Option<ItemType> {
    return match tags.first()?.as_str() {
        "Consumable" => {
            let field = &defenitions.first()?;
            let override_value = field.default_override.clone()?;
            let arr = override_value.get("params")?;
            let value = arr.get(0)?;
            let val = value.as_u64()? as u32;
            Some(ItemType::Consumable(match field.identifier.as_str() {
                "Heal" => Consumable::Heal(val),
                "Damage" => Consumable::Damage(val),
                _ => panic!("field on consumable should be either Heal or Damage"),
            }))
        }
        "Material" => Some(ItemType::CraftMaterial),
        "Weapon" => todo!(),
        _ => panic!("all tags should be covered"),
    };
}

pub fn generate_assets_for_entries(
    map_assets: Res<Assets<LdtkAsset>>,
    mut tilesets: ResMut<EntitiesResource>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if map_assets.is_changed() {
        for (_, asset) in map_assets.iter() {
            let mut tilesets_definitions: HashMap<i32, (&TilesetDefinition, &Handle<Image>)> =
                HashMap::new();
            for tileset in &asset.project.defs.tilesets {
                if let Some(image) = asset.tileset_map.get(&tileset.uid) {
                    tilesets_definitions.insert(tileset.uid, (tileset, image));
                }
            }

            for entity in &asset.project.defs.entities {
                if let Some(tile_rect) = entity.tile_rect {
                    // BUG: not all entities have item_type and can be used as item
                    let item_type = generate_item_type(&entity.field_defs, &entity.tags);

                    let tileset_id = entity
                        .tileset_id
                        .expect("entities with rect to have tileset");
                    let (tileset_definition, tileset_image) =
                        *tilesets_definitions.get(&tileset_id).expect("to exist");
                    let row = tile_rect.y / tile_rect.h;
                    let column = tile_rect.x / tile_rect.w;
                    let index = row * tileset_definition.c_wid + column;

                    let texture_atlas = TextureAtlas::from_grid(
                        tileset_image.clone(),
                        Vec2::new(tile_rect.w as f32, tile_rect.h as f32),
                        tileset_definition.c_wid as usize,
                        tileset_definition.c_hei as usize,
                        None,
                        None,
                    );
                    let texture_atlas_handle = texture_atlases.add(texture_atlas);
                    tilesets.entities.insert(
                        entity.identifier.clone(),
                        TileInformation {
                            tileset_identifier: tileset_id,
                            texture_atlas: texture_atlas_handle,
                            index: index as usize,
                            item_type,
                        },
                    );
                }
            }
        }
    }
}

impl LdtkEntity for ItemBundle {
    fn bundle_entity(
        entity_instance: &bevy_ecs_ldtk::EntityInstance,
        _layer_instance: &bevy_ecs_ldtk::prelude::LayerInstance,
        tileset: Option<&Handle<Image>>,
        tileset_definition: Option<&bevy_ecs_ldtk::prelude::TilesetDefinition>,
        _asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) -> Self {
        let entity_tile = entity_instance.tile.unwrap();
        let tileset_definition = tileset_definition.unwrap();
        let name = entity_instance.identifier.clone();
        let item_type = match entity_instance
            .tags
            .first()
            .expect("ldtk entities contain tags")
            .as_str()
        {
            "Consumable" => {
                let field = &entity_instance
                    .field_instances
                    .first()
                    .expect("to have at least one field");

                match field.value {
                    FieldValue::Int(value) => {
                        let value = value.expect("to be non empty") as u32;
                        match field.identifier.as_str() {
                            "Damage" => ItemType::Consumable(Consumable::Damage(value)),
                            "Heal" => ItemType::Consumable(Consumable::Heal(value)),
                            _ => panic!("unknown field"),
                        }
                    }
                    _ => panic!("field shoud be typeof integer"),
                }
            }
            "Material" => ItemType::CraftMaterial,
            _ => panic!("all tags should be covered"),
        };

        let row = entity_tile.y / entity_tile.h;
        let column = entity_tile.x / entity_tile.w;
        let index = row * tileset_definition.c_wid + column;

        let texture_atlas = TextureAtlas::from_grid(
            tileset.unwrap().clone(),
            Vec2::new(entity_tile.w as f32, entity_tile.h as f32),
            tileset_definition.c_wid as usize,
            tileset_definition.c_hei as usize,
            None,
            None,
        );

        let texture_atlas_sprite = TextureAtlasSprite {
            index: index as usize,
            ..Default::default()
        };
        let texture_atlas = texture_atlases.add(texture_atlas);

        Self {
            collider: ColliderBundle {
                collider: Collider::ball(10.),
                rigid_body: RigidBody::Dynamic,
                ..Default::default()
            },
            sprite: SpriteSheetBundle {
                sprite: texture_atlas_sprite,
                texture_atlas,
                ..Default::default()
            },
            item: Item(name),
            item_type,
        }
    }
}

#[derive(Debug, Clone, Component)]
pub enum ItemType {
    Consumable(Consumable),
    Weapon(Weapon),
    CraftMaterial,
}
impl Default for ItemType {
    fn default() -> Self {
        Self::Consumable(Consumable::Heal(0))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Consumable {
    Heal(u32),
    Damage(u32),
}
impl Default for Consumable {
    fn default() -> Self {
        Self::Heal(0)
    }
}

#[derive(Default, Debug)]
pub struct CraftMaterial;

#[derive(Default, Clone, Copy, Debug)]
pub enum WeaponType {
    #[default]
    Sword,
    Bow,
}

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub damage: usize,
}

pub fn pickup_item(
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    player: Query<Entity, With<Player>>,
    items: Query<(Entity, &TextureAtlasSprite, &ItemType, &Item), With<Item>>,
    rapier_context: Res<RapierContext>,
) {
    let slots = inventory.slots;
    let size = inventory.items.len();
    let capacity = inventory.capacity;
    let current_capacity = inventory.current_capacity;
    if slots <= size || current_capacity >= capacity {
        return;
    };

    // TODO: make generic function to check collisions
    if let Ok(player_entity) = player.get_single() {
        for (entity, sprite, item_type, Item(item_name)) in &items {
            let contact = rapier_context.contact_pair(player_entity, entity);
            if contact.is_some() {
                if let Err(e) = inventory.add_item(InventoryItem {
                    icon: sprite.clone(),
                    name: item_name.clone(),
                    item_type: item_type.clone(),
                    amount: 1,
                }) {
                    warn!("{}", e)
                } else {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
