extern crate legion;
extern crate sdl2;

use legion::prelude::*;

use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::gfx::framerate::FPSManager;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

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
struct Paddle(u8);

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity {
    dx: i32,
    dy: i32,
}

fn collide(world: &mut World) {
    let query = <(Write<Position>, Write<Velocity>)>::query();
    
    for (entity, (mut pos, mut vel)) in query.iter_entities(world) {
        if pos.x < 0 {
            pos.x = 0;
            vel.dx *= -1;
        } else if (pos.x + pos.width) >= WIDTH {
            pos.x = WIDTH - pos.width;
            vel.dx *= -1;
        }

        if pos.y < 0 {
            pos.y = 0;
            vel.dy *= -1;
        } else if (pos.y + pos.height) >= HEIGHT {
            pos.y = HEIGHT - pos.height;
            vel.dy *= -1;
        }
    }
}

fn draw(world: &World, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for pos in <Read<Position>>::query().iter_immutable(world)
    {
        canvas.fill_rect(Rect::new(
            pos.x,
            pos.y,
            pos.width as u32,
            pos.height as u32
        ));
    }

    canvas.present();
}

fn input(world: &mut World, event_pump: &mut EventPump, p1: Entity, p2: Entity) -> bool {

    for e in event_pump.poll_iter() {
        match e {
            Event::Quit { .. } => return true,
            Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                let mut p1_vel = world.get_component_mut::<Velocity>(p1).unwrap();
                p1_vel.dy = PADDLE_MOVE_SPEED;
            },
            Event::KeyUp { keycode: Some(Keycode::S), .. } => {
                let mut p1_vel = world.get_component_mut::<Velocity>(p1).unwrap();
                p1_vel.dy = 0;
            },
            Event::KeyDown { keycode: Some(Keycode::W), .. } => {
                let mut p1_vel = world.get_component_mut::<Velocity>(p1).unwrap();
                p1_vel.dy = -PADDLE_MOVE_SPEED;
            },
            Event::KeyUp { keycode: Some(Keycode::W), .. } => {
                let mut p1_vel = world.get_component_mut::<Velocity>(p1).unwrap();
                p1_vel.dy = 0;
            },
            Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                let mut p2_vel = world.get_component_mut::<Velocity>(p2).unwrap();
                p2_vel.dy = PADDLE_MOVE_SPEED;
            },
            Event::KeyUp { keycode: Some(Keycode::Down), .. } => {
                let mut p2_vel = world.get_component_mut::<Velocity>(p2).unwrap();
                p2_vel.dy = 0;
            },
            Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                let mut p2_vel = world.get_component_mut::<Velocity>(p2).unwrap();
                p2_vel.dy = -PADDLE_MOVE_SPEED;
            },
            Event::KeyUp { keycode: Some(Keycode::Up), .. } => {
                let mut p2_vel = world.get_component_mut::<Velocity>(p2).unwrap();
                p2_vel.dy = 0;
            },

            _ => {}
        }
    }

    return false;
}

fn paddle_collide(ball: Entity, world: &mut World) {
    let ball_rect = {
        let ball_pos = world.get_component::<Position>(ball).unwrap();

        Rect::new(ball_pos.x, ball_pos.y, ball_pos.width as u32, ball_pos.height as u32)
    };

    let p1_rect = {
        <Read<Position>>::query()
            .filter(tag::<Paddle>())
            .iter(world)
            .map(|p| {
                Rect::new(p.x, p.y, p.width as u32, p.height as u32)
            })
            .nth(0)
            .unwrap()
    };

    let p2_rect = {
        <Read<Position>>::query()
            .filter(tag::<Paddle>())
            .iter(world)
            .map(|p| {
                Rect::new(p.x, p.y, p.width as u32, p.height as u32)
            })
            .nth(1)
            .unwrap()
    };

    if ball_rect.has_intersection(p1_rect) || ball_rect.has_intersection(p2_rect) {
        let mut ball_vel = world.get_component_mut::<Velocity>(ball).unwrap();

        ball_vel.dx *= -1;
    }
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

    let ball = world.insert(
        (Ball,),
        vec![(
            Position {
                x: 0,
                y: 0,
                width: BALL_WIDTH,
                height: BALL_HEIGHT,
            },
            Velocity { dx: 3, dy: 2 },
        )],
    )[0];

    let p1 = world.insert(
        (Paddle(0),),
        vec![(
            Position {
                x: 80,
                y: 40,
                width: PADDLE_WIDTH,
                height: PADDLE_HEIGHT
            },
            Velocity { dx: 0, dy: 0 },
        )]
    )[0];

    let p2 = world.insert(
        (Paddle(1),),
        vec![(
            Position {
                x: WIDTH - PADDLE_WIDTH - 80,
                y: 40,
                width: PADDLE_WIDTH,
                height: PADDLE_HEIGHT
            },
            Velocity { dx: 0, dy: 0 },
        )]
    )[0];

    let mut fps_manager = FPSManager::new();
    fps_manager.set_framerate(60);

    loop {
        if input(&mut world, &mut event_pump, p1, p2) {
            break;
        }

        tick(&mut world);
        collide(&mut world);
        paddle_collide(ball, &mut world);
        draw(&world, &mut canvas);

        fps_manager.delay();
    }
}
