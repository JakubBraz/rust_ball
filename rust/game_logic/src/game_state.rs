#[derive(Default, PartialEq)]
pub struct GameState {
    pub player_x: f32,
    pub player_y: f32,
    pub player_r: f32,
    pub ball_x: f32,
    pub ball_y: f32,
    pub ball_r: f32,
    pub touch_vec_x: f32,
    pub touch_vec_y: f32,
}

impl GameState {
    pub fn to_bytes(&self) -> Vec<u8> {
        [
            self.player_x.to_ne_bytes(),
            self.player_y.to_ne_bytes(),
            self.player_r.to_ne_bytes(),
            self.ball_x.to_ne_bytes(),
            self.ball_y.to_ne_bytes(),
            self.ball_r.to_ne_bytes(),
            // todo don't send to client touch_vec, let it handle drawing it on its own
            self.touch_vec_x.to_ne_bytes(),
            self.touch_vec_y.to_ne_bytes(),
        ].concat()
    }
}
