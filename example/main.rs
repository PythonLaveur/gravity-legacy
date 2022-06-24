use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct Chest;

// region: Game constants

pub const PIXELS_PER_METER: f32 = 492.3;

// endregion: Game constants

// region: Assets constants

// endregion: Assets constants

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "test".to_string(),
            width: 360.0,
            height: 640.0,
            ..Default::default()
        })
        .insert_resource(Msaa::default())
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.label("main_setup"))
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(
            PIXELS_PER_METER,
        ))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    // Set gravity to x and spawn camera.
    //rapier_config.gravity = Vector2::zeros();
    rapier_config.gravity = Vec2::new(0.0, -220.0);

    commands
        .spawn()
        .insert_bundle(OrthographicCameraBundle::new_2d());
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("Items/Boxes/Box1/Idle.png"),
            ..Default::default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(Collider::cuboid(10., 10.))
        .insert(Transform::from_xyz(0., 0., 0.))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Restitution::coefficient(0.7))
        .insert(Chest);

    for i in 1..10 {
        commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("Items/Boxes/Box1/Idle.png"),
            ..Default::default()
        })
        .insert(RigidBody::Fixed)
        .insert(Sleeping::disabled())
        .insert(Ccd::enabled())
        .insert(Collider::cuboid(10., 10.))
        .insert(Transform::from_xyz(-100. + (i as f32) * 20., -100., 0.))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(Restitution::coefficient(0.7))
        .insert(Chest);
    }
}
