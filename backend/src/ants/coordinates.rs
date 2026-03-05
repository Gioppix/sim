use rand::Rng;
use rand::RngExt;
use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[ts(export, export_to = crate::TS_EXPORT_FILE)]
pub struct Coordinates {
    pub x: f64,
    pub y: f64,
}

impl Coordinates {
    pub fn distance(&self, other: &Coordinates) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn move_toward(&self, target: &Coordinates, step: f64) -> Coordinates {
        let dist = self.distance(target);
        if dist <= step {
            return *target;
        }
        let ratio = step / dist;
        Coordinates {
            x: self.x + (target.x - self.x) * ratio,
            y: self.y + (target.y - self.y) * ratio,
        }
    }

    pub fn random(rng: &mut impl Rng, size: (f64, f64)) -> Self {
        Coordinates {
            x: rng.random_range(0.0..size.0),
            y: rng.random_range(0.0..size.1),
        }
    }

    pub fn random_near(
        rng: &mut impl Rng,
        center: Coordinates,
        radius: f64,
        size: (f64, f64),
    ) -> Self {
        let angle = rng.random_range(0.0..std::f64::consts::TAU);
        let r = rng.random_range(0.0..radius);
        let x = (center.x + r * angle.cos()).clamp(0.0, size.0);
        let y = (center.y + r * angle.sin()).clamp(0.0, size.1);
        Coordinates { x, y }
    }
}
