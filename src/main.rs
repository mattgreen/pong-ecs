extern crate legion;
extern crate sdl2;

use std::time::{Duration, Instant};
use std::thread;

use legion::prelude::*;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf::Font;
use sdl2::video::Window;
use sdl2::EventPump;

static GAME_WIDTH: f32 = 800.0;
static GAME_HEIGHT: f32 = 450.0;

static WINDOW_WIDTH: u32 = 800;
static WINDOW_HEIGHT: u32 = 600;

static PADDLE_WIDTH: f32 = 20.0;
static PADDLE_HEIGHT: f32 = 80.0;
static PADDLE_MOVE_SPEED: f32 = 480.0;

static BALL_START_SPEED: f32 = 180.0;
static BALL_SPEED_BOOST: f32 = 12.0;
static BALL_WIDTH: f32 = 20.0;
static BALL_HEIGHT: f32 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ball;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Paddle;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Score {
    p1: u16,
    p2: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity {
    dx: f32,
    dy: f32,
}

enum Collision {
    Bounce(Bounce),
    ScoreWall(Player)
}

enum Bounce {
    Horizontal,
    Vertical
}

enum Player {
    First,
    Second
}

fn accelerate(v: f32, amount: f32) -> f32 {
    if v < 0.0 {
        v - amount
    } else {
        v + amount
    }
}

fn collide(world: &mut World) {
    // Paddle + game area collision handling
    for mut paddle in <Write<Position>>::query().filter(tag_value(&Paddle)).iter(world) {
        if paddle.y < 0.0 {
            paddle.y = 0.0;
        } else if (paddle.y + paddle.height) > GAME_HEIGHT {
            paddle.y = GAME_HEIGHT - paddle.height;
        }
    }

    // Ball + game area/paddle collision handling
    let ball = *<Read<Position>>::query()
        .filter(tag_value(&Ball))
        .iter_immutable(world)
        .nth(0)
        .unwrap();

    let collision = {
        if ball.x < 0.0 {
            Some(Collision::ScoreWall(Player::First))
        } else if (ball.x + ball.width) > GAME_WIDTH {
            Some(Collision::ScoreWall(Player::Second))
        } else {
            if ball.y < 0.0 {
                Some(Collision::Bounce(Bounce::Vertical))
            } else if (ball.y + ball.height) > GAME_HEIGHT {
                Some(Collision::Bounce(Bounce::Vertical))
            } else {
                let mut c = None;

                let ball_bounds = Rect::new(ball.x as i32, ball.y as i32, ball.width as u32, ball.height as u32);
                for paddle in <Read<Position>>::query().filter(tag_value(&Paddle)).iter(world) {
                    let bounds = Rect::new(paddle.x as i32, paddle.y as i32, paddle.width as u32, paddle.height as u32);
                    
                    if ball_bounds.intersection(bounds).is_some() {
                        c = Some(Collision::Bounce(Bounce::Horizontal));
                        break;
                    }
                }

                c
            }
        }
    };

    if let Some(c) = collision {
        match c {
            Collision::ScoreWall(p) => {
                for mut score in <Write<Score>>::query().iter(world) {
                    match p {
                        Player::First => score.p2 += 1,
                        Player::Second => score.p1 += 1,
                    };
                }

                for (mut ball_pos, mut ball_vel) in <(Write<Position>, Write<Velocity>)>::query().filter(tag_value(&Ball)).filter(tag_value(&Ball)).iter(world) {
                    ball_pos.x = 150.0;
                    ball_pos.y = 150.0;
                    ball_vel.dx = BALL_START_SPEED;
                    ball_vel.dy = BALL_START_SPEED * 1.25;
                }
            },
            Collision::Bounce(b) => {
                let mut ball_vel = <Write<Velocity>>::query()
                    .filter(tag_value(&Ball))
                    .iter(world)
                    .nth(0)
                    .unwrap();

                match b {
                    Bounce::Horizontal => { 
                        ball_vel.dx *= -1.0;
                        ball_vel.dx = accelerate(ball_vel.dx, BALL_SPEED_BOOST);
                    },
                    Bounce::Vertical => {
                        ball_vel.dy *= -1.0;
                        ball_vel.dy = accelerate(ball_vel.dy, BALL_SPEED_BOOST);
                    },
                }
            },
        }
    }
}

fn draw(world: &World, font: &Font, canvas: &mut Canvas<Window>, buffer: &mut Vec<u8>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for pos in <Read<Position>>::query().iter_immutable(world) {
        canvas.fill_rect(Rect::new(pos.x as i32, pos.y as i32, pos.width as u32, pos.height as u32)).unwrap();
    }

    canvas.fill_rect(Rect::new(0, GAME_HEIGHT as i32, GAME_WIDTH as u32, 4)).unwrap();

    for score in <Read<Score>>::query().iter_immutable(world) {
        use std::io::Write;
        use std::str;

        buffer.clear();
        write!(buffer, "{:02} - {:02}", score.p1, score.p2).unwrap();
        let s = unsafe { str::from_utf8_unchecked(&buffer[..]) };

        let surface = font.render(s).blended(Color::WHITE).unwrap();
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator.create_texture_from_surface(surface).unwrap();

        let (width, height) = font.size_of(&s).unwrap();
        let target = Rect::new(305, GAME_HEIGHT as i32 + 20, width, height);
        
        canvas.copy(&texture, None, Some(target)).unwrap();
    }

    canvas.present();
}

fn input(world: &mut World, event_pump: &mut EventPump, p1: Entity, p2: Entity) -> bool {
    for e in event_pump.poll_iter() {
        match e {
            Event::Quit { .. } => return true,
            Event::KeyDown {
                keycode: Some(Keycode::S),
                ..
            } => {
                let mut p1_vel = world.get_component_mut::<Velocity>(p1).unwrap();
                p1_vel.dy = PADDLE_MOVE_SPEED;
            }
            Event::KeyUp {
                keycode: Some(Keycode::S),
                ..
            } => {
                let mut p1_vel = world.get_component_mut::<Velocity>(p1).unwrap();
                p1_vel.dy = 0.0;
            }
            Event::KeyDown {
                keycode: Some(Keycode::W),
                ..
            } => {
                let mut p1_vel = world.get_component_mut::<Velocity>(p1).unwrap();
                p1_vel.dy = -PADDLE_MOVE_SPEED;
            }
            Event::KeyUp {
                keycode: Some(Keycode::W),
                ..
            } => {
                let mut p1_vel = world.get_component_mut::<Velocity>(p1).unwrap();
                p1_vel.dy = 0.0;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } => {
                let mut p2_vel = world.get_component_mut::<Velocity>(p2).unwrap();
                p2_vel.dy = PADDLE_MOVE_SPEED;
            }
            Event::KeyUp {
                keycode: Some(Keycode::Down),
                ..
            } => {
                let mut p2_vel = world.get_component_mut::<Velocity>(p2).unwrap();
                p2_vel.dy = 0.0;
            }
            Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } => {
                let mut p2_vel = world.get_component_mut::<Velocity>(p2).unwrap();
                p2_vel.dy = -PADDLE_MOVE_SPEED;
            }
            Event::KeyUp {
                keycode: Some(Keycode::Up),
                ..
            } => {
                let mut p2_vel = world.get_component_mut::<Velocity>(p2).unwrap();
                p2_vel.dy = 0.0;
            }

            _ => {}
        }
    }

    return false;
}

fn tick(world: &mut World, dt: f32) {
    for (vel, mut pos) in <(Read<Velocity>, Write<Position>)>::query().iter(world) {
        pos.x += vel.dx * dt;
        pos.y += vel.dy * dt;
    }
}

fn main() {
    let ctx = sdl2::init().unwrap();
    let ttf_ctx = sdl2::ttf::init().unwrap();
    let font = ttf_ctx.load_font("assets/super-retro-m54.ttf", 48).unwrap();

    let window = ctx
        .video()
        .unwrap()
        .window("Pong", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut text = vec![0 as u8; 0];

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = ctx.event_pump().unwrap();

    let universe = Universe::new();
    let mut world = universe.create_world();

    world.insert(
        (Ball,),
        vec![(
            Position {
                x: 150.0,
                y: 150.0,
                width: BALL_WIDTH,
                height: BALL_HEIGHT,
            },
            Velocity { dx: BALL_START_SPEED, dy: BALL_START_SPEED * 1.25 },
        )],
    )[0];

    let p1 = world.insert(
        (Paddle,),
        vec![(
            Position {
                x: 80.0,
                y: 40.0,
                width: PADDLE_WIDTH,
                height: PADDLE_HEIGHT,
            },
            Velocity { dx: 0.0, dy: 0.0 },
        )],
    )[0];

    let p2 = world.insert(
        (Paddle,),
        vec![(
            Position {
                x: GAME_WIDTH - PADDLE_WIDTH - 100.0,
                y: 40.0,
                width: PADDLE_WIDTH,
                height: PADDLE_HEIGHT,
            },
            Velocity { dx: 0.0, dy: 0.0 },
        )],
    )[0];

    world.insert((), vec![(
        Score { p1: 0, p2: 0},
    )]);

    let time_per_frame = Duration::from_micros(16670);

    loop {
        let start = Instant::now();
        if input(&mut world, &mut event_pump, p1, p2) {
            break;
        }

        tick(&mut world, time_per_frame.as_secs_f32());
        collide(&mut world);
        draw(&world, &font, &mut canvas, &mut text);

        let elapsed = start.elapsed();
        let sleep_time = if elapsed < time_per_frame {
            time_per_frame - elapsed
        } else {
            Duration::from_nanos(0)
        };

        thread::sleep(sleep_time);
    }
}
