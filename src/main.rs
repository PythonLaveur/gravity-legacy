#![allow(clippy::redundant_field_names)]
#![allow(clippy::too_many_arguments)]
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_kira_audio::AudioPlugin;
use components::MainCamera;
use slime::SlimePlugin;
use std::env;
//use components::PlayerBundle;
use ascii::AsciiPlugin;
use fadeout::FadeoutPlugin;
use heron::prelude::*;
use systems::*;
//use systems::{process_my_entities};
use start_menu::MainMenuPlugin;
mod ascii;
mod components;
mod fadeout;
mod slime;
mod slime_collision;
mod start_menu;
mod systems;

//Player sprites
const PLAYER_JUMP: &str = "Sprites/Player/jump.png";
const PLAYER_JUMP_SIZE: (f32, f32) = (24., 27.);
const PLAYER_JUMP_COLUMN: usize = 8;

const PLAYER_IDLE: &str = "Sprites/Player/idle.png";
const PLAYER_IDLE_SIZE: (f32, f32) = (22., 23.);
const PLAYER_IDLE_COLUMN: usize = 8;

const PLAYER_WALK: &str = "Sprites/Player/walk.png";
const PLAYER_WALK_SIZE: (f32, f32) = (23., 24.);
const PLAYER_WALK_COLUMN: usize = 10;

const PLAYER_SCALE: f32 = 0.7;
const SECURITY_DISTANCE: f32 = 10.;
const BASE_SPEED: f32 = 100.;

pub struct GameTextures {
    player_idle: Handle<TextureAtlas>,
    player_jump: Handle<TextureAtlas>,
    player_walk: Handle<TextureAtlas>,
}

// region:    --- Assets constants
pub const TILE_SIZE: f32 = 0.1;
const MAP_LDTK: &str = "Maps/Levels.ldtk";
pub const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);

//const TILE_SIZE: f32 = 16.;
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum GameState {
    StartMenu,
    Overworld,
    Combat,
}

pub struct GetGameState {
    game_state: GameState,
    level_index: usize,
}

// endregion: --- Assets constants
fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    App::new()
        .add_state(GameState::StartMenu)
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(WindowDescriptor {
            title: "Gravity Legacy 2".to_string(),
            width: 500.0,
            height: 300.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LdtkPlugin)
        .add_plugin(AudioPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(FadeoutPlugin)
        .add_plugin(AsciiPlugin)
        .add_plugin(MainMenuPlugin)
        .add_plugin(SlimePlugin)
        .insert_resource(Gravity::from(Vec3::new(0.0, -2000., 0.0)))
        .insert_resource(LevelSelection::Index(0))
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseZeroTranslation,
            set_clear_color: SetClearColor::FromLevelBackground,
            ..Default::default()
        })
        .add_startup_system(systems::setup)
        .add_system(systems::spawn_wall_collision)
        .add_system(world_rotation_system)
        .add_system(player_collision_with_pot)
        .add_startup_system(background_audio)
        .add_system(systems::animate_sprite_system)
        .add_system(spawn_level_system)
        // Map the components to match project structs
        // Tiles
        .register_ldtk_int_cell::<components::WallBundle>(1)
        .register_ldtk_int_cell::<components::WallBundle>(2)
        //Entities
        .register_ldtk_entity::<components::PotBundle>("Pot")
        .register_ldtk_entity::<components::KeyBundle>("Key")
        .run();
}

fn spawn_level_system (
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut gravity: ResMut<Gravity>,
    mut world_status: ResMut<WorldStatus>,
    mut get_game_state: ResMut<GetGameState>,
    mut query: Query<&mut Transform, With<MainCamera>>
) {
    if input.just_pressed(KeyCode::L) {
        commands.insert_resource(LevelSelection::Index(1));
        get_game_state.level_index = 1;
         //Reset the gravity
        if let Ok(mut camera_tf) = query.get_single_mut() { 
            camera_tf.rotation = Quat::from_axis_angle(Vec3::new(0., 0., 1.), 0.);
        }
        *gravity = Gravity::from(Vec3::new(0., -2000., 0.0));
        world_status.rotation = Vec2::new(1., 0.);
    }
}