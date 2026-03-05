use serde::Serialize;
use ts_rs::TS;

use super::coordinates::Coordinates;
use super::world::World;

#[derive(Serialize, TS)]
#[ts(export, export_to = crate::TS_EXPORT_FILE)]
pub struct WorldMetadata {
    pub width: f64,
    pub height: f64,
}

pub fn metadata(world: &World) -> WorldMetadata {
    WorldMetadata {
        width: world.config.size.0,
        height: world.config.size.1,
    }
}

#[derive(Serialize, TS)]
#[ts(export, export_to = crate::TS_EXPORT_FILE)]
pub struct AntSnapshot {
    pub id: usize,
    pub position: Coordinates,
    pub queen: bool,
    pub food: f64,
}

#[derive(Serialize, TS)]
#[ts(export, export_to = crate::TS_EXPORT_FILE)]
pub struct FoodSnapshot {
    pub position: Coordinates,
    pub amount: f64,
}

#[derive(Serialize, TS)]
#[ts(export, export_to = crate::TS_EXPORT_FILE)]
pub struct WorldSnapshot {
    pub step_count: u32,
    pub ants: Vec<AntSnapshot>,
    pub food: Vec<FoodSnapshot>,
}

pub fn snapshot(world: &World) -> WorldSnapshot {
    WorldSnapshot {
        step_count: world.step_count,
        ants: world
            .ants
            .iter()
            .map(|a| AntSnapshot {
                id: a.id,
                position: a.position,
                queen: a.queen,
                food: a.food,
            })
            .collect(),
        food: world
            .food
            .iter()
            .map(|f| FoodSnapshot {
                position: f.position,
                amount: f.amount,
            })
            .collect(),
    }
}
