use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    collisions::ColliderBundle,
    enemy::Loot,
    items::EntitiesResource,
    particles::spawn_splash_particles,
    player::{AnimationBundle, MoveDirection},
};

#[derive(Component)]
pub struct Health(pub isize);

#[derive(Component, Default, Clone, Debug)]
pub struct Damage(pub isize);

impl Default for Health {
    fn default() -> Self {
        Self(100)
    }
}

#[derive(Bundle, Default)]
pub struct CreatureBundle {
    pub health: Health,
    pub damage: Damage,
    pub sprite: SpriteSheetBundle,
    pub move_direction: MoveDirection,
    pub character_controller: KinematicCharacterController,
    pub character_output: KinematicCharacterControllerOutput,
    pub collider_bundle: ColliderBundle,
    pub animation_bundle: AnimationBundle,
}

pub fn detect_creature_death(
    mut commands: Commands,
    creatures: Query<(Entity, &Health, &Transform, Option<&Loot>), Changed<Health>>,
    entities_resource: Res<EntitiesResource>,
) {
    for (creature, Health(health), transform, loot) in &creatures {
        if *health <= 0 {
            println!("despawned died creature");
            commands.entity(creature).despawn();
            if let Some(Loot(loot)) = loot {
                loot.iter().for_each(|item| {
                    entities_resource.spawn_item(&mut commands, item.clone(), transform.clone());
                });
            }

            spawn_splash_particles(&mut commands, 40, transform.clone(), None);
        }
    }
}
