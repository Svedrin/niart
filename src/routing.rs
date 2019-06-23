use specs::prelude::*;

#[derive(Debug)]
pub struct Junction {
    pub connections: Vec<Entity>
}

impl Junction {
    pub fn new() -> Self {
        Self {
            connections: vec![]
        }
    }
}

impl Component for Junction {
    type Storage = VecStorage<Self>;
}
