use crate::ants::{World, WorldConfig};

#[derive(Debug, Clone)]
pub struct SimStats {
    pub ant_count: usize,
    pub queen_count: usize,
    pub remaining_food: f64,
}

pub fn simulate(mut world: World, steps: u32) -> SimStats {
    for _ in 0..steps {
        world.step();
    }

    SimStats {
        ant_count: world.ants.len(),
        queen_count: world.ants.iter().filter(|a| a.queen).count(),
        remaining_food: world.food.iter().map(|f| f.amount).sum(),
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
}
