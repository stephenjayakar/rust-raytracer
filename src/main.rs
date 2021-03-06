/// Imports
extern crate sdl2;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;
use std::f32;
use std::cmp;

/// Types
#[derive(Clone, Copy, Debug)]
struct Point {
    x: f32,
    y: f32,
}
type Vector = Point;
#[derive(Debug)]
struct State {
    position: Point,
    samples: u32,
    direction: f32,
    fov: f32,
    fog: bool,
}

/// Constants
// 75 deg
const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;
const SPEED: f32 = 0.2;
// 1 deg * 2
const ROT_SPEED: f32 = 0.017453292519943 * 2.0;

/// Helper Functions
// TOOD: turn these into macros
fn rad(deg: f32) -> f32 {
    return (deg / 180.0) * f32::consts::PI;
}
fn fmin(a: f32, b:f32) -> f32 {
    if a < b {
        return a;
    } else {
        return b;
    }
}
fn fmax(a: f32, b:f32) -> f32 {
    if a < b {
        return b;
    } else {
        return a;
    }
}
fn angle_to_vec(theta: f32) -> Vector {
    return Vector { x: f32::cos(theta), y: f32::sin(theta) };
}
// Intersection algorithm AABB
// - Returns -1 on failed intersection, otherwise returns distance
// TODO: Consider changing it -> an optional on failed intersection (it'll clean up other areas)
fn intersect(origin: Point, vec: Vector, cube: Point) -> f32 {
    let mut tmin = f32::NEG_INFINITY;
    let mut tmax = f32::INFINITY;
    if vec.x != 0.0 {
        let tx1 = (cube.x - origin.x) / vec.x;
        let tx2 = (cube.x + 1.0 - origin.x) / vec.x;
        tmin = fmax(tmin, fmin(tx1, tx2));
        tmax = fmin(tmax, fmax(tx1, tx2));
    }
    if vec.y != 0.0 {
        let ty1 = (cube.y - origin.y) / vec.y;
        let ty2 = (cube.y + 1.0 - origin.y) / vec.y;
        tmin = fmax(tmin, fmin(ty1, ty2));
        tmax = fmin(tmax, fmax(ty1, ty2));
    }
    if tmax >= tmin && tmin >= 0.0 {
        let dist = f32::sqrt(f32::powf(vec.x * tmin, 2.0) + f32::powf(vec.y * tmin, 2.0));
        return dist;
    } else {
        // TODO: Change this to a constant
        return -1.0;
    }
}
fn gen_map(map: &mut Vec<Point>) {
    map.push(Point{ x: 1.0, y: 1.0 });
    map.push(Point{ x: 3.0, y: 3.0 });
    map.push(Point{ x: 3.0, y: 4.0 });
    map.push(Point{ x: 1.0, y: 6.0 });
    map.push(Point{ x: 3.0, y: 5.0 });
    map.push(Point{ x: 5.0, y: 3.0 });
    map.push(Point{ x: 5.0, y: 4.0 });
    map.push(Point{ x: 5.0, y: 5.0 });
}
// TODO: Why does this still cause minor distortion?
fn distance_to_height(dist: f32, angle: f32) -> f32 {
    return ((HEIGHT) as f32) / (dist * f32::cos(angle));
}
fn draw_rect(canvas: &mut Canvas<Window>, state: &State, x: u32, height: f32, width: u32) {
    if state.fog {
        let alpha = ((1.0 - fmin(1.0, ((height + HEIGHT as f32 / 2.0) / (HEIGHT as f32)))) * 255.0) as u8;
        canvas.set_draw_color(Color::RGB(alpha, alpha, alpha));
    } else {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
    }
    let height = cmp::min(height as u32, HEIGHT);
    let x = x as i32;
    let y = ((HEIGHT / 2) - (height / 2)) as i32;
    //println!("x: {}, y: {}, width: {}, height: {}", x, y, width, height);
    canvas.fill_rect(Rect::new(x, y, width, height));
}
fn render(canvas: &mut Canvas<Window>,
          map: &Vec<Point>,
          state: &State) {
    let State { position, samples, direction, fov, fog } = *state;
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    let mut theta = direction + (fov / 2.0);
    let delta_theta = fov / (samples as f32);
    let width = WIDTH / samples;
    for i in 0..samples {
        let vector = angle_to_vec(theta);
        let mut dist = f32::NEG_INFINITY;
        for cube in map {
            let temp = intersect(position, vector, *cube);
            if temp > 0.0 {
                if dist == f32::NEG_INFINITY {
                    dist = temp;
                } else {
                    dist = fmin(dist, temp);
                }
            }
        }
        if dist > 0.0 {
            let height = distance_to_height(dist, (direction - theta).abs());
            draw_rect(canvas, state, i * width, height, width);
        }
        theta -= delta_theta;
    }
    canvas.present();
}

/// Main
fn main() {
    // Canvas setup
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-raytracer", WIDTH, HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    // Scene setup
    let mut render_flag = true;
    let mut state = State {
      position: Point { x: 0.0, y: 0.0 },
      fov: f32::consts::PI * 0.416,
      samples: 800,
      direction: f32::consts::PI / 4.0,
      fog: true,
    };
    let mut map: Vec<Point> = Vec::new();
    gen_map(&mut map);
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                  Event::KeyDown { keycode: Some(Keycode::Escape), .. } |
                  Event::KeyDown { keycode: Some(Keycode::Q), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                    let vector = angle_to_vec(state.direction);
                    state.position.x += SPEED * vector.x;
                    state.position.y += SPEED * vector.y;
                    render_flag = true;
                },
                Event::KeyDown { keycode: Some(Keycode::A), .. } => {
                    state.direction += ROT_SPEED;
                    render_flag = true;
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    let vector = angle_to_vec(state.direction);
                    state.position.x -= SPEED * vector.x;
                    state.position.y -= SPEED * vector.y;
                    render_flag = true;
                },
                Event::KeyDown { keycode: Some(Keycode::D), .. } => {
                    state.direction -= ROT_SPEED;
                    render_flag = true;
                },
                // Decrease fov
                Event::KeyDown { keycode: Some(Keycode::Num1), .. } => {
                    state.fov -= rad(1.0);
                    render_flag = true;
                },
                // Increase fov
                Event::KeyDown { keycode: Some(Keycode::Num2), .. } => {
                    state.fov += rad(1.0);
                    render_flag = true;
                },
                // Decrease Resolution
                Event::KeyDown { keycode: Some(Keycode::Num3), .. } => {
                    state.samples /= 2;
                    state.samples = cmp::max(1, state.samples);
                    render_flag = true;
                },
                // Increase Resolution
                Event::KeyDown { keycode: Some(Keycode::Num4), .. } => {
                    state.samples *= 2;
                    state.samples = cmp::min(state.samples, WIDTH);
                    render_flag = true;
                },
                // Toggle fog mode
                Event::KeyDown { keycode: Some(Keycode::F), .. } => {
                    state.fog = !state.fog;
                    render_flag = true;
                },
                _ => {}
            }
        }
        if render_flag {
            render(&mut canvas, &map, &state);
            render_flag = false;
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
