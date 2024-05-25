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
    ball_handle: RigidBodyHandle
}

impl GamePhysics {
    pub fn init() -> GamePhysics {
        let mut gp = GamePhysics {
            physics_pipeline: Default::default(),
            rigid_body_set: Default::default(),
            collider_set: Default::default(),
            gravity: vector![0.0, -9.81],
            integration_parameters: Default::default(),
            island_manager: Default::default(),
            broad_phase: Default::default(),
            narrow_phase: Default::default(),
            impulse_joint_set: Default::default(),
            multi_body_joint_set: Default::default(),
            ccd_solver: Default::default(),
            query_pipeline: Default::default(),
            ball_handle: Default::default(),
        };

        /* Create the ground. */
        let collider = ColliderBuilder::cuboid(100.0, 0.1).build();
        gp.collider_set.insert(collider);

        /* Create the bouncing ball. */
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(vector![10.0, 10.0])
            .build();
        // let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
        let collider = ColliderBuilder::ball(0.5).restitution(1.5).build();
        let ball_body_handle = gp.rigid_body_set.insert(rigid_body);
        gp.collider_set.insert_with_parent(collider, ball_body_handle, &mut gp.rigid_body_set);
        gp.ball_handle = ball_body_handle;

        gp
    }

    pub fn step(&mut self) {
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
            &()
        );
    }

    pub fn ball(&self) -> (f32, f32) {
        let ball_body = &self.rigid_body_set[self.ball_handle];
        (ball_body.translation().x, ball_body.translation().y)
    }
}

fn main() {
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    /* Create the ground. */
    let collider = ColliderBuilder::cuboid(100.0, 0.1).build();
    collider_set.insert(collider);

    /* Create the bouncing ball. */
    let rigid_body = RigidBodyBuilder::dynamic()
        .translation(vector![0.0, 10.0])
        .build();
    let collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
    let ball_body_handle = rigid_body_set.insert(rigid_body);
    collider_set.insert_with_parent(collider, ball_body_handle, &mut rigid_body_set);

    /* Create other structures necessary for the simulation. */
    let gravity = vector![0.0, -9.81];
    let integration_parameters = IntegrationParameters::default();
    let mut physics_pipeline = PhysicsPipeline::new();
    let mut island_manager = IslandManager::new();
    let mut broad_phase = DefaultBroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut impulse_joint_set = ImpulseJointSet::new();
    let mut multibody_joint_set = MultibodyJointSet::new();
    let mut ccd_solver = CCDSolver::new();
    let mut query_pipeline = QueryPipeline::new();
    let physics_hooks = ();
    let event_handler = ();

    /* Run the game loop, stepping the simulation once per frame. */
    for _ in 0..200 {
        physics_pipeline.step(
            &gravity,
            &integration_parameters,
            &mut island_manager,
            &mut broad_phase,
            &mut narrow_phase,
            &mut rigid_body_set,
            &mut collider_set,
            &mut impulse_joint_set,
            &mut multibody_joint_set,
            &mut ccd_solver,
            Some(&mut query_pipeline),
            &physics_hooks,
            &event_handler,
        );

        let ball_body = &rigid_body_set[ball_body_handle];
        println!(
            "Ball altitude: {}",
            ball_body.translation().y
        );
    }
}
