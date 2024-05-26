use game_logic::physics::GamePhysics;
use macroquad::input::KeyCode::{Down, Escape, Left, Right, Up, W};
use macroquad::prelude::*;
use rapier2d::prelude::Vector;
use std::process::exit;

const SCALING: f32 = 20.0;

#[derive(Debug)]
struct S {
    x: u32,
    y: u32,
}

#[derive(Debug)]
struct ComplexStruct {
    val: i32,
    v: Vec<u32>,
}

#[macroquad::main("BasicShapes")]
async fn main() {
    // fn main() {
    println!("Hello, world!");
    println!("{:?}", game_logic::add(1, 2));
    println!("{:?}", game_logic::f2(30, 3));
    // println!("{:?}", game_logic::physics::diff(30, 3));

    let mut a = S { x: 13, y: 19 };
    println!("{:?}", a);
    a.x = 12;
    println!("{:?}", a);

    let mut st = ComplexStruct {
        val: 10,
        v: vec![1, 2, 3],
    };
    println!("{:?}", st);
    st.v.push(12);
    println!("{:?}", st);

    let mut s = GamePhysics::init();

    println!("{:?}", s.player());

    loop {
        if is_key_pressed(Up) {
            s.apply_force(0.0, -10.0);
        } else if is_key_pressed(Down) {
            s.apply_force(0.0, 10.0);
        } else if is_key_pressed(Left) {
            s.apply_force(-10.0, 0.0);
        } else if is_key_pressed(Right) {
            s.apply_force(10.0, 0.0);
        } else if is_key_pressed(Escape) {
            exit(0);
        }

        s.step();
        clear_background(Color::from_hex(0x_00_99_00));

        // draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        // draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        let (player_x, player_y, player_radius, ball_x, ball_y, ball_radius) = s.player();
        // println!("{:?}", (player_x, player_y, player_x * SCALING, player_y * SCALING));
        draw_circle(
            player_x * SCALING,
            player_y * SCALING,
            player_radius * SCALING,
            YELLOW,
        );
        draw_circle(
            ball_x * SCALING,
            ball_y * SCALING,
            ball_radius * SCALING,
            WHITE,
        );

        let walls = s.static_bodies();
        for (x, y, w, h) in walls {
            draw_rectangle(
                x * SCALING,
                y * SCALING,
                w * SCALING,
                h * SCALING,
                Color::from_hex(0x_00_BB_00),
            );
        }

        next_frame().await
    }
}
