use bevy::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod scene;
mod cube;
mod input;
mod component;
mod debug_ui;

mod util;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    Load,
    Setup,
    InGame,
    Pause,
}

#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();
    app.add_plugin(scene::ScenePlugin)
        .add_plugin(cube::CubePlugin);

    #[cfg(not(feature = "public"))]
    app.add_plugin(debug_ui::DebugUiPlugin);

    app.run();
}
