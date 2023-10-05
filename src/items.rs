use bevy::prelude::*;
use bevy_ecs_ldtk::{prelude::LdtkEntity, EntityInstance};
use bevy_rapier2d::prelude::*;

use crate::{
    collisions::ColliderBundle,
    inventory::{Inventory, InventoryItem},
    player::Player,
};

#[derive(Component, Default)]
pub struct Item(String);

#[derive(Bundle, Default)]
pub struct ItemBundle {
    pub collider: ColliderBundle,
    pub sprite: SpriteSheetBundle,
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
        }
    }
}

#[derive(Debug, Clone)]
pub enum ItemType {
    Consumable(Consumable),
    Material,
    Weapon,
}

#[derive(Component, Debug, Clone)]
pub enum Consumable {
    Heal(u32),
    Damage(u32),
}
impl Default for Consumable {
    fn default() -> Self {
        Self::Heal(10)
    }
}

#[derive(Component, Default, Debug)]
pub struct Material;

#[derive(Component, Default, Debug)]
pub struct Weapon;

#[derive(Bundle, Default, LdtkEntity)]
pub struct MeatBundle {
    #[from_entity_instance]
    pub use_type: Consumable,
    #[ldtk_entity]
    pub item_bundle: ItemBundle,
    #[from_entity_instance]
    pub item: Item,
}

#[derive(Bundle, Default, LdtkEntity)]
pub struct AppleBundle {
    #[from_entity_instance]
    pub use_type: Consumable,
    #[ldtk_entity]
    pub item_bundle: ItemBundle,
    #[from_entity_instance]
    pub item: Item,
}

#[derive(Bundle, Default, LdtkEntity)]
pub struct PillsBundle {
    #[from_entity_instance]
    pub use_type: Consumable,
    #[ldtk_entity]
    pub item_bundle: ItemBundle,
    #[from_entity_instance]
    pub item: Item,
}

impl From<&EntityInstance> for Consumable {
    fn from(entity_instance: &EntityInstance) -> Self {
        match entity_instance.identifier.as_str() {
            "Apple" => Self::Heal(10),
            "Meat" => Self::Heal(40),
            "Pills" => Self::Damage(25),
            _ => Self::Heal(0),
        }
    }
}

impl From<&EntityInstance> for Item {
    fn from(entity_instance: &EntityInstance) -> Self {
        Self(entity_instance.identifier.to_string())
    }
}

pub fn pickup_item(
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    player: Query<Entity, With<Player>>,
    items: Query<(Entity, &TextureAtlasSprite, &Consumable, &Item), With<Item>>,
    rapier_context: Res<RapierContext>,
) {
    let slots = inventory.slots;
    let size = inventory.items.len() as u32;
    let capacity = inventory.capacity;
    let current_capacity = inventory.current_capacity;
    if slots <= size || current_capacity >= capacity {
        return;
    };

    // TODO: make generic function to check collisions
    if let Ok(player_entity) = player.get_single() {
        for (entity, sprite, effect, Item(item_name)) in &items {
            let contact = rapier_context.contact_pair(player_entity, entity);
            if contact.is_some() {
                let mut closest_slot = 0;
                for (key, val) in &inventory.items {
                    if *key != closest_slot && val.name != *item_name {
                        closest_slot += 1;
                        continue;
                    } else if val.name == *item_name {
                        closest_slot = *key;
                        break;
                    } else {
                        closest_slot += 1;
                    }
                }

                if let Err(e) = inventory.add_item(
                    closest_slot,
                    InventoryItem {
                        icon: sprite.clone(),
                        name: item_name.clone(),
                        item_type: ItemType::Consumable(effect.clone()),
                        amount: 1,
                    },
                ) {
                    warn!("{}", e)
                } else {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
