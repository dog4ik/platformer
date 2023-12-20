use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::hash::Hash;

use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_ldtk::{ldtk::Level, prelude::*};

use crate::{map::Wall, player::Player};

#[derive(Resource, Default)]
pub struct AstarGrid {
    pub heap: BinaryHeap<Reverse<usize>>,
}

impl AstarGrid {
    fn add_item(&mut self, item: usize) {
        self.heap.push(Reverse(item))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Node {
    walkable: bool,
    world_pos: (usize, usize),
    grid_pos: (usize, usize),
    g_cost: usize,
    h_cost: usize,
}

impl Node {
    fn new(walkable: bool, world_pos: Vec2, grid_pos: Vec2) -> Self {
        Self {
            walkable,
            world_pos: (world_pos.x as usize, world_pos.y as usize),
            grid_pos: (grid_pos.x as usize, grid_pos.y as usize),
            g_cost: 0,
            h_cost: 0,
        }
    }

    fn f_cost(&self) -> usize {
        self.g_cost + self.h_cost
    }
}

pub fn generate_waypoint_graph(
    walls_query: Query<&Transform, With<Wall>>,
    level_query: Query<(&Transform, &Handle<LdtkLevel>), Without<Player>>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
    mut astar_grid: ResMut<AstarGrid>,
) {
    let (
        Transform {
            translation: lvl_position,
            ..
        },
        lvl_handle,
    ) = level_query.get_single().expect("lvl to be loaded");
    let lvl_position = lvl_position.truncate();
    let lvl: &Level = &ldtk_levels.get(lvl_handle).expect("lvl to be loaded").level;
    let lvl_height = lvl.px_hei as f32;
    let lvl_width = lvl.px_wid as f32;
    let lvl_dimentions = Vec2::new(lvl_width, lvl_height);
    let lvl_bounds = Rect {
        min: lvl_position,
        max: lvl_position + lvl_dimentions,
    };
}

#[allow(clippy::missing_panics_doc)]
pub fn astar<FN, IN, FH, FS>(
    start: &Node,
    mut successors: FN,
    mut heuristic: FH,
    mut success: FS,
) -> Option<(Vec<Node>, usize)>
where
    FN: FnMut(&Node) -> IN,
    IN: IntoIterator<Item = (Node, usize)>,
    FH: FnMut(&Node) -> usize,
    FS: FnMut(&Node) -> bool,
{
    let mut to_see = BinaryHeap::new();
    to_see.push(SmallestCostHolder {
        estimated_cost: 0,
        cost: 0,
        index: 0,
    });
    let mut parents: HashMap<Node, (usize, usize)> = HashMap::default();
    parents.insert(start.clone(), (usize::max_value(), 0));
    while let Some(SmallestCostHolder { cost, index, .. }) = to_see.pop() {
        let successors = {
            let (node, &(_, c)) = parents.iter().nth(index).unwrap(); // Cannot fail
            if success(node) {
                let path = reverse_path(&parents, |&(p, _)| p, index);
                return Some((path, cost));
            }
            // We may have inserted a node several time into the binary heap if we found
            // a better way to access it. Ensure that we are currently dealing with the
            // best path and discard the others.
            if cost > c {
                continue;
            }
            successors(node)
        };
        for (successor, move_cost) in successors {
            let new_cost = cost + move_cost;
            let h; // heuristic(&successor)
            let n; // index for successor
            match parents.entry(successor) {
                bevy::utils::hashbrown::hash_map::Entry::Vacant(e) => {
                    h = heuristic(e.key());
                    n = e.index();
                    e.insert((index, new_cost));
                }
                bevy::utils::hashbrown::hash_map::Entry::Occupied(mut e) => {
                    if e.get().1 > new_cost {
                        h = heuristic(e.key());
                        n = e.index();
                        e.insert((index, new_cost));
                    } else {
                        continue;
                    }
                }
            }

            to_see.push(SmallestCostHolder {
                estimated_cost: new_cost + h,
                cost: new_cost,
                index: n,
            });
        }
    }
    None
}
struct SmallestCostHolder<T> {
    estimated_cost: T,
    cost: T,
    index: usize,
}

impl<T: PartialEq> PartialEq for SmallestCostHolder<T> {
    fn eq(&self, other: &Self) -> bool {
        self.estimated_cost.eq(&other.estimated_cost) && self.cost.eq(&other.cost)
    }
}

impl<T: PartialEq> Eq for SmallestCostHolder<T> {}

impl<T: Ord> PartialOrd for SmallestCostHolder<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord> Ord for SmallestCostHolder<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.estimated_cost.cmp(&self.estimated_cost) {
            Ordering::Equal => self.cost.cmp(&other.cost),
            s => s,
        }
    }
}

fn reverse_path<N, V, F>(parents: &HashMap<N, V>, mut parent: F, start: usize) -> Vec<N>
where
    N: Eq + Hash + Clone,
    F: FnMut(&V) -> usize,
{
    let mut i = start;
    let path = std::iter::from_fn(|| {
        parents.get_index(i).map(|(node, value)| {
            i = parent(value);
            node
        })
    })
    .collect::<Vec<&N>>();
    // Collecting the going through the vector is needed to revert the path because the
    // unfold iterator is not double-ended due to its iterative nature.
    path.into_iter().rev().cloned().collect()
}
