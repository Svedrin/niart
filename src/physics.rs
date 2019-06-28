use specs::prelude::*;

#[derive(Clone,Debug,PartialEq)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
}

impl Vector {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn length(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn scale_to_length(&self, new_length: f64) -> Self {
        let fac = new_length / self.length();
        Self {
            x: self.x * fac,
            y: self.y * fac
        }
    }
}

#[derive(Clone,Debug,PartialEq)]
pub struct Position {
    pub x: f64,
    pub y: f64
}

impl Position {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn new(x: f64, y: f64) -> Self {
        Self { x: x, y: y }
    }

    pub fn distance_to(&self, other: &Position) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    pub fn distance_length_to(&self, other: &Position) -> f64 {
        self.distance_to(other).length()
    }

    pub fn as_f32_array(&self) -> [f32; 2] {
        [self.x as f32, self.y as f32]
    }

    pub fn as_f64_array(&self) -> [f64; 2] {
        [self.x, self.y]
    }

    pub fn as_f32_tuple(&self) -> (f32, f32) {
        (self.x as f32, self.y as f32)
    }

    pub fn as_f64_tuple(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl From<[f64; 2]> for Position {
    fn from(floats: [f64; 2]) -> Self {
        Self {
            x: floats[0],
            y: floats[1]
        }
    }
}

impl From<[f32; 2]> for Position {
    fn from(floats: [f32; 2]) -> Self {
        Self {
            x: floats[0] as f64,
            y: floats[1] as f64
        }
    }
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}


#[derive(Debug)]
pub struct TrainEngine {
    pub velocity:     Vector,
    pub acceleration: Vector,
    pub vmax: f64,
    pub amax: f64,
}

impl Component for TrainEngine {
    type Storage = HashMapStorage<Self>;
}

pub struct TrainEngineSystem;

impl<'a> System<'a> for TrainEngineSystem {
    type SystemData = (
        WriteStorage<'a, Position>,
        WriteStorage<'a, TrainEngine>,
        Read<'a, super::DeltaTime>,
    );

    fn run(&mut self, (mut positions, mut engines, delta): Self::SystemData) {
        for (position, engine) in (&mut positions, &mut engines).join() {
            engine.velocity.x += engine.acceleration.x * delta.fraction;
            engine.velocity.y += engine.acceleration.y * delta.fraction;

            position.x += engine.velocity.x * delta.fraction;
            position.y += engine.velocity.y * delta.fraction;
        }
    }
}
