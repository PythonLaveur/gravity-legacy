use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use std::collections::HashSet;

use heron::prelude::*;

// region: --- common structs

#[derive(Component)]
pub struct Camera;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: CollisionShape,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub rotation_constraints: RotationConstraints,
    pub physic_material: PhysicMaterial,
}

impl From<EntityInstance> for ColliderBundle {
    fn from(entity_instance: EntityInstance) -> ColliderBundle {
        let rotation_constraints = RotationConstraints::lock();
        match entity_instance.identifier.as_str() {
            "Player" => ColliderBundle {
                collider: CollisionShape::Cuboid {
                    half_extends: Vec3::new(6., 6., 0.),
                    border_radius: None,
                },
                rigid_body: RigidBody::Dynamic,
                rotation_constraints,
                ..Default::default()
            },
            "Pot" => ColliderBundle {
                collider: CollisionShape::Cuboid {
                    half_extends: Vec3::new(5., 5., 0.),
                    border_radius: None,
                },
                rigid_body: RigidBody::Dynamic,
                rotation_constraints,
                ..Default::default()
            },
            "Key" => ColliderBundle {
                collider: CollisionShape::Cuboid {
                    half_extends: Vec3::new(8., 8., 0.),
                    border_radius: None,
                },
                rigid_body: RigidBody::Dynamic,
                rotation_constraints,
                physic_material: PhysicMaterial {
                    friction: 0.5,
                    density: 15.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            _ => ColliderBundle::default(),
        }
    }
}

impl From<IntGridCell> for ColliderBundle {
    fn from(int_grid_cell: IntGridCell) -> ColliderBundle {
        println!("Setup grid");
        let rotation_constraints = RotationConstraints::lock();

        if int_grid_cell.value == 2 || int_grid_cell.value == 1 {
            ColliderBundle {
                collider: CollisionShape::Cuboid {
                    half_extends: Vec3::new(8., 8., 0.),
                    border_radius: None,
                },
                rigid_body: RigidBody::Sensor,
                rotation_constraints,
                ..Default::default()
            }
        } else {
            ColliderBundle::default()
        }
    }
}

// endregion: --- common structs

#[derive(Copy, Clone, PartialEq, Debug, Component)]
pub struct Player {
    pub previous_input: Vec2,
}
impl Default for Player {
    fn default() -> Self {
        Player {
            previous_input: Vec2::new(0., 0.),
        }
    }
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_bundle("player.png")]
    #[bundle]
    pub sprite_bundle: SpriteBundle,
    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,
    pub player: Player,
    #[worldly]
    pub worldly: Worldly,
    pub ground_detection: GroundDetection,

    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
}

#[derive(Clone, Default, Component)]
pub struct GroundDetection {
    pub on_ground: bool,
}

#[derive(Component)]
pub struct GroundSensor {
    pub ground_detection_entity: Entity,
    pub intersecting_ground_entities: HashSet<Entity>,
}
