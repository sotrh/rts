use rts_engine::Settings;

fn main() -> anyhow::Result<()> {
    let settings = Settings {
        resolution: (800, 600),
        fullscreen: false,
    };
    pollster::block_on(rts_engine::run::<rts::Game>(settings))
}
