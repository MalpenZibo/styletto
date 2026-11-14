mod chrome;
mod stories;
mod system;

use guido::prelude::*;

use chrome::shell;

// Services reach D-Bus, which wants a runtime with IO: Guido's own has
// only timers.
#[tokio::main]
async fn main() {
    App::new().run(|app| {
        app.add_surface(
            SurfaceConfig::new()
                .anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT)
                .layer(Layer::Top)
                .keyboard_interactivity(KeyboardInteractivity::OnDemand)
                .margin(Margin::all(shell::MARGIN))
                .namespace("styletto-storybook")
                .background_color(Color::TRANSPARENT),
            shell::view,
        );
    });
}
