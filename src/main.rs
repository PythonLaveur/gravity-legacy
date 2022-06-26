use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

//use components::PlayerBundle;
use heron::prelude::*;
use systems::print_player_position;
//use systems::{process_my_entities};

mod components;
mod systems;

// region:    --- Assets constants

const MAP_LDTK: &str = "test.ldtk";

const TILE_SIZE: f32 = 16.;

// endregion: --- Assets constants

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "RustyGame!".to_string(),
            width: 500.0,
            height: 300.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LdtkPlugin)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::new(0.0, -10., 0.0)))
        .insert_resource(LevelSelection::Index(0))
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..Default::default()
        })
        .add_startup_system(systems::setup)
        //.add_startup_system(process_my_entities)
        .add_system(systems::spawn_wall_collision)
        .add_system(print_player_position)
        
        // Map the components to match project structs
        // Tiles
        .register_ldtk_int_cell::<components::WallBundle>(1)
        .register_ldtk_int_cell::<components::WallBundle>(2)
        //Entities
        .register_ldtk_entity::<components::PlayerBundle>("Player")
        .run();
}