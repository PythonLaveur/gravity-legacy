// This example shows off a more in-depth implementation of a game with `bevy_ecs_ldtk`.
// Please run with `--release`.

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use heron::prelude::*;

mod components;
mod systems;

// region:    --- Assets constants

const MAP_LDTK: &str = "test.ldtk";

// endregion: --- Assets constants

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LdtkPlugin)
        .add_plugin(PhysicsPlugin::default())
        //Gravity parameter
        .insert_resource(Gravity::from(Vec3::new(0.0, -2000., 0.0)))
        //Select the lvl to compute
        .insert_resource(LevelSelection::Index(0))
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..Default::default()
        })
        .add_startup_system(systems::setup)
        // Map the components to match project structs
        // Tiles
        .register_ldtk_int_cell::<components::WallBundle>(1)
        .register_ldtk_int_cell::<components::FloorBundle>(2)
        //Entities
        .register_ldtk_entity::<components::PlayerBundle>("Player")
        .register_ldtk_entity::<components::PotBundle>("Pot")
        .register_ldtk_entity::<components::KeyBundle>("Key")
        .run();
}
