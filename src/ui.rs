use bevy::prelude::*;

use crate::{creature::Health, inventory::Inventory, player::Player};

const BACKGROUND_COLOR: Color = Color::rgba(0.1, 0.3, 0.2, 0.5);
const SELECTED_COLOR: Color = Color::WHITE;
const TRANSPARENT: Color = Color::rgba(0., 0., 0., 0.);

#[derive(Debug, Component, Default)]
pub struct Ui;

#[derive(Debug, Component, Default)]
pub struct HealthIndicator;

#[derive(Debug, Component, Default)]
pub struct InventoryImageIndicator;

#[derive(Debug, Component, Default)]
pub struct InventoryContainer;

#[derive(Debug, Component, Default)]
pub struct InventoryAmountIndicator;

#[derive(Debug, Component)]
pub struct InventorySlot(u32);

pub fn update_health_ui(
    player_health: Query<&Health, (With<Player>, Changed<Health>)>,
    mut health_ui: Query<&mut Text, With<HealthIndicator>>,
) {
    if let Ok(health) = player_health.get_single() {
        if let Ok(mut text) = health_ui.get_single_mut() {
            if let Some(section) = text.sections.first_mut() {
                section.value = format!(" {}", health.0)
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
            let i = i as u32;
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

pub fn slot_buttons_system(
    mut interaction_query: Query<
        (&Interaction, &InventorySlot),
        (Changed<Interaction>, With<Button>, With<InventorySlot>),
    >,
    mut inventory: ResMut<Inventory>,
) {
    for (interaction, InventorySlot(slot)) in &mut interaction_query {
        let slot = *slot;
        match *interaction {
            Interaction::Pressed => {
                inventory.selected_slot = slot;
            }
            _ => (),
        }
    }
}

pub fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut assets: ResMut<Assets<TextureAtlas>>,
) {
    let font: Handle<Font> = asset_server.load("fonts/main_font.ttf");
    let text_style = TextStyle {
        font,
        ..Default::default()
    };
    // TODO: find a way to use texture atlas from ldtk
    let inventory_tiles = asset_server.load("atlas/icons_atlas.png");
    let texture_atlas =
        TextureAtlas::from_grid(inventory_tiles, Vec2::new(32., 32.), 16, 95, None, None);
    let texture_atlas = assets.add(texture_atlas);

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(0.),
                left: Val::Px(0.),
                height: Val::Percent(10.),
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(36.0)),
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Hp", text_style.to_owned()));
            parent.spawn((
                TextBundle::from_section("", text_style.to_owned()),
                HealthIndicator,
            ));
        });

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Start,
                row_gap: Val::Px(20.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for i in 0..5 {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                row_gap: Val::Px(10.),
                                border: UiRect::all(Val::Px(2.)),
                                ..default()
                            },
                            border_color: TRANSPARENT.into(),
                            background_color: BACKGROUND_COLOR.into(),
                            ..default()
                        },
                        InventorySlot(i),
                    ))
                    .with_children(|parent| {
                        parent
                            .spawn((
                                AtlasImageBundle {
                                    style: Style {
                                        width: Val::Px(50.),
                                        height: Val::Px(50.),
                                        justify_content: JustifyContent::End,
                                        align_items: AlignItems::End,
                                        padding: UiRect::all(Val::Px(4.)),
                                        ..default()
                                    },
                                    texture_atlas: texture_atlas.clone(),
                                    ..Default::default()
                                },
                                InventoryImageIndicator,
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    TextBundle {
                                        text: Text::from_section("0", text_style.clone()),
                                        ..Default::default()
                                    },
                                    InventoryAmountIndicator,
                                ));
                            });
                    });
            }
        });
}
