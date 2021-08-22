pub struct Game {}

impl rts_engine::Game for Game {
    fn init(engine: &mut rts_engine::Engine) -> Self {
        Game {}
    }

    fn resize(&mut self, engine: &mut rts_engine::Engine) {}

    fn update(&mut self, engine: &mut rts_engine::Engine) {}

    fn draw(&mut self, engine: &mut rts_engine::Engine) {}
}
