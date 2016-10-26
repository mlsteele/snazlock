extern crate rand;
extern crate num;
extern crate pam_auth;
extern crate rpassword;
extern crate users;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate glutin;
extern crate opengl_graphics;

use std::vec::Vec;
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use piston::window::{Window};
use glutin_window::GlutinWindow;
use glutin::CursorState;
use opengl_graphics::{ GlGraphics, OpenGL };
use rand::Rng;
use num::traits::Float;

const PAM_APP_NAME: &'static str = "snazlock";
const GRAPHICS_APP_NAME: &'static str = "snazlock";

pub fn main() {
    App::main();
}

#[derive(Copy, Clone)]
struct Tendril {
    start_angle: f64,
    curl: Smooth<f64>,
}

impl Tendril {
    fn curl(&mut self, delta: f64) {
        let x = self.curl.target() + delta;
        self.curl.set(x);
    }
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    tendrils: Vec<Tendril>,
}

impl App {
    fn main() {
        let opengl = OpenGL::V3_2;

        // Create an Glutin window.
        let mut window: GlutinWindow = WindowSettings::new(
                GRAPHICS_APP_NAME,
                [200, 200]
            )
            .opengl(opengl)
            .samples(8)
            .fullscreen(true)
            .exit_on_esc(true)
            .build()
            .unwrap();

        // window.set_capture_cursor(true);
        window.window.set_cursor_state(CursorState::SuperGrab).unwrap();

        // Create a new game and run it.
        let mut app = App{
            gl: GlGraphics::new(opengl),
            tendrils: Vec::new(),
        };

        let ntendrils = 23;
        for i in 0..ntendrils {
            let j = i as f64;
            app.tendrils.push(Tendril{
                start_angle: j * deg_to_rad(360.0 / (ntendrils as f64)),
                curl: Smooth::new((rand::thread_rng().next_f64() - 0.5) * 2.0),
            });
        }

        let mut passphrase = String::new();

        let mut events = window.events();
        while let Some(e) = events.next(&mut window) {
            e.render(|r| app.render(&r));

            e.update(|u| app.update(u));

            e.press(|b| {
                match b {
                    Button::Keyboard(Key::Return) => {
                        if passphrase.chars().count() > 0 {
                            if let Ok(true) = pam_authenticate(PAM_APP_NAME, None, &passphrase) {
                                window.set_should_close(true);
                            }
                        }
                        passphrase.truncate(0);
                        app.reset_tendrils();
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
                        window.set_should_close(true);
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
        }

        println!("out {:?}", window.should_close());
        println!("left the event loop");
    }

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        #[allow(unused_variables,dead_code)]
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLUE1: [f32; 4] = [0.76953125, 0.81640625, 0.91796875, 1.0];
        const BLUE2: [f32; 4] = [0.5703125, 0.68359375, 0.83984375, 1.0];

        let (cx, cy) = ((args.width / 2) as f64,
                        (args.height / 2) as f64);

        let unit = rectangle::centered(rectangle::square(0.0, 0.0, 1.0));

        let tendrils = self.tendrils.clone();
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLUE1, gl);

            let dot = ellipse::Ellipse{
                color: BLUE2,
                border: None,
                resolution: 16,
            };
            // Draw tendrils
            for &t in tendrils.iter() {
                let mut t1 = c.transform
                    .trans(cx, cy)
                    .rot_rad(t.start_angle);
                for _ in 1..20 {
                    let t2 = t1.zoom(12.0);
                    dot.draw(unit, &Default::default(), t2, gl);
                    t1 = t1
                        .trans(40.0, 0.0)
                        .rot_deg(t.curl.val())
                        .zoom(0.96);
                }
            }
        });
    }

    fn update(&mut self, _: &UpdateArgs) {
        for t in &mut self.tendrils {
            t.curl.tick();
        }
    }

    fn kick(&mut self) {
        let n_affect: usize = rand::thread_rng().gen_range(1, 4);
        for _ in 0..n_affect {
            let i: usize = rand::thread_rng().gen_range(0, self.tendrils.len());
            let t: &mut Tendril = &mut self.tendrils[i];
            Self::tweak(t,deg_to_rad(150.0), deg_to_rad(600.0));
        }
        for t in &mut self.tendrils {
            Self::tweak(t,deg_to_rad(0.0), deg_to_rad(80.0));
        }
    }

    fn tweak(t: &mut Tendril, min: f64, max: f64) {
        let mut delta = rand::thread_rng().gen_range(min, max);
        if rand::thread_rng().gen() {
            delta *= -1.0;
        }
        t.curl(delta);
    }

    fn reset_tendrils(&mut self) {
        for t in &mut self.tendrils {
            t.curl.set((rand::thread_rng().next_f64() - 0.5) * 2.0);
        }
    }
}

// Authenticate using a username and password.
// If username is not supplied, it is assumed to be the current uid user.
// Returns Ok(true) if successfully authenticated.
fn pam_authenticate(app_name: &'static str, username: Option<&str>, password: &str) -> Result<bool, String> {
    let username = match username {
        Some(username) => username.to_owned(),
        None => {
            let user = users::get_user_by_uid(users::get_current_uid());
            let user = try!(user.ok_or("error getting username"));
            user.name().to_owned()
        },
    };
    let authenticator = pam_auth::Authenticator::new(app_name);
    let mut authenticator = try!(authenticator.ok_or("error making authenticator"));
    authenticator.set_credentials(&username, &password);
    Ok(authenticator.authenticate().is_ok() && authenticator.open_session().is_ok())
}

#[allow(dead_code)]
fn pam_authenticate_example() {
    println!("Hello!");

    println!("password me up >");
    let password = rpassword::read_password().unwrap();
    println!("got it.");

    let authed: bool = pam_authenticate(PAM_APP_NAME, None, &password).unwrap_or(false);
    if authed {
        println!("Successfully opened a session!");
    } else {
        println!("Authentication failed =/");
    }
}

#[allow(dead_code)]
fn auth_pam_example() {
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

#[inline(always)]
fn deg_to_rad(deg: f64) -> f64 {
    deg / 180.0 * ::std::f64::consts::PI
}

#[derive(Copy, Clone)]
struct Smooth<T : Float> {
    current: T,
    target: T,
}

impl Smooth<f64> {
    pub fn new(val: f64) -> Smooth<f64> {
        Smooth{
            current: val,
            target: val,
        }
    }

    pub fn set(&mut self, target: f64) {
        self.target = target;
    }

    pub fn tick(&mut self) {
        self.current = self.current + (0.08 * (self.target - self.current));
    }

    pub fn val(&self) -> f64 {
        self.current
    }

    pub fn target(&self) -> f64 {
        self.target
    }
}
