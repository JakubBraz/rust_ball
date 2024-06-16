// use game_logic::physics::GamePhysics;
use macroquad::input::KeyCode::{A, D, Down, Escape, Left, Right, S, Up, W};
use macroquad::prelude::*;
use rapier2d::prelude::{Vector, vector};
use std::process::exit;
use game_logic_lib::physics::GamePhysics;
use rapier2d::parry::transformation::utils::scaled;

const SCALING: f32 = 20.0;
const MAX_TOUCH_LEN: f32 = 70.0;

#[macroquad::main("BasicShapes")]
async fn main() {
    // fn main() {
    println!("Hello, world!");
    // println!("{:?}", game_logic::physics::diff(30, 3));

    let mut s = GamePhysics::init();
    let mut first_touch: (f32, f32) = (0.0, 0.0);
    let mut vec_x = 0.0;
    let mut vec_y = 0.0;

    println!("{:?}", s.player());

    loop {
        if is_key_pressed(Up) {
            s.apply_impulse(0.0, -10.0);
        }
        if is_key_pressed(Down) {
            s.apply_impulse(0.0, 10.0);
        }
        if is_key_pressed(Left) {
            s.apply_impulse(-10.0, 0.0);
        }
        if is_key_pressed(Right) {
            s.apply_impulse(10.0, 0.0);
        }

        if is_mouse_button_down(MouseButton::Left) {
            let mp = mouse_position();
            if first_touch == (0.0, 0.0) {
                first_touch = mp;
            }
            vec_x = mp.0 - first_touch.0;
            vec_y = mp.1 - first_touch.1;
            let vec_len = (vec_x * vec_x + vec_y * vec_y).sqrt();
            if vec_len > MAX_TOUCH_LEN {
                vec_x = vec_x / vec_len * MAX_TOUCH_LEN;
                vec_y = vec_y / vec_len * MAX_TOUCH_LEN;
            }
            let vx = vec_x / MAX_TOUCH_LEN;
            let vy = vec_y / MAX_TOUCH_LEN;
            s.move_mouse(vx, vy);
        }
        if is_mouse_button_released(MouseButton::Left) {
            first_touch = (0.0, 0.0);
            vec_x = 0.0;
            vec_y = 0.0;
            s.move_mouse(vec_x, vec_y);
        }

        s.player_input([
            get_keys_down().contains(&W),
            get_keys_down().contains(&S),
            get_keys_down().contains(&A),
            get_keys_down().contains(&D)
        ]);

        if is_key_pressed(Escape) {
            exit(0);
        }

        // if get_keys_down().is_empty() {
        //     s.stop_force();
        // }

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

        let (p2x, p2y, p2r) = s.player2();
        draw_circle(p2x * SCALING, p2y * SCALING, p2r * SCALING, Color::from_hex(0x_77_77_77));

        let (p3x, p3y, p3r) = s.player3();
        draw_circle(p3x * SCALING, p3y * SCALING, p3r * SCALING, Color::from_hex(0x_FF_AA_11));
        draw_line(p3x * SCALING, p3y * SCALING, p3x * SCALING + vec_x, p3y * SCALING + vec_y, 2.0, Color::from_hex(0x_FF_00_00));

        let (p4x, p4y, p4r) = s.player4();
        draw_circle(p4x * SCALING, p4y * SCALING, p4r * SCALING, Color::from_hex(0x_11_AA_FF));

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
