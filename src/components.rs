use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use std::collections::HashSet;

use heron::prelude::*;

use crate::systems::Animation;

// region: --- common structs

#[derive(Component)]
pub struct MainCamera;

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
                    half_extends: Vec3::new(8., 15., 0.),
                    border_radius: None,
                },
                rigid_body: RigidBody::Dynamic,
                rotation_constraints,
                physic_material: PhysicMaterial {
                    friction: 0.,
                    density: 15.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            "Key" => ColliderBundle {
                collider: CollisionShape::Cuboid {
                    half_extends: Vec3::new(8., 8., 0.),
                    border_radius: None,
                },
                rigid_body: RigidBody::Sensor,
                rotation_constraints,
                ..Default::default()
            },
            _ => ColliderBundle::default(),
        }
    }
}

impl From<IntGridCell> for ColliderBundle {
    fn from(int_grid_cell: IntGridCell) -> ColliderBundle {
        let rotation_constraints = RotationConstraints::lock();

        if int_grid_cell.value == 2 || int_grid_cell.value == 1 {
            ColliderBundle {
                collider: CollisionShape::Cuboid {
                    half_extends: Vec3::new(8., 8., 0.),
                    border_radius: None,
                },
                rigid_body: RigidBody::Static,
                rotation_constraints,
                ..Default::default()
            }
        } else if int_grid_cell.value == 3 {
            ColliderBundle {
                collider: CollisionShape::Cuboid {
                    half_extends: Vec3::new(5., 5., 0.),
                    border_radius: None,
                },
                rigid_body: RigidBody::Sensor,
                rotation_constraints,
                ..Default::default()
            }
        } else {ColliderBundle::default()}
    }
}

// endregion: --- common structs

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
    //The code below allows to spawn a collider for each cell, not very good performance wise
    //#[from_int_grid_cell]
    //#[bundle]
    //pub collider_bundle: ColliderBundle,
}

#[derive(Copy, Clone, PartialEq, Debug, Default, Component)]
pub struct Pot;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PotBundle {
    #[sprite_bundle("Sprites/Traps/Pot.png")]
    #[bundle]
    pub sprite_bundle: SpriteBundle,
    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,
    pub pot: Pot,
    pub ground_detection: GroundDetection,

    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Spikes;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct SpikesBundle {
    spikes: Spikes,
    #[from_int_grid_cell]
    #[bundle]
    pub collider_bundle: ColliderBundle,
}

#[derive(Component)]
pub struct AnimationToSpawn(pub Vec3, pub Animation);

#[derive(Component)]
pub struct FxAnimationTimer(pub Timer, pub usize);

#[derive(Copy, Clone, PartialEq, Debug, Default, Component)]
pub struct Key;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct KeyBundle {
    #[sprite_bundle("Sprites/Items/Checkpoints/flag.png")]
    #[bundle]
    pub sprite_bundle: SpriteBundle,
    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,
    pub pot: Key,
    pub ground_detection: GroundDetection,

    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
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

#[derive(Copy, Clone, PartialEq, Debug, Component)]
pub struct Player;

#[derive(Component)]
pub struct SpriteSize {
    pub val: Vec2,
}

#[derive(Component, Debug)]
pub struct Slime {
    pub side: crate::slime_collision::Side,
    pub side_before: crate::slime_collision::Side,
    pub lenght_on_side: f32,
    pub depth: f32,
    pub is_jumping: bool,
    pub is_walking: bool,
    pub need_new_sprite: bool,
    pub stop_timer: u8,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Component, Deref, DerefMut)]
pub struct ResetCollisionTimer(pub Timer);
