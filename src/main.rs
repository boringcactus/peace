use amethyst::{
    assets::{AssetStorage, Loader},
    audio::AudioBundle,
    core::transform::TransformBundle,
    input::{is_close_requested, is_key_down, VirtualKeyCode, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        RenderingBundle,
    },
    ui::{RenderUi, UiBundle},
    utils::application_root_dir,
};
use std::time::{Instant, Duration};

mod audio;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let resources = app_root.join("resources");
    let display_config = resources.join("display_config.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(AudioBundle::default())?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config)
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderUi::default())
                .with_plugin(RenderFlat2D::default())
        )?;

    let mut game = Application::new(resources, MyState::default(), game_data)?;
    game.run();

    Ok(())
}

#[derive(Default)]
pub struct MyState {
    exit: Option<Instant>,
}

impl State<GameData<'static, 'static>, StateEvent> for MyState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        initialise_help(world);
        audio::initialise_audio(world);
    }

    fn handle_event<'a>(
        &mut self,
        data: StateData<'a, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            // Check if the window should be closed
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                use amethyst::{
                    audio::{output::Output, Source},
                    ecs::{Read, ReadExpect},
                };
                use crate::audio::Sounds;
                use std::ops::Deref;
                let world = data.world;
                type SystemData<'a> = (
                    Read<'a, AssetStorage<Source>>,
                    ReadExpect<'a, Sounds>,
                    Option<Read<'a, Output>>,
                );
                let (storage, sounds, audio_output): SystemData<'a> = world.system_data();
                audio::play_exit_sound(&*sounds, &storage, audio_output.as_ref().map(|o| o.deref()));
                self.exit = Some(Instant::now() + Duration::from_secs(2));
            }
        }

        // Keep going
        Trans::None
    }

    fn update(
        &mut self,
        data: StateData<'_, GameData<'_, '_>>
    ) -> SimpleTrans {
        if let Some(exit) = self.exit {
            let now = Instant::now();
            if now > exit {
                return Trans::Quit;
            }
        }

        data.data.update(&data.world);
        Trans::None
    }
}

fn initialise_help(world: &mut World) {
    use amethyst::ui::{Anchor, TtfFormat, UiText, UiTransform};
    let font = world.read_resource::<Loader>().load(
        "m5x7.ttf",
        TtfFormat,
        (),
        &world.read_resource(),
    );
    let help_transform = UiTransform::new(
        "Help".to_string(), Anchor::Middle, Anchor::Middle,
        0., 0., 1., 500., 500.,
    );

    let help = world
        .create_entity()
        .with(help_transform)
        .with(UiText::new(
            font.clone(),
            "Press Esc to exit.".to_string(),
            [1., 1., 1., 1.],
            50.,
        )).build();
}
