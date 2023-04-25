mod camera;
mod engine;
mod instances;
mod math;
mod minor_types;
mod object_data;
pub mod prelude;
mod texture;

use prelude::*;

pub async fn run() {
    let event_loop = EventLoop::new();
    let mut state = Engine::new(vec2(700., 700.), &event_loop).await;
    state.set_fps(Some(144));

    let manager = StateManager::new(vec![]);
    state.enter_loop(manager, event_loop);
}

struct StateManager;
impl Manager for StateManager {
    fn new(_textures: Vec<Texture>) -> Self {
        Self {}
    }
    fn update(&mut self, _frame_time: f64, _input: &Input) {}
    fn render(&self, _state: &mut Engine) {}
}
