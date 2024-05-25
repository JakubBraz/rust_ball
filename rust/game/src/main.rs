use game_logic::physics::GamePhysics;
use macroquad::prelude::*;
use rapier2d::prelude::Vector;

#[derive(Debug)]
struct S {
    x: u32,
    y: u32
}

#[derive(Debug)]
struct ComplexStruct {
    val: i32,
    v: Vec<u32>
}

#[macroquad::main("BasicShapes")]
async fn main() {
// fn main() {
    println!("Hello, world!");
    println!("{:?}", game_logic::add(1, 2));
    println!("{:?}", game_logic::f2(30, 3));
    // println!("{:?}", game_logic::physics::diff(30, 3));

    let mut a = S {x: 13, y : 19};
    println!("{:?}", a);
    a.x = 12;
    println!("{:?}", a);

    let mut st = ComplexStruct {val: 10, v: vec![1, 2, 3]};
    println!("{:?}", st);
    st.v.push(12);
    println!("{:?}", st);

    let mut s = GamePhysics::init();

    loop {
        s.step();
        clear_background(RED);

        draw_line(40.0, 40.0, 100.0, 200.0, 15.0, BLUE);
        draw_rectangle(screen_width() / 2.0 - 60.0, 100.0, 120.0, 60.0, GREEN);
        // draw_circle(screen_width() - 30.0, screen_height() - 30.0, 15.0, YELLOW);
        let (x, y) = s.ball();
        println!("{:?}", (x, y));
        draw_circle(x * 20.0, y * 20.0, 15.0, YELLOW);

        draw_text("IT WORKS!", 20.0, 20.0, 30.0, DARKGRAY);

        next_frame().await
    }
}
