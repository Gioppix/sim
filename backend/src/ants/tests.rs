use super::ant::Action;
use super::constants::*;
use super::coordinates::Coordinates;
use super::food::FoodItem;
use super::world::{RestrictedWorld, World, WorldConfig};

#[test]
fn test_world_creation() {
    let world = World::random(WorldConfig::default());
    assert_eq!(world.food.len(), WORLD_DEFAULT_FOOD_COUNT as usize);
    let queens: Vec<_> = world.ants.iter().filter(|a| a.queen).collect();
    assert_eq!(queens.len(), 1);
    let workers: Vec<_> = world.ants.iter().filter(|a| !a.queen).collect();
    assert_eq!(workers.len(), INITIAL_WORKER_COUNT);
}

#[test]
fn test_restricted_world_radius() {
    let mut world = World::random(WorldConfig::default());
    world.food.push(FoodItem {
        position: Coordinates { x: 90.0, y: 90.0 },
        amount: 5.0,
    });
    let center = Coordinates { x: 0.0, y: 0.0 };
    let rw = RestrictedWorld::new(&world, center, 10.0);
    for f in rw.visible_food() {
        assert!(center.distance(&f.position) <= 10.0);
    }
}

#[test]
fn test_pick_and_drop() {
    let mut world = World::random(WorldConfig::default());
    world.food.clear();
    world.food.push(FoodItem {
        position: Coordinates { x: 50.0, y: 50.0 },
        amount: 10.0,
    });
    world.ants[0].position = Coordinates { x: 50.0, y: 50.0 };

    world.execute_action(
        0,
        &Action::PickFood {
            at: Coordinates { x: 50.0, y: 50.0 },
            amount: 5.0,
        },
    );
    assert!(
        world.food[0].amount < 10.0,
        "food should decrease after pick"
    );
    assert!(world.ants[0].food > 0.0, "ant should carry food after pick");

    let food_count_before = world.food.len();
    let carrying = world.ants[0].food;
    world.execute_action(
        0,
        &Action::DropFood {
            at: Coordinates { x: 10.0, y: 10.0 },
            amount: carrying,
        },
    );
    assert!(
        world.food.len() > food_count_before,
        "food item should be created after drop"
    );
}

#[test]
fn test_hatch_requires_queen() {
    let mut world = World::random(WorldConfig::default());
    let initial_count = world.ants.len();
    assert!(!world.ants[0].queen);
    world.execute_action(0, &Action::HatchAnts { count: 1 });
    assert_eq!(world.ants.len(), initial_count);
}

#[test]
fn test_step_runs() {
    let mut world = World::random(WorldConfig::default());
    for _ in 0..100 {
        world.step();
    }
}

#[test]
fn test_move_clamps_to_world() {
    let mut world = World::random(WorldConfig::default());
    let ant_id = world.ants[0].id;
    world.execute_action(ant_id, &Action::Move(Coordinates { x: 200.0, y: 200.0 }));
    let ant = world.ants.iter().find(|a| a.id == ant_id).unwrap();
    assert!(ant.position.x <= world.config.size.0);
    assert!(ant.position.y <= world.config.size.1);
}

#[test]
fn test_queen_hatches_over_time() {
    let mut world = World::random(WorldConfig::default());
    let initial_count = world.ants.len();
    for _ in 0..200 {
        world.step();
    }
    let total = world.ants.len() + world.cemetery.len();
    assert!(
        total > initial_count,
        "queen should have hatched additional ants after 200 steps (total created: {total})"
    );
}
