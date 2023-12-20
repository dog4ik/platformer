use bevy::{prelude::*, ui::RelativeCursorPosition};

use crate::{creature::Health, player::Player};

pub const BACKGROUND_COLOR: Color = Color::INDIGO;
pub const SELECTED_COLOR: Color = Color::WHITE;
pub const TRANSPARENT: Color = Color::rgba(0., 0., 0., 0.);

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
pub struct InventorySlot(pub usize);

#[derive(Debug, Component, Default)]
pub struct ExpandedInventoryIndicator;

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

    let spawn_item_node = |parent: &mut ChildBuilder<'_, '_, '_>, i| {
        parent
            .spawn((
                ButtonBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        width: Val::Px(50.),
                        height: Val::Px(50.),
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.)),
                        ..default()
                    },
                    border_color: TRANSPARENT.into(),
                    background_color: BACKGROUND_COLOR.with_a(0.6).into(),
                    ..default()
                },
                InventorySlot(i),
                RelativeCursorPosition::default(),
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
                            z_index: ZIndex::Global(1),
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
    };

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
                row_gap: Val::Px(5.),
                column_gap: Val::Px(5.),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            for i in 0..4 {
                spawn_item_node(parent, i);
            }
        });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Px(50. * 5.),
                    height: Val::Percent(0.),
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Start,
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(5.),
                    column_gap: Val::Px(5.),
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            ExpandedInventoryIndicator,
        ))
        .with_children(|parent| {
            for i in 4..32 {
                spawn_item_node(parent, i);
            }
        });
}
