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
    pub cemetery: Vec<Ant>,
    pub step_count: u32,
    next_id: usize,
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
                food: ANT_START_FOOD,

                steps_since_last_hatch: 0,
                memory: Memory {
                    last_seen_queen: Some(queen_pos),
                    ..Memory::default()
                },
            })
            .collect();

        let queen_id = INITIAL_WORKER_COUNT;
        ants.push(Ant {
            id: queen_id,
            position: queen_pos,
            queen: true,
            food: ANT_HATCH_COST * 5.0,
            steps_since_last_hatch: QUEEN_HATCH_COOLDOWN,
            memory: Memory::default(),
        });

        World {
            config,
            food,
            ants,
            cemetery: Vec::new(),
            step_count: 0,
            next_id: INITIAL_WORKER_COUNT + 1,
        }
    }

    pub fn step(&mut self) {
        let n = self.ants.len();

        let mut decisions: Vec<(usize, Action)> = Vec::with_capacity(n);
        for i in 0..n {
            let rw = RestrictedWorld::new(self, self.ants[i].position, ANT_VISION_RADIUS);
            let action = self.ants[i].decide(&rw);
            decisions.push((self.ants[i].id, action));
        }

        let mut successes = vec![false; n];
        let mut order: Vec<usize> = (0..n).collect();
        order.shuffle(&mut rand::rng());
        for i in order {
            let (ant_id, ref action) = decisions[i];
            successes[i] = self.execute_action(ant_id, action);
        }

        for i in 0..n {
            self.ants[i].update_memory(&decisions[i].1, successes[i]);
        }

        // Phase 5: remove depleted food and dead ants
        self.food.retain(|f| f.amount > 0.0);
        let mut i = 0;
        while i < self.ants.len() {
            if self.ants[i].food <= 0.0 {
                let dead = self.ants.swap_remove(i);
                self.cemetery.push(dead);
            } else {
                i += 1;
            }
        }

        self.step_count += 1;
    }

    pub fn execute_action(&mut self, ant_id: usize, action: &Action) -> bool {
        match action {
            Action::Move(pos) => {
                let pos = *pos;
                if let Some(ant) = self.ants.iter_mut().find(|a| a.id == ant_id) {
                    let clamped = Coordinates {
                        x: pos.x.clamp(0.0, self.config.size.0),
                        y: pos.y.clamp(0.0, self.config.size.1),
                    };
                    let dist = ant.position.distance(&clamped).min(ANT_STEP_SIZE);
                    let actual = ant.position.move_toward(&clamped, dist);
                    ant.food -= dist * STEP_COST;
                    ant.position = actual;
                }
                true
            }

            Action::Eat { at, amount } => {
                let (at, amount) = (*at, *amount);
                let ant_pos = match self.ants.iter().find(|a| a.id == ant_id) {
                    Some(a) => a.position,
                    None => return false,
                };
                if ant_pos.distance(&at) > ANT_STEP_SIZE * 1.5 {
                    return false;
                }
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
                    let eaten = amount.min(food.amount);
                    food.amount -= eaten;
                    if let Some(ant) = self.ants.iter_mut().find(|a| a.id == ant_id) {
                        ant.food += eaten;
                    }
                    true
                } else {
                    false
                }
            }

            Action::PickFood { at, amount } => {
                let (at, amount) = (*at, *amount);
                let ant_pos = match self.ants.iter().find(|a| a.id == ant_id) {
                    Some(a) => a.position,
                    None => return false,
                };
                if ant_pos.distance(&at) > ANT_STEP_SIZE * 1.5 {
                    return false;
                }
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
                        ant.food += picked;
                    }
                    true
                } else {
                    false
                }
            }

            Action::DropFood { at, amount } => {
                let (at, amount) = (*at, *amount);
                if let Some(ant) = self.ants.iter_mut().find(|a| a.id == ant_id) {
                    if ant.position.distance(&at) > ANT_STEP_SIZE * 1.5 {
                        return false;
                    }
                    let dropped = amount.min(ant.food);
                    ant.food -= dropped;
                    self.food.push(FoodItem {
                        position: at,
                        amount: dropped,
                    });
                }
                true
            }

            Action::HatchAnts { count } => {
                let count = *count;
                let queen_idx = match self.ants.iter().position(|a| a.id == ant_id && a.queen) {
                    Some(idx) => idx,
                    None => return false,
                };
                let cost = ANT_HATCH_COST * count as f64;
                if self.ants[queen_idx].food < cost {
                    return false;
                }
                self.ants[queen_idx].food -= cost;
                self.ants[queen_idx].steps_since_last_hatch = 0;
                let queen_pos = self.ants[queen_idx].position;
                let start_id = self.ants.len();
                for i in 0..count {
                    self.ants.push(Ant {
                        id: start_id + i,
                        position: queen_pos,
                        queen: false,
                        food: ANT_START_FOOD,
        
                        steps_since_last_hatch: 0,
                        memory: Memory {
                            last_seen_queen: Some(queen_pos),
                            ..Memory::default()
                        },
                    });
                }
                true
            }

            Action::Rest | Action::Pheromone { .. } => true,
        }
    }
}
