use rand::RngExt;

use crate::ants::FoodItem;

use super::constants::*;
use super::coordinates::Coordinates;
use super::world::RestrictedWorld;

const EAT_THRESHOLD: f64 = 3.0;

#[derive(Clone, Debug, Default)]
pub struct Memory {
    pub wandering_direction: Option<f64>,
    pub explore_steps_remaining: u32,
    pub last_seen_food: Option<Coordinates>,
    pub last_seen_queen: Option<Coordinates>,
}

#[derive(Clone, Debug)]
pub enum Action {
    Move(Coordinates),
    Eat { at: Coordinates, amount: f64 },
    PickFood { at: Coordinates, amount: f64 },
    DropFood { at: Coordinates, amount: f64 },
    HatchAnts { count: usize },
    Rest,
    Pheromone { at: Coordinates, strength: f64 },
}

pub struct Ant {
    pub id: usize,
    pub position: Coordinates,
    pub queen: bool,
    pub food: f64,
    pub steps_since_last_hatch: u32,
    pub memory: Memory,
}

impl Ant {
    pub fn decide(&self, world: &RestrictedWorld) -> Action {
        if self.queen {
            self.queen_decide(world)
        } else {
            self.worker_decide(world)
        }
    }

    fn queen_decide(&self, world: &RestrictedWorld) -> Action {
        if self.steps_since_last_hatch >= QUEEN_HATCH_COOLDOWN
            && self.food >= ANT_HATCH_COST + ANT_START_FOOD
        {
            return Action::HatchAnts { count: 1 };
        }

        let visible = world.visible_food();
        if let Some(closest) = closest_food(self.position, &visible) {
            if self.position.distance(&closest.position) <= ANT_STEP_SIZE * 1.5 {
                return Action::PickFood {
                    at: closest.position,
                    amount: MAX_PICK_AMOUNT.min(closest.amount),
                };
            }
        }

        Action::Rest
    }

    fn worker_decide(&self, world: &RestrictedWorld) -> Action {
        // Eat when hungry (any food, including near queen)
        if self.food < EAT_THRESHOLD {
            let all_visible = world.visible_food();
            if let Some(closest) = closest_food(self.position, &all_visible) {
                if self.position.distance(&closest.position) <= ANT_STEP_SIZE * 1.5 {
                    return Action::Eat {
                        at: closest.position,
                        amount: MAX_EAT_AMOUNT.min(closest.amount),
                    };
                }
                return Action::Move(self.position.move_toward(&closest.position, ANT_STEP_SIZE));
            }
        }

        // Deliver surplus food to queen
        if self.food > ANT_START_FOOD {
            if let Some(queen_pos) = self.memory.last_seen_queen {
                if self.position.distance(&queen_pos) <= ANT_STEP_SIZE * 1.5 {
                    return Action::DropFood {
                        at: self.position,
                        amount: self.food - ANT_START_FOOD,
                    };
                }
                return Action::Move(self.position.move_toward(&queen_pos, ANT_STEP_SIZE));
            }
            return self.wander();
        }

        // Forage — ignore food near queen
        let visible: Vec<_> = world
            .visible_food()
            .into_iter()
            .filter(|f| {
                self.memory
                    .last_seen_queen
                    .map_or(true, |q| f.position.distance(&q) > QUEEN_EXCLUSION_RADIUS)
            })
            .collect();
        if let Some(closest) = closest_food(self.position, &visible) {
            if self.position.distance(&closest.position) <= ANT_STEP_SIZE * 1.5 {
                return Action::PickFood {
                    at: closest.position,
                    amount: MAX_PICK_AMOUNT.min(closest.amount),
                };
            }
            return Action::Move(self.position.move_toward(&closest.position, ANT_STEP_SIZE));
        }

        // Navigate to remembered food
        if let Some(food_pos) = self.memory.last_seen_food {
            if self.position.distance(&food_pos) > ANT_STEP_SIZE * 1.5 {
                return Action::Move(self.position.move_toward(&food_pos, ANT_STEP_SIZE));
            }
        }

        self.wander()
    }

    fn wander(&self) -> Action {
        let dir = self.memory.wandering_direction.unwrap_or(0.0);
        Action::Move(Coordinates {
            x: self.position.x + dir.cos() * ANT_STEP_SIZE,
            y: self.position.y + dir.sin() * ANT_STEP_SIZE,
        })
    }

    pub fn update_memory(&mut self, action: &Action, success: bool) {
        match action {
            Action::PickFood { at, .. } => {
                self.memory.last_seen_food = if success { Some(*at) } else { None };
            }
            Action::Eat { at, .. } if success => {
                self.memory.last_seen_food = Some(*at);
            }
            _ => {}
        }

        // Tick wander direction regardless of action; the ant reads it whenever it wanders.
        if self.memory.explore_steps_remaining == 0 {
            let mut rng = rand::rng();
            self.memory.wandering_direction = Some(rng.random_range(0.0..std::f64::consts::TAU));
            self.memory.explore_steps_remaining = rng.random_range(5u32..15u32);
        } else {
            self.memory.explore_steps_remaining =
                self.memory.explore_steps_remaining.saturating_sub(1);
        }

        if !matches!(action, Action::HatchAnts { .. }) {
            self.steps_since_last_hatch = self.steps_since_last_hatch.saturating_add(1);
        }
    }
}

fn closest_food<'a>(pos: Coordinates, food: &[&'a FoodItem]) -> Option<&'a FoodItem> {
    food.iter()
        .min_by(|a, b| {
            pos.distance(&a.position)
                .partial_cmp(&pos.distance(&b.position))
                .unwrap()
        })
        .copied()
}
