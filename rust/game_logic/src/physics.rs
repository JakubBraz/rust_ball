use rapier2d::prelude::*;

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
    ball_handle: RigidBodyHandle,
    wall_handlers: Vec<ColliderHandle>,
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
            ball_handle: Default::default(),
            wall_handlers: vec![],
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

        /* Create the bouncing ball. */
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![20.0, 10.0])
            .build();
        // let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
        let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
        let ball_body_handle = gp.rigid_body_set.insert(rigid_body);
        gp.collider_set
            .insert_with_parent(collider, ball_body_handle, &mut gp.rigid_body_set);
        gp.player_handle = ball_body_handle;

        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![10.0, 10.0])
            .build();
        let collider = ColliderBuilder::ball(0.25).restitution(0.7).build();
        let ball_handle = gp.rigid_body_set.insert(rigid_body);
        gp.collider_set
            .insert_with_parent(collider, ball_handle, &mut gp.rigid_body_set);
        gp.ball_handle = ball_handle;

        gp
    }

    pub fn step(&mut self) {
        let mut b = &mut self.rigid_body_set[self.player_handle];
        b.reset_forces(true);
        let v = b.linvel();
        b.add_force(vector![-v.x, -v.y], true);

        let mut ball = &mut self.rigid_body_set[self.ball_handle];
        ball.reset_forces(true);
        let v = ball.linvel();
        ball.add_force(vector![-v.x * 0.2, -v.y * 0.2], true);

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

    pub fn apply_force(&mut self, x: f32, y: f32) {
        &self.rigid_body_set[self.player_handle].apply_impulse(vector![x, y], true);
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
}
