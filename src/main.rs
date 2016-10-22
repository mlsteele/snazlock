extern crate pam_auth;
extern crate rpassword;
extern crate users;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate glutin;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use piston::window::{Window,AdvancedWindow};
use glutin_window::GlutinWindow;
use glutin::CursorState;
use opengl_graphics::{ GlGraphics, OpenGL };

const PAM_APP_NAME: &'static str = "snazlock";
const GRAPHICS_APP_NAME: &'static str = "snazlock";

pub fn main() {
    try_graphics();
    std::process::exit(0);
    try_auth();
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64,   // Rotation for the square.
    size: f64,   // Size for the square.
}

impl App {
    fn new(opengl: OpenGL) -> App {
        App {
            gl: GlGraphics::new(opengl),
            rotation: 0.0,
            size: 1.0,
        }
    }
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLUE1: [f32; 4] = [0.76953125, 0.81640625, 0.91796875, 1.0];
        const BLUE2: [f32; 4] = [0.5703125, 0.68359375, 0.83984375, 1.0];

        let size = self.size * 50.0;
        let square = rectangle::square(0.0, 0.0, size);
        let rotation = self.rotation;
        let (x, y) = ((args.width / 2) as f64,
                      (args.height / 2) as f64);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLUE1, gl);

            let transform = c.transform
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-size / 2.0, -size / 2.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(BLUE2, square, transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.rotation += 0.5 * args.dt;
        let scaledown: f64 = 0.2;
        self.size *= scaledown.powf(args.dt as f64);
        self.size = self.size.max(1.0).min(7.0);
    }

    fn kick(&mut self) {
        self.rotation -= 0.1;
        self.size *= 1.2;
    }
}

fn try_graphics() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: GlutinWindow = WindowSettings::new(
            GRAPHICS_APP_NAME,
            [200, 200]
        )
        .fullscreen(false)
        .opengl(opengl)
        .samples(8)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // window.set_capture_cursor(true);
    window.window.set_cursor_state(CursorState::SuperGrab);

    // Create a new game and run it.
    let mut app = App::new(opengl);

    let mut passphrase = String::new();

    let mut events = window.events();
    let mut stayin = true;
    while let Some(e) = events.next(&mut window) {
        e.render(|r| app.render(&r));

        e.update(|u| app.update(u));

        e.press(|b| {
            match b {
                Button::Keyboard(Key::Return) => {
                    passphrase.truncate(0);
                    println!("RET");
                },
                Button::Keyboard(Key::Backspace) => {
                    println!("DEL");
                    let p_len = passphrase.chars().count();
                    let take_n = if p_len > 0 { p_len - 1} else { 0 };
                    passphrase = passphrase.chars().take(take_n).collect();
                },
                Button::Keyboard(Key::Escape) => {
                    // TODO put this behind --unsafe duh.
                    stayin = false;
                },
                _ => {},
            };
        });

        e.text(|s| {
            if !s.is_empty() {
                let c = s.chars().next().unwrap();
                passphrase.push(c);
                app.kick();
            }
        });

        window.set_should_close(false);

        if !stayin {
            return;
        }
    }

    println!("out {:?}", window.should_close());
    println!("left the event loop");
}

fn try_auth() {
    let username = users::get_user_by_uid(users::get_current_uid()).unwrap().name().to_owned();
    println!("Hello, {}!", username);

    println!("password me up >");
    let password = rpassword::read_password().unwrap();
    println!("got it.");

    let mut auth = pam_auth::Authenticator::new(PAM_APP_NAME).unwrap();
    auth.set_credentials(&username, &password);
    if auth.authenticate().is_ok() && auth.open_session().is_ok() {
        println!("Successfully opened a session!");
    }
    else {
        println!("Authentication failed =/");
    }
}
