use crate::ants::{World, WorldConfig};

#[derive(Debug, Clone)]
pub struct SimStats {
    pub ant_count: usize,
    pub queen_count: usize,
    pub remaining_food: f64,
    pub dead_count: usize,
}

pub fn simulate(mut world: World, steps: u32) -> SimStats {
    for _ in 0..steps {
        world.step();
    }

    SimStats {
        ant_count: world.ants.len(),
        queen_count: world.ants.iter().filter(|a| a.queen).count(),
        remaining_food: world.food.iter().map(|f| f.amount).sum(),
        dead_count: world.cemetery.len(),
    }
}

pub fn simulate_default(steps: u32) -> SimStats {
    simulate(World::random(WorldConfig::default()), steps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_default_1000_steps() {
        let stats = simulate_default(1000);
        println!("{stats:?}");
    }

    #[test]
    fn eval_trace_early() {
        use crate::ants::{World, WorldConfig};
        let mut world = World::random(WorldConfig::default());
        for step in 1..=300 {
            world.step();
            let queen = world.ants.iter().find(|a| a.queen);
            if step % 10 == 0 || queen.is_none() {
                println!(
                    "step {step:3}: ants={} dead={} queen_food={:.1}",
                    world.ants.len(),
                    world.cemetery.len(),
                    queen.map(|q| q.food).unwrap_or(-1.0),
                );
            }
            if queen.is_none() {
                break;
            }
        }
    }
}
