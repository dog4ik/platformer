use std::fmt::Display;

use bevy::{prelude::*, ui::RelativeCursorPosition, utils::HashMap};

use crate::{
    creature::Health,
    items::{Consumable, ItemType},
    player::Player,
    ui::{
        ExpandedInventoryIndicator, InventoryAmountIndicator, InventoryImageIndicator,
        InventorySlot, SELECTED_COLOR, TRANSPARENT,
    },
};

#[derive(Resource, Debug, Clone)]
pub struct InventoryDragState {
    pub is_dragging: bool,
    pub slot: usize,
}
impl Default for InventoryDragState {
    fn default() -> Self {
        Self {
            is_dragging: false,
            slot: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InventoryItem {
    pub icon: TextureAtlasSprite,
    pub item_type: ItemType,
    pub name: String,
    pub amount: u32,
}

#[derive(Resource)]
pub struct Inventory {
    pub capacity: usize,
    pub current_capacity: usize,
    pub slots: usize,
    pub selected_slot: usize,
    pub items: HashMap<usize, InventoryItem>,
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
    pub fn add_item(&mut self, item: InventoryItem) -> Result<(), InventoryError> {
        let mut closest_slot = 0;
        let mut found_fit = false;
        for (key, val) in self.items.iter() {
            if val.name == item.name && val.amount < 64 {
                closest_slot = *key;
                found_fit = true;
                break;
            }
        }
        if !found_fit {
            let mut taken_slots: Vec<&usize> = self.items.keys().collect();
            taken_slots.sort();
            for slot in taken_slots {
                if *slot == closest_slot {
                    closest_slot += 1;
                }
            }
        }

        self.add_item_in_slot(closest_slot, item)?;

        Ok(())
    }
    pub fn add_item_in_slot(
        &mut self,
        slot: usize,
        item: InventoryItem,
    ) -> Result<(), InventoryError> {
        if self.slots <= self.items.len() {
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

    pub fn remove_item(&mut self, slot: usize) {
        if let Some(val) = self.items.get_mut(&slot) {
            if val.amount > 1 {
                val.amount -= 1;
            } else {
                self.items.remove(&slot);
            }
            self.current_capacity -= 1;
        }
    }

    pub fn move_slot(&mut self, from: usize, to: usize) {
        if from == to {
            return;
        }
        if let Some(from_item) = self.items.remove(&from) {
            if let Some(swap_item) = self.items.insert(to, from_item) {
                self.items.insert(from, swap_item);
            }
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
                            Consumable::Heal(amount) => health.0 += *amount as isize,
                            Consumable::Damage(amount) => health.0 -= *amount as isize,
                        }
                        inventory.remove_item(selected_slot);
                    };
                }
            }
        }
    }
}

pub fn update_inventory_ui(
    mut images: Query<&mut UiTextureAtlasImage, With<InventoryImageIndicator>>,
    mut amounts: Query<&mut Text, With<InventoryAmountIndicator>>,
    mut slots_borders: Query<&mut BorderColor, With<InventorySlot>>,
    inventory: Res<Inventory>,
) {
    if inventory.is_changed() {
        let images = images.iter_mut();
        let amounts = amounts.iter_mut();
        let borders = slots_borders.iter_mut();
        let zipped_iterator = images.zip(amounts.zip(borders));

        for (i, (mut image, (mut amount, mut border))) in zipped_iterator.enumerate() {
            let i = i;
            if inventory.selected_slot == i {
                border.0 = SELECTED_COLOR;
            } else {
                border.0 = TRANSPARENT;
            }
            if let Some(item) = inventory.items.get(&i) {
                image.index = item.icon.index;
                amount.sections.first_mut().unwrap().value = item.amount.to_string();
            } else {
                image.index = 0;
                amount.sections.first_mut().unwrap().value = "0".to_string();
            }
        }
    }
}

pub fn toggle_inventory(
    input: Res<Input<KeyCode>>,
    mut expanded_inventory: Query<&mut Visibility, With<ExpandedInventoryIndicator>>,
) {
    if input.is_changed() {
        if input.just_pressed(KeyCode::C) {
            if let Ok(mut visibility) = expanded_inventory.get_single_mut() {
                match visibility.as_mut() {
                    Visibility::Hidden => *visibility = Visibility::Visible,
                    Visibility::Visible => *visibility = Visibility::Hidden,
                    _ => (),
                }
            }
        }
    }
}

pub fn update_selected_slot(input: Res<Input<KeyCode>>, mut inventory: ResMut<Inventory>) {
    if input.is_changed() {
        for press in input.get_just_pressed() {
            match press {
                KeyCode::Key1 => inventory.selected_slot = 0,
                KeyCode::Key2 => inventory.selected_slot = 1,
                KeyCode::Key3 => inventory.selected_slot = 2,
                KeyCode::Key4 => inventory.selected_slot = 3,
                KeyCode::Key5 => inventory.selected_slot = 4,
                _ => (),
            }
        }
    }
}

pub fn move_drag_objects(
    drag_state: Res<InventoryDragState>,
    slots_query: Query<(&InventorySlot, &RelativeCursorPosition, &Children), With<Button>>,
    mut children_query: Query<&mut Style, With<InventoryImageIndicator>>,
) {
    for (InventorySlot(slot), cursor_position, children) in &slots_query {
        let first_child = children.first().expect("to have first child");
        let mut style = children_query.get_mut(*first_child).expect("to exist");
        if *slot == drag_state.slot && drag_state.is_dragging {
            if let Some(cursor_position) = cursor_position.as_ref() {
                if drag_state.is_dragging {
                    style.position_type = PositionType::Absolute;
                    style.left = Val::Px(cursor_position.x * 50. - 25.);
                    style.top = Val::Px(cursor_position.y * 50. - 25.);
                }
            }
        } else {
            style.position_type = PositionType::Relative;
            style.left = Val::Auto;
            style.top = Val::Auto;
        }
    }
}

pub fn slot_buttons_system(
    interaction_query: Query<(&InventorySlot, &Interaction, &RelativeCursorPosition), With<Button>>,
    mut inventory: ResMut<Inventory>,
    mut drag_state: ResMut<InventoryDragState>,
    mut input: ResMut<Input<MouseButton>>,
) {
    let is_released = input.just_released(MouseButton::Left);
    for (InventorySlot(slot), interaction, cursor_position) in &interaction_query {
        let slot = *slot;
        let mouse_over = cursor_position.mouse_over();
        if drag_state.is_dragging && is_released && !mouse_over {
            drag_state.is_dragging = false;
        }
        if is_released && mouse_over {
            drag_state.is_dragging = false;
            inventory.move_slot(drag_state.slot, slot);
        } else if let Interaction::Pressed = *interaction {
            drag_state.slot = slot;
            inventory.selected_slot = slot;
            if !cursor_position.mouse_over() {
                drag_state.is_dragging = true;
            }
            input.clear_just_pressed(MouseButton::Left);
        }
    }
}
