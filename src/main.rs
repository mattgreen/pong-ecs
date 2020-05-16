extern crate legion;
extern crate sdl2;

use legion::prelude::*;

use sdl2::event::Event;
use sdl2::gfx::framerate::FPSManager;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::surface::Surface;
use sdl2::ttf::Font;
use sdl2::video::Window;
use sdl2::EventPump;

static WIDTH: i32 = 800;
static HEIGHT: i32 = 600;

static PADDLE_WIDTH: i32 = 20;
static PADDLE_HEIGHT: i32 = 80;
static PADDLE_MOVE_SPEED: i32 = 8;

static BALL_WIDTH: i32 = 20;
static BALL_HEIGHT: i32 = 20;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Ball;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Paddle;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Score {
    p1: u16,
    p2: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity {
    dx: i32,
    dy: i32,
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

fn collide(world: &mut World) {
    // Paddle + game area collision handling
    for mut paddle in <Write<Position>>::query().filter(tag_value(&Paddle)).iter(world) {
        if paddle.y < 0 {
            paddle.y = 0;
        } else if (paddle.y + paddle.height) > HEIGHT {
            paddle.y = HEIGHT - paddle.height;
        }
    }

    // Ball + game area/paddle collision handling
    let ball = *<Read<Position>>::query()
        .filter(tag_value(&Ball))
        .iter_immutable(world)
        .nth(0)
        .unwrap();

    let collision = {
        if ball.x < 0 {
            Some(Collision::ScoreWall(Player::First))
        } else if (ball.x + ball.width) > WIDTH {
            Some(Collision::ScoreWall(Player::Second))
        } else {
            if ball.y < 0 {
                Some(Collision::Bounce(Bounce::Vertical))
            } else if (ball.y + ball.height) > HEIGHT {
                Some(Collision::Bounce(Bounce::Vertical))
            } else {
                let mut c = None;

                let ball_bounds = Rect::new(ball.x, ball.y, ball.width as u32, ball.height as u32);
                for paddle in <Read<Position>>::query().filter(tag_value(&Paddle)).iter(world) {
                    let bounds = Rect::new(paddle.x, paddle.y, paddle.width as u32, paddle.height as u32);
                    
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
                    ball_pos.x = 150;
                    ball_pos.y = 150;
                    ball_vel.dx = 3;
                    ball_vel.dy = 4;
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
                        ball_vel.dx *= -1
                    },
                    Bounce::Vertical => {
                        ball_vel.dy *= -1
                    },
                }
            },
        }
    }
}

fn draw(world: &World, font: &Font, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for pos in <Read<Position>>::query().iter_immutable(world) {
        canvas.fill_rect(Rect::new(pos.x, pos.y, pos.width as u32, pos.height as u32));
    }

    for score in <Read<Score>>::query().iter_immutable(world) {
        let s = format!("{} - {}", score.p1, score.p2);
        let surface = font.render(&s).blended(Color::WHITE).unwrap();
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator.create_texture_from_surface(surface).unwrap();

        let (width, height) = font.size_of(&s).unwrap();
        let target = Rect::new(325, 0, width, height);

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
                p1_vel.dy = 0;
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
                p1_vel.dy = 0;
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
                p2_vel.dy = 0;
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
                p2_vel.dy = 0;
            }

            _ => {}
        }
    }

    return false;
}

fn tick(world: &mut World) {
    let query = <(Read<Velocity>, Write<Position>)>::query();
    for (vel, mut pos) in query.iter(world) {
        pos.x += vel.dx;
        pos.y += vel.dy;
    }
}

fn main() {
    let ctx = sdl2::init().unwrap();
    let ttf_ctx = sdl2::ttf::init().unwrap();
    let font = ttf_ctx.load_font("assets/super-retro-m54.ttf", 48).unwrap();

    let window = ctx
        .video()
        .unwrap()
        .window("Pong", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = ctx.event_pump().unwrap();

    let universe = Universe::new();
    let mut world = universe.create_world();

    world.insert(
        (Ball,),
        vec![(
            Position {
                x: 150,
                y: 150,
                width: BALL_WIDTH,
                height: BALL_HEIGHT,
            },
            Velocity { dx: 3, dy: 2 },
        )],
    )[0];

    let p1 = world.insert(
        (Paddle,),
        vec![(
            Position {
                x: 80,
                y: 40,
                width: PADDLE_WIDTH,
                height: PADDLE_HEIGHT,
            },
            Velocity { dx: 0, dy: 0 },
        )],
    )[0];

    let p2 = world.insert(
        (Paddle,),
        vec![(
            Position {
                x: WIDTH - PADDLE_WIDTH - 80,
                y: 40,
                width: PADDLE_WIDTH,
                height: PADDLE_HEIGHT,
            },
            Velocity { dx: 0, dy: 0 },
        )],
    )[0];

    world.insert((), vec![(
        Score { p1: 0, p2: 0},
    )]);

    let mut fps_manager = FPSManager::new();
    fps_manager.set_framerate(60);

    loop {
        if input(&mut world, &mut event_pump, p1, p2) {
            break;
        }

        tick(&mut world);
        collide(&mut world);
        draw(&world, &font, &mut canvas);

        fps_manager.delay();
    }
}
