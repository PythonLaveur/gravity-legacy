use crate::{components::Player, MAP_LDTK};
use crate::{
    components::*, GameState, GameTextures, GetGameState, PLAYER_IDLE, PLAYER_IDLE_COLUMN,
    PLAYER_IDLE_SIZE, PLAYER_JUMP, PLAYER_JUMP_COLUMN, PLAYER_JUMP_SIZE, PLAYER_WALK,
    PLAYER_WALK_COLUMN, PLAYER_WALK_SIZE,
};

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_kira_audio::Audio;

use std::collections::{HashMap, HashSet};

use heron::{CollisionShape, PhysicMaterial, RigidBody};

use heron::*;

const PI: f32 = 3.1415;

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    //Camera setup
    let mut camera = OrthographicCameraBundle::new_2d();
    //Offset
    camera.transform = Transform {
        translation: Vec3::new(0., 128., 1000.),
        ..default()
    };
    commands.spawn_bundle(camera).insert(MainCamera);

    //Add ressources
    commands.insert_resource(GetGameState {
        game_state: GameState::StartMenu,
    });

    //Enable to recall the setup but ignoring the code before
    asset_server.watch_for_changes().unwrap();

    //Load the levels
    let ldtk_handle = asset_server.load(MAP_LDTK);

    //load textures atlases :
    let texture_handle = asset_server.load(PLAYER_JUMP);
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::from(PLAYER_JUMP_SIZE),
        PLAYER_JUMP_COLUMN,
        1,
    );
    let player_jump = texture_atlases.add(texture_atlas);

    let texture_handle = asset_server.load(PLAYER_IDLE);
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::from(PLAYER_IDLE_SIZE),
        PLAYER_IDLE_COLUMN,
        1,
    );
    let player_idle = texture_atlases.add(texture_atlas);

    let texture_handle = asset_server.load(PLAYER_WALK);
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::from(PLAYER_WALK_SIZE),
        PLAYER_WALK_COLUMN,
        1,
    );
    let player_walk = texture_atlases.add(texture_atlas);

    let game_textures = GameTextures {
        player_idle,
        player_jump,
        player_walk,
    };
    commands.insert_resource(game_textures);

    //let map_entity = commands.spawn().id();
    //Spawning the levels
    commands.spawn_bundle(LdtkWorldBundle {
        ldtk_handle,
        ..Default::default()
    });
}

pub fn background_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play_looped(asset_server.load("Audio/Hyper.ogg"));
}

pub fn spawn_wall_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
    parent_query: Query<&Parent, Without<Wall>>,
    level_query: Query<(Entity, &Handle<LdtkLevel>)>,
    levels: Res<Assets<LdtkLevel>>,
) {
    /// Represents a wide wall that is 1 tile tall
    /// Used to spawn wall collisions
    #[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    // consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.for_each(|(&grid_coords, &Parent(parent))| {
        // the intgrid tiles' direct parents will be bevy_ecs_tilemap chunks, not the level
        // To get the level, you need their grandparents, which is where parent_query comes in
        if let Ok(&Parent(level_entity)) = parent_query.get(parent) {
            level_to_wall_locations
                .entry(level_entity)
                .or_insert(HashSet::new())
                .insert(grid_coords);
        }
    });

    if !wall_query.is_empty() {
        level_query.for_each(|(level_entity, level_handle)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let level = levels
                    .get(level_handle)
                    .expect("Level should be loaded by this point");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level
                    .level
                    .layer_instances
                    .clone()
                    .expect("Level asset should have layers")[0];

                println!("The gridsize is {:?}", grid_size);

                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right
                    // edge
                    for x in 0..width + 1 {
                        match (plate_start, level_walls.contains(&GridCoords { x, y })) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => plate_start = Some(x),
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut wall_rects: Vec<Rect<i32>> = Vec::new();
                let mut previous_rects: HashMap<Plate, Rect<i32>> = HashMap::new();

                // an extra empty row so the algorithm "terminates" the rects that touch the top
                // edge
                plate_stack.push(Vec::new());

                for (y, row) in plate_stack.iter().enumerate() {
                    let mut current_rects: HashMap<Plate, Rect<i32>> = HashMap::new();
                    for plate in row {
                        if let Some(previous_rect) = previous_rects.remove(plate) {
                            current_rects.insert(
                                *plate,
                                Rect {
                                    top: previous_rect.top + 1,
                                    ..previous_rect
                                },
                            );
                        } else {
                            current_rects.insert(
                                *plate,
                                Rect {
                                    bottom: y as i32,
                                    top: y as i32,
                                    left: plate.left,
                                    right: plate.right,
                                },
                            );
                        }
                    }

                    // Any plates that weren't removed above have terminated
                    wall_rects.append(&mut previous_rects.values().copied().collect());
                    previous_rects = current_rects;
                }

                // spawn colliders for every rectangle
                for wall_rect in wall_rects {
                    commands
                        .spawn()
                        .insert(CollisionShape::Cuboid {
                            half_extends: Vec3::new(
                                (wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                                (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                                0.,
                            ),
                            border_radius: None,
                        })
                        .insert(RigidBody::Static)
                        .insert(PhysicMaterial {
                            friction: 0.,
                            ..Default::default()
                        })
                        .insert(Transform::from_xyz(
                            (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32 / 2.,
                            (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32 / 2.,
                            0.,
                        ))
                        .insert(GlobalTransform::default())
                        // Making the collider a child of the level serves two purposes:
                        // 1. Adjusts the transforms to be relative to the level for free
                        // 2. the colliders will be despawned automatically when levels unload
                        .insert(Parent(level_entity));
                }
            }
        });
    }
}

pub fn world_rotation_system(
    input: Res<Input<KeyCode>>,
    mut gravity: ResMut<Gravity>,
    get_game_state: Res<GetGameState>,
    mut query: Query<&mut Transform, With<MainCamera>>,
) {
    if get_game_state.game_state == GameState::Overworld {
        //Rotate the camera
        if let Ok(mut camera_tf) = query.get_single_mut() {
            if input.just_pressed(KeyCode::R) {
                //Rotate the camera
                camera_tf.rotate(Quat::from_rotation_z(PI / 2.));
                //Change gravity
                let gravity = gravity.as_mut();
                if gravity.vector().y < 0. {
                    *gravity = Gravity::from(Vec3::new(2000., 0., 0.0));
                } else if gravity.vector().x > 0. {
                    *gravity = Gravity::from(Vec3::new(0., 2000., 0.0));
                } else if gravity.vector().y > 0. {
                    *gravity = Gravity::from(Vec3::new(-2000., 0., 0.0));
                } else if gravity.vector().x < 0. {
                    *gravity = Gravity::from(Vec3::new(0., -2000., 0.0));
                }
            }
            if input.just_pressed(KeyCode::T) {
                //Rotate the camera
                camera_tf.rotate(Quat::from_rotation_z(-PI / 2.));
                //Change gravity
                let gravity = gravity.as_mut();
                if gravity.vector().y < 0. {
                    *gravity = Gravity::from(Vec3::new(-2000., 0., 0.0));
                } else if gravity.vector().x < 0. {
                    *gravity = Gravity::from(Vec3::new(0., 2000., 0.0));
                } else if gravity.vector().y > 0. {
                    *gravity = Gravity::from(Vec3::new(2000., 0., 0.0));
                } else if gravity.vector().x > 0. {
                    *gravity = Gravity::from(Vec3::new(0., -2000., 0.0));
                }
            }
        }
    }
}

pub fn animate_sprite_system(
    //mut commands: Commands,
    time: Res<Time>,
    //game_textures: Res<GameTextures>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<
        (
            &mut AnimationTimer,
            &mut TextureAtlasSprite,
            &Handle<TextureAtlas>,
        ),
        With<AnimationTimer>,
    >,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}
