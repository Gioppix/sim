use rand::RngExt;

use super::constants::*;
use super::coordinates::Coordinates;
use super::world::RestrictedWorld;

#[derive(Clone, Debug, Default)]
pub struct Memory {
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
    pub carrying_food: f64,
    pub direction: f64,
    pub steps_since_last_hatch: u32,
    pub memory: Memory,
}

impl Ant {
    pub fn decide(&mut self, world: &RestrictedWorld) -> Action {
        if self.queen {
            self.queen_decide(world)
        } else {
            self.worker_decide(world)
        }
    }

    fn queen_decide(&mut self, world: &RestrictedWorld) -> Action {
        if self.steps_since_last_hatch >= QUEEN_HATCH_COOLDOWN
            && self.carrying_food >= ANT_HATCH_COST
        {
            return Action::HatchAnts { count: 1 };
        }
        self.steps_since_last_hatch = self.steps_since_last_hatch.saturating_add(1);

        // Pick up food dropped nearby by workers
        let visible = world.visible_food();
        if !visible.is_empty() {
            let pos = self.position;
            let closest = visible
                .iter()
                .min_by(|a, b| {
                    pos.distance(&a.position)
                        .partial_cmp(&pos.distance(&b.position))
                        .unwrap()
                })
                .unwrap();
            if self.position.distance(&closest.position) <= ANT_STEP_SIZE * 1.5 {
                return Action::PickFood {
                    at: closest.position,
                    amount: MAX_PICK_AMOUNT.min(closest.amount),
                };
            }
        }

        // Stay put — workers come to her
        Action::Rest
    }

    fn worker_decide(&mut self, world: &RestrictedWorld) -> Action {
        // Always update queen location when visible
        if let Some(queen) = world.visible_ants().iter().find(|a| a.queen) {
            self.memory.last_seen_queen = Some(queen.position);
        }

        if self.carrying_food > 0.0 {
            // Return food to queen
            if let Some(queen_pos) = self.memory.last_seen_queen {
                if self.position.distance(&queen_pos) <= ANT_STEP_SIZE * 1.5 {
                    return Action::DropFood {
                        at: self.position,
                        amount: self.carrying_food,
                    };
                }
                return Action::Move(self.position.move_toward(&queen_pos, ANT_STEP_SIZE));
            }
            // No queen memory yet — wander until we spot her
            return self.wander_move();
        }

        // Forage — ignore food near the queen (it's for her to pick up)
        let visible: Vec<_> = world
            .visible_food()
            .into_iter()
            .filter(|f| {
                self.memory
                    .last_seen_queen
                    .map_or(true, |q| f.position.distance(&q) > QUEEN_EXCLUSION_RADIUS)
            })
            .collect();
        if !visible.is_empty() {
            let pos = self.position;
            let closest = visible
                .iter()
                .min_by(|a, b| {
                    pos.distance(&a.position)
                        .partial_cmp(&pos.distance(&b.position))
                        .unwrap()
                })
                .unwrap();
            self.memory.last_seen_food = Some(closest.position);
            if self.position.distance(&closest.position) <= ANT_STEP_SIZE * 1.5 {
                return Action::PickFood {
                    at: closest.position,
                    amount: MAX_PICK_AMOUNT.min(closest.amount),
                };
            }
            return Action::Move(self.position.move_toward(&closest.position, ANT_STEP_SIZE));
        }

        if let Some(food_pos) = self.memory.last_seen_food {
            let near_queen = self
                .memory
                .last_seen_queen
                .map_or(false, |q| food_pos.distance(&q) <= QUEEN_EXCLUSION_RADIUS);
            if near_queen || self.position.distance(&food_pos) <= ANT_STEP_SIZE * 1.5 {
                self.memory.last_seen_food = None;
            } else {
                return Action::Move(self.position.move_toward(&food_pos, ANT_STEP_SIZE));
            }
        }

        self.wander_move()
    }

    fn wander_move(&mut self) -> Action {
        let mut rng = rand::rng();
        if self.memory.explore_steps_remaining == 0 {
            self.direction = rng.random_range(0.0..std::f64::consts::TAU);
            self.memory.explore_steps_remaining = rng.random_range(5u32..15u32);
        }
        self.memory.explore_steps_remaining -= 1;
        let dx = self.direction.cos() * ANT_STEP_SIZE;
        let dy = self.direction.sin() * ANT_STEP_SIZE;
        Action::Move(Coordinates {
            x: self.position.x + dx,
            y: self.position.y + dy,
        })
    }
}
