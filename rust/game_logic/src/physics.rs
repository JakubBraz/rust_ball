use std::f32::consts::PI;
use rapier2d::na::Rotation2;
use rapier2d::prelude::*;

const PLAYER_SPEED: f32 = 10.0;
const PLAYER_ACC: f32 = 2.0;
const MAX_VEC: f32 = 5.0;
const BALL_IMPACT: f32 = 5.0;

pub struct GamePhysics {
    physics_pipeline: PhysicsPipeline,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    gravity: Vector<Real>,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multi_body_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
    player_handle: RigidBodyHandle,
    player2_handle: RigidBodyHandle,
    ball_handle: RigidBodyHandle,
    wall_handlers: Vec<ColliderHandle>,
    ball_collider_handle: ColliderHandle,

    active_forces: [bool; 4],

    player3_handle: RigidBodyHandle,
    player_collider_handle: ColliderHandle,
    touch_vector: Vector<f32>,
    after_kick: bool,

    player4_handle: RigidBodyHandle,
    player4_collider_handle: ColliderHandle,
    touch_vector4: Vector<f32>,
    after_kick4: bool,
}

impl GamePhysics {
    pub fn init() -> GamePhysics {
        let mut gp = GamePhysics {
            physics_pipeline: Default::default(),
            rigid_body_set: Default::default(),
            collider_set: Default::default(),
            // gravity: vector![0.0, -9.81],
            gravity: vector![0.0, 0.0],
            integration_parameters: Default::default(),
            island_manager: Default::default(),
            broad_phase: Default::default(),
            narrow_phase: Default::default(),
            impulse_joint_set: Default::default(),
            multi_body_joint_set: Default::default(),
            ccd_solver: Default::default(),
            query_pipeline: Default::default(),
            player_handle: Default::default(),
            player2_handle: Default::default(),
            player3_handle: Default::default(),
            ball_handle: Default::default(),
            wall_handlers: vec![],
            player_collider_handle: Default::default(),
            ball_collider_handle: Default::default(),
            active_forces: [false, false, false, false],
            touch_vector: vector![0.0, 0.0],
            after_kick: false,

            player4_handle: Default::default(),
            player4_collider_handle: Default::default(),
            touch_vector4: vector![0.0, 0.0],
            after_kick4: false,
        };

        /* Create the ground. */
        let wall = ColliderBuilder::cuboid(15.0, 0.5)
            .restitution(0.7)
            .translation(vector![20.0, 0.0])
            .build();
        let handler = gp.collider_set.insert(wall);
        gp.wall_handlers.push(handler);

        let wall = ColliderBuilder::cuboid(15.0, 0.5)
            .restitution(0.7)
            .translation(vector![20.0, 30.0])
            .build();
        let handler = gp.collider_set.insert(wall);
        gp.wall_handlers.push(handler);

        let wall = ColliderBuilder::cuboid(0.5, 15.0)
            .restitution(0.7)
            .translation(vector![5.0, 15.0])
            .build();
        let handler = gp.collider_set.insert(wall);
        gp.wall_handlers.push(handler);

        let wall = ColliderBuilder::cuboid(0.5, 15.0)
            .restitution(0.7)
            .translation(vector![35.0, 15.0])
            .build();
        let handler = gp.collider_set.insert(wall);
        gp.wall_handlers.push(handler);

        // create player
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![20.0, 10.0])
            .linear_damping(1.0)
            .build();
        let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
        let ball_body_handle = gp.rigid_body_set.insert(rigid_body);
        gp.collider_set
            .insert_with_parent(collider, ball_body_handle, &mut gp.rigid_body_set);
        gp.player_handle = ball_body_handle;

        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![30.0, 25.0])
            .linear_damping(1.0)
            .build();
        let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
        let player2_handle = gp.rigid_body_set.insert(rigid_body);
        gp.collider_set
            .insert_with_parent(collider, player2_handle, &mut gp.rigid_body_set);
        gp.player2_handle = player2_handle;

        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![10.0, 15.0])
            .linear_damping(1.0)
            .build();
        let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
        let player3_handle = gp.rigid_body_set.insert(rigid_body);
        let player_collider_handler = gp.collider_set
            .insert_with_parent(collider, player3_handle, &mut gp.rigid_body_set);
        gp.player3_handle = player3_handle;
        gp.player_collider_handle = player_collider_handler;

        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![30.0, 15.0])
            .linear_damping(1.0)
            .build();
        let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
        let player4_handle = gp.rigid_body_set.insert(rigid_body);
        let player_collider_handler = gp.collider_set
            .insert_with_parent(collider, player4_handle, &mut gp.rigid_body_set);
        gp.player4_handle = player4_handle;
        gp.player4_collider_handle = player_collider_handler;

        // create ball
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![10.0, 10.0])
            .linear_damping(1.0)
            .build();
        let collider = ColliderBuilder::ball(0.3).restitution(0.7).build();
        let ball_handle = gp.rigid_body_set.insert(rigid_body);
        let collider_handle = gp.collider_set
            .insert_with_parent(collider, ball_handle, &mut gp.rigid_body_set);
        gp.ball_handle = ball_handle;
        gp.ball_collider_handle = collider_handle;

        gp
    }

    pub fn step(&mut self) {
        let mut player2 = &mut self.rigid_body_set[self.player2_handle];
        // println!("{:?} {:?}", self.active_forces, player2.user_force());
        // println!("VEL: {}", player2.linvel());
        // println!("{}", player2.linvel().norm());

        player2.reset_forces(true);
        let force_vector = vector![0.0, PLAYER_SPEED];
        let force = match self.active_forces {
            [false, true, false, false] => force_vector,
            [false, true, true, false] => Rotation2::new(PI * 0.25) * force_vector,
            [false, false, true, false] => Rotation2::new(PI * 0.5) * force_vector,
            [true, false, true, false] => Rotation2::new(PI * 0.75) * force_vector,
            [true, false, false, false] => Rotation2::new(PI) * force_vector,
            [true, false, false, true] => Rotation2::new(PI * 1.25) * force_vector,
            [false, false, false, true] => Rotation2::new(PI * 1.5) * force_vector,
            [false, true, false, true] => Rotation2::new(PI * 1.75) * force_vector,
            _ => vector![0.0, 0.0]
        };
        player2.add_force(force, true);

        let mut player3 = &mut self.rigid_body_set[self.player3_handle];
        player3.reset_forces(true);
        if player3.linvel().norm() < self.touch_vector.norm() {
            player3.add_force(self.touch_vector, true);
        }
        // println!("{}", player3.linvel().norm());
        // println!("{}", self.touch_vector.norm());

        let ball_contact = match self.narrow_phase.contact_pair(self.player_collider_handle, self.ball_collider_handle) {
            Some(pair) => pair.has_any_active_contact,
            None => false
        };
        // println!("{} {}", ball_contact, player3.linvel().norm());
        if !self.after_kick && ball_contact && self.touch_vector.norm() + 0.1 >= MAX_VEC {
            let v: Vector<f32> = self.rigid_body_set[self.ball_handle].translation() - self.rigid_body_set[self.player3_handle].translation();
            let impulse = v.normalize() * BALL_IMPACT;
            let ball = &mut self.rigid_body_set[self.ball_handle];
            ball.apply_impulse(impulse, true);
            self.after_kick = true;
        }

        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multi_body_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }

    pub fn player(&self) -> (f32, f32, f32, f32, f32, f32) {
        let ball_body = &self.rigid_body_set[self.player_handle];
        let player_handle = ball_body.colliders().first().unwrap().0;
        let player_radius = &self.collider_set[player_handle]
            .shape()
            .as_ball()
            .unwrap()
            .radius;
        let ball = &self.rigid_body_set[self.ball_handle];
        let ball_pos = ball.translation();
        let ball_radius = &self.collider_set[ball.colliders().first().unwrap().0]
            .shape()
            .as_ball()
            .unwrap()
            .radius;

        (
            ball_body.translation().x,
            ball_body.translation().y,
            *player_radius,
            ball_pos.x,
            ball_pos.y,
            *ball_radius,
        )
    }

    pub fn player2(&self) -> (f32, f32, f32) {
        let p = &self.rigid_body_set[self.player2_handle];
        let collider_handle = p.colliders().first().unwrap().0;
        let radius = &self.collider_set[collider_handle].shape().as_ball().unwrap().radius;
        (p.translation().x, p.translation().y, *radius)
    }

    pub fn player3(&self) -> (f32, f32, f32) {
        let p = &self.rigid_body_set[self.player3_handle];
        let collider_handle = p.colliders().first().unwrap().0;
        let radius = &self.collider_set[collider_handle].shape().as_ball().unwrap().radius;
        (p.translation().x, p.translation().y, *radius)
    }

    pub fn player4(&self) -> (f32, f32, f32) {
        let p = &self.rigid_body_set[self.player4_handle];
        let collider_handle = p.colliders().first().unwrap().0;
        let radius = &self.collider_set[collider_handle].shape().as_ball().unwrap().radius;
        (p.translation().x, p.translation().y, *radius)
    }

    pub fn apply_impulse(&mut self, x: f32, y: f32) {
        &self.rigid_body_set[self.player_handle].apply_impulse(vector![x, y], true);
    }

    pub fn move_player(&mut self, x: f32, y: f32) {
        self.rigid_body_set[self.player2_handle].add_force(vector![x, y], true);
    }

    pub fn stop_force(&mut self) {
        self.rigid_body_set[self.player2_handle].reset_forces(true);
    }

    pub fn move_mouse(&mut self, vec_x: f32, vec_y: f32) {
        if vec_x == 0.0 && vec_y == 0.0 {
            self.reset_kick();
        }
        let mut v = vector![vec_x, vec_y];
        if v.norm() > 1.0 {
            v = v.normalize();
        }
        self.touch_vector = v * MAX_VEC;
    }

    pub fn player_input(&mut self, keys: [bool; 4]) {
        self.active_forces = keys;
    }

    pub fn static_bodies(&self) -> Vec<(f32, f32, f32, f32)> {
        self.wall_handlers
            .iter()
            .map(|x| {
                let s = self.collider_set[*x]
                    .shape()
                    .as_cuboid()
                    .unwrap()
                    .half_extents;
                let t = self.collider_set[*x].translation();
                (-s.x + t.x, -s.y + t.y, s.x * 2.0, s.y * 2.0)
            })
            .collect()
    }

    pub fn get_game_state(&self) -> (f32, f32, f32, f32, f32, f32) {
        let (player1_x, player1_y, player1_r) = self.player3();
        let (player2_x, player2_y, player2_r) = self.player4();
        let (_, _, _, ball_x, ball_y, ball_r) = self.player();
        (ball_x, ball_y, player1_x, player1_y, player2_x, player2_y)
    }

    fn reset_kick(&mut self) {
        if self.after_kick {
            self.after_kick = false;
        }
    }
}
