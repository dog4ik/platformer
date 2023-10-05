use bevy::{prelude::*, utils::HashSet};
use bevy_ecs_ldtk::prelude::*;

use crate::collisions::SensorBundle;

#[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Climber {
    pub climbing: bool,
    pub intersecting_climbables: HashSet<Entity>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Climbable;

#[derive(Clone, Debug, Default, Bundle)]
pub struct LadderBundle {
    pub sensor_bundle: SensorBundle,
    pub climbable: Climbable,
}

impl LdtkIntCell for LadderBundle {
    fn bundle_int_cell(int_grid_cell: IntGridCell, _layer_instance: &LayerInstance) -> Self {
        LadderBundle {
            sensor_bundle: int_grid_cell.into(),
            climbable: Climbable,
        }
    }
}
