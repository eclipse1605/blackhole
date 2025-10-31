mod gl_bindings;

mod camera;
mod shader;
mod renderer;

use renderer::app::App;

fn main() {
	let mut app = App::new();
	app.run();
}
