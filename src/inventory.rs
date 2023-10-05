use std::fmt::Display;

use bevy::{prelude::*, utils::HashMap};

use crate::{
    creature::Health,
    items::{Consumable, ItemType},
    player::Player,
};

#[derive(Debug)]
pub struct InventoryItem {
    pub icon: TextureAtlasSprite,
    pub item_type: ItemType,
    pub name: String,
    pub amount: u32,
}

#[derive(Resource)]
pub struct Inventory {
    pub capacity: u32,
    pub current_capacity: u32,
    pub slots: u32,
    pub selected_slot: u32,
    pub items: HashMap<u32, InventoryItem>,
}

impl Default for Inventory {
    fn default() -> Self {
        let slots = 32;
        Self {
            capacity: 100,
            slots,
            current_capacity: 0,
            selected_slot: 0,
            items: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub enum InventoryError {
    NoSlots,
    NoCapacity,
}

impl Display for InventoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InventoryError::NoSlots => write!(f, "No more slots available"),
            InventoryError::NoCapacity => write!(f, "No more capacity available"),
        }
    }
}

impl Inventory {
    pub fn add_item(&mut self, slot: u32, item: InventoryItem) -> Result<(), InventoryError> {
        if self.slots <= self.items.len() as u32 {
            return Err(InventoryError::NoSlots);
        }
        if self.current_capacity >= self.capacity {
            return Err(InventoryError::NoCapacity);
        };

        if let Some(item) = self.items.get_mut(&slot) {
            item.amount += 1;
        } else {
            self.items.insert(slot, item);
        }
        self.current_capacity += 1;
        Ok(())
    }

    pub fn remove_item(&mut self, slot: u32) {
        if let Some(val) = self.items.get_mut(&slot) {
            if val.amount > 1 {
                val.amount -= 1;
            } else {
                self.items.remove(&slot);
            }
            self.current_capacity -= 1;
        }
    }
}

pub fn consume_selected_item(
    input: Res<Input<KeyCode>>,
    mut inventory: ResMut<Inventory>,
    mut player_health: Query<&mut Health, With<Player>>,
) {
    if input.is_changed() {
        if input.just_pressed(KeyCode::Period) {
            if let Ok(mut health) = player_health.get_single_mut() {
                let selected_slot = inventory.selected_slot.clone();
                let val = inventory.items.get_mut(&selected_slot);
                if let Some(item) = val {
                    if let ItemType::Consumable(effect) = &item.item_type {
                        match effect {
                            Consumable::Heal(amount) => health.0 += amount,
                            Consumable::Damage(amount) => health.0 -= amount,
                        }
                    };
                    inventory.remove_item(selected_slot);
                }
            }
        }
    }
}
