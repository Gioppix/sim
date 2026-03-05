use rand::seq::SliceRandom;

use super::ant::{Action, Ant, Memory};
use super::constants::*;
use super::coordinates::Coordinates;
use super::food::FoodItem;

pub struct WorldConfig {
    pub size: (f64, f64),
    pub food_count: f64,
}

impl Default for WorldConfig {
    fn default() -> Self {
        WorldConfig {
            size: WORLD_DEFAULT_SIZE,
            food_count: WORLD_DEFAULT_FOOD_COUNT,
        }
    }
}

pub struct RestrictedWorld<'a> {
    world: &'a World,
    pub center: Coordinates,
    pub radius: f64,
}

impl<'a> RestrictedWorld<'a> {
    pub fn new(world: &'a World, center: Coordinates, radius: f64) -> Self {
        RestrictedWorld {
            world,
            center,
            radius,
        }
    }

    pub fn visible_food(&self) -> Vec<&FoodItem> {
        self.world
            .food
            .iter()
            .filter(|f| self.center.distance(&f.position) <= self.radius)
            .collect()
    }

    pub fn visible_ants(&self) -> Vec<&Ant> {
        self.world
            .ants
            .iter()
            .filter(|a| self.center.distance(&a.position) <= self.radius)
            .collect()
    }
}

pub struct World {
    pub config: WorldConfig,
    pub food: Vec<FoodItem>,
    pub ants: Vec<Ant>,
    pub step_count: u32,
}

impl World {
    pub fn random(config: WorldConfig) -> Self {
        let mut rng = rand::rng();

        let clusters: Vec<Coordinates> = (0..FOOD_CLUSTER_COUNT)
            .map(|_| Coordinates::random(&mut rng, config.size))
            .collect();

        let total = config.food_count as usize;
        let food: Vec<FoodItem> = (0..total)
            .map(|i| {
                let cluster = clusters[i % FOOD_CLUSTER_COUNT];
                FoodItem {
                    position: Coordinates::random_near(
                        &mut rng,
                        cluster,
                        CLUSTER_RADIUS,
                        config.size,
                    ),
                    amount: 1.0,
                }
            })
            .collect();

        let queen_pos = Coordinates {
            x: config.size.0 / 2.0,
            y: config.size.1 / 2.0,
        };

        let mut ants: Vec<Ant> = (0..INITIAL_WORKER_COUNT)
            .map(|id| Ant {
                id,
                position: Coordinates::random(&mut rng, config.size),
                queen: false,
                carrying_food: 0.0,
                direction: 0.0,
                steps_since_last_hatch: 0,
                memory: Memory {
                    last_seen_queen: Some(queen_pos),
                    ..Memory::default()
                },
            })
            .collect();

        let queen_id = ants.len();
        ants.push(Ant {
            id: queen_id,
            position: queen_pos,
            queen: true,
            carrying_food: ANT_HATCH_COST * 5.0,
            direction: 0.0,
            steps_since_last_hatch: QUEEN_HATCH_COOLDOWN,
            memory: Memory::default(),
        });

        World {
            config,
            food,
            ants,
            step_count: 0,
        }
    }

    pub fn step(&mut self) {
        let n = self.ants.len();

        // Snapshot mutable-during-decide state so we can use &self for RestrictedWorld
        struct DecideState {
            memory: Memory,
            direction: f64,
            steps_since_last_hatch: u32,
        }
        let mut states: Vec<DecideState> = self
            .ants
            .iter()
            .map(|a| DecideState {
                memory: a.memory.clone(),
                direction: a.direction,
                steps_since_last_hatch: a.steps_since_last_hatch,
            })
            .collect();

        let mut actions: Vec<(usize, Action)> = Vec::new();

        for i in 0..n {
            let rw = RestrictedWorld::new(&*self, self.ants[i].position, ANT_VISION_RADIUS);

            let mut temp = Ant {
                id: self.ants[i].id,
                position: self.ants[i].position,
                queen: self.ants[i].queen,
                carrying_food: self.ants[i].carrying_food,
                direction: states[i].direction,
                steps_since_last_hatch: states[i].steps_since_last_hatch,
                memory: states[i].memory.clone(),
            };

            let action = temp.decide(&rw);
            states[i].memory = temp.memory;
            states[i].direction = temp.direction;
            states[i].steps_since_last_hatch = temp.steps_since_last_hatch;
            actions.push((temp.id, action));
        }

        for i in 0..n {
            self.ants[i].memory = states[i].memory.clone();
            self.ants[i].direction = states[i].direction;
            self.ants[i].steps_since_last_hatch = states[i].steps_since_last_hatch;
        }

        let mut rng = rand::rng();
        actions.shuffle(&mut rng);

        for (ant_id, action) in actions {
            self.execute_action(ant_id, action);
        }

        self.food.retain(|f| f.amount > 0.0);
        self.step_count += 1;
    }

    pub fn execute_action(&mut self, ant_id: usize, action: Action) {
        match action {
            Action::Move(pos) => {
                if let Some(ant) = self.ants.iter_mut().find(|a| a.id == ant_id) {
                    ant.position.x = pos.x.clamp(0.0, self.config.size.0);
                    ant.position.y = pos.y.clamp(0.0, self.config.size.1);
                }
            }

            Action::Eat { at, amount } => {
                if let Some(food) = self
                    .food
                    .iter_mut()
                    .filter(|f| f.position.distance(&at) <= 2.0)
                    .min_by(|a, b| {
                        a.position
                            .distance(&at)
                            .partial_cmp(&b.position.distance(&at))
                            .unwrap()
                    })
                {
                    food.amount -= amount.min(food.amount);
                }
            }

            Action::PickFood { at, amount } => {
                if let Some(food) = self
                    .food
                    .iter_mut()
                    .filter(|f| f.position.distance(&at) <= 2.0)
                    .min_by(|a, b| {
                        a.position
                            .distance(&at)
                            .partial_cmp(&b.position.distance(&at))
                            .unwrap()
                    })
                {
                    let picked = amount.min(food.amount);
                    food.amount -= picked;
                    if let Some(ant) = self.ants.iter_mut().find(|a| a.id == ant_id) {
                        ant.carrying_food += picked;
                    }
                }
            }

            Action::DropFood { at, amount } => {
                if let Some(ant) = self.ants.iter_mut().find(|a| a.id == ant_id) {
                    let dropped = amount.min(ant.carrying_food);
                    ant.carrying_food -= dropped;
                    self.food.push(FoodItem {
                        position: at,
                        amount: dropped,
                    });
                }
            }

            Action::HatchAnts { count } => {
                let queen_idx = match self.ants.iter().position(|a| a.id == ant_id && a.queen) {
                    Some(idx) => idx,
                    None => return,
                };
                let cost = ANT_HATCH_COST * count as f64;
                if self.ants[queen_idx].carrying_food < cost {
                    return;
                }
                self.ants[queen_idx].carrying_food -= cost;
                self.ants[queen_idx].steps_since_last_hatch = 0;
                let queen_pos = self.ants[queen_idx].position;
                let start_id = self.ants.len();
                for i in 0..count {
                    self.ants.push(Ant {
                        id: start_id + i,
                        position: queen_pos,
                        queen: false,
                        carrying_food: 0.0,
                        direction: 0.0,
                        steps_since_last_hatch: 0,
                        memory: Memory {
                            last_seen_queen: Some(queen_pos),
                            ..Memory::default()
                        },
                    });
                }
            }

            Action::Rest | Action::Pheromone { .. } => {}
        }
    }
}
