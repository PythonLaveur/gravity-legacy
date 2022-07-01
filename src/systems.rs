use crate::{components::Player, MAP_LDTK};
use crate::{
    components::*, GameState, GameTextures, GetGameState, PLAYER_IDLE, PLAYER_IDLE_COLUMN,
    PLAYER_IDLE_SIZE, PLAYER_JUMP, PLAYER_JUMP_COLUMN, PLAYER_JUMP_SIZE, PLAYER_WALK,
    PLAYER_WALK_COLUMN, PLAYER_WALK_SIZE, CollisionStatus,
};

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_kira_audio::Audio;

use std::collections::{HashMap, HashSet};

use heron::{CollisionShape, PhysicMaterial, RigidBody};

use heron::*;

const PI: f32 = 3.1415;
const ANIMATION_LEN: u32 = 6;

pub enum Animation {
    Explosion,
}

// region:     --- Ressources

pub struct WorldStatus {
    pub rotation: Vec2,
}
// endregion:  --- Ressources



pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    //Camera setup
    let mut camera = OrthographicCameraBundle::new_2d();
    //Offset
    /*TODO : SET CAMERA AT CURRENT LEVEL LOCATION USING THE FOLLOWING IDEA
        x = level_x + level_width/2 
        y = level_y + level_width/2 */
    camera.transform = Transform {
        translation: Vec3::new(200., 200., 1000.),
        ..default()
    };
    commands.spawn_bundle(camera).insert(MainCamera);

    //Add ressources
    commands.insert_resource(GetGameState {
        game_state: GameState::StartMenu,
        level_index: 0,
        respawn_level: 0,
        player_spawned: false,
    });
    commands.insert_resource(CollisionStatus::default());

    //Enable to recall the setup but ignoring the code before
    asset_server.watch_for_changes().unwrap();

    //Load the levels
    let ldtk_handle = asset_server.load(MAP_LDTK);

    //insert ressource
    commands.insert_resource(WorldStatus{rotation: Vec2::new(1., 0.)});


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

    // create explosion texture atlas
    let explosion_atlas = asset_server.load("Sprites/Items/Fruits/Collected.png");
    let texture_atlas = TextureAtlas::from_grid(explosion_atlas, Vec2::new(32., 32.), 6, 1);
    let explosion = texture_atlases.add(texture_atlas);

    let game_textures = GameTextures {
        player_idle,
        player_jump,
        player_walk,
        explosion,
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
                        .insert(Parent(level_entity))
                        .insert(Wall);
                }
            }
        });
    }
}
pub fn world_rotation_system(
    input: Res<Input<KeyCode>>,
    mut gravity: ResMut<Gravity>,
    mut world_status: ResMut<WorldStatus>,
    mut query: Query<&mut Transform, With<MainCamera>>
) {
    //Rotate the camera
    if let Ok(mut camera_tf) = query.get_single_mut() {
        if input.just_pressed(KeyCode::R) {
            //Rotate the camera
            camera_tf.rotate(Quat::from_rotation_z(PI / 2.));
            //Change gravity
            let gravity = gravity.as_mut();
            if gravity.vector().y < 0. {
                *gravity = Gravity::from(Vec3::new(2000., 0., 0.0));
                world_status.rotation = Vec2::new(0., 1.);
            } else
            if gravity.vector().x > 0. {
                *gravity = Gravity::from(Vec3::new(0., 2000., 0.0));
                world_status.rotation = Vec2::new(-1., 0.);
            } else
            if gravity.vector().y > 0. {
                *gravity = Gravity::from(Vec3::new(-2000., 0., 0.0));
                world_status.rotation = Vec2::new(0., -1.);
            } else
            if gravity.vector().x < 0. {
                *gravity = Gravity::from(Vec3::new(0., -2000., 0.0));
                world_status.rotation = Vec2::new(1., 0.);

            }

        }
        if input.just_pressed(KeyCode::T) {
            //Rotate the camera
            camera_tf.rotate(Quat::from_rotation_z(-PI / 2.));
            //Change gravity
            let gravity = gravity.as_mut();
            if gravity.vector().y < 0. {
                *gravity = Gravity::from(Vec3::new(-2000., 0., 0.0));
                world_status.rotation = Vec2::new(0., -1.);
            } else
            if gravity.vector().x < 0. {
                *gravity = Gravity::from(Vec3::new(0., 2000., 0.0));
                world_status.rotation = Vec2::new(-1., 0.);
            } else 
            if gravity.vector().y > 0. {
                *gravity = Gravity::from(Vec3::new(2000., 0., 0.0));
                world_status.rotation = Vec2::new(0., 1.);
            } else 
            if gravity.vector().x > 0. {
                *gravity = Gravity::from(Vec3::new(0., -2000., 0.0));
                world_status.rotation = Vec2::new(1., 0.);
            }
        }

    }
}

pub fn player_collision_with_pot (
    pot_query: Query<Entity, With<Pot>>,
    wall_query: Query<Entity, With<Wall>>,
    mut player: Query<Entity, With<Player>>,
    mut collisions: EventReader<CollisionEvent>,
    mut collision_status: ResMut<CollisionStatus>,
) {
    for collision in collisions.iter() {
        match collision {
            CollisionEvent::Started(collider_a, collider_b ) => {

                let mut player_normal = Vec3::new(0.,0.,0.); 

                if let Ok(mut player) = player.get_mut(collider_a.rigid_body_entity()) {
                    
                    if pot_query.get(collider_b.rigid_body_entity()).is_ok() {
                        println!("player normal to pot: {:?}", collider_a.normals());
                        //println!("pot normal: {:?}", collider_b.normals());
                        player_normal = *collider_a.normals().get(0).unwrap();
                    }
                    else if wall_query.get(collider_b.rigid_body_entity()).is_ok() {
                        println!("player normal to wall: {:?}", collider_a.normals());
                        //println!("wall normal: {:?}", collider_b.normals());
                        player_normal = *collider_a.normals().get(0).unwrap();
                    }

                }
                else if let Ok(mut player) = player.get_mut(collider_b.rigid_body_entity()) {
                    
                    if pot_query.get(collider_a.rigid_body_entity()).is_ok() {
                        println!("player normal to pot: {:?}", collider_b.normals());
                        //println!("pot normal: {:?}", collider_a.normals());
                        player_normal = *collider_b.normals().get(0).unwrap();
                    }
                    else if wall_query.get(collider_a.rigid_body_entity()).is_ok() {
                        println!("player normal to wall: {:?}", collider_b.normals());
                        //println!("wall normal: {:?}", collider_a.normals());
                        player_normal = *collider_b.normals().get(0).unwrap();
                    }
                }
                if player_normal == Vec3::new(0., -1., 0.) {
                    collision_status.top = true;
                }
                if player_normal == Vec3::new(0., 1., 0.) {
                    collision_status.bottom = true;
                }
                if player_normal == Vec3::new(-1., 0., 0.) {
                    collision_status.right = true;
                }
                if player_normal == Vec3::new(1., 0., 0.) {
                    collision_status.left = true;
                }

                if (collision_status.bottom && collision_status.top)||(collision_status.right && collision_status.left) {
                    println!("Game Over")
                }

            }
            CollisionEvent::Stopped(collider_aa, collider_bb )=>{
                let mut player_normal = Vec3::new(0.,0.,0.); 

                if let Ok(mut player) = player.get_mut(collider_aa.rigid_body_entity()) {
                    
                    if pot_query.get(collider_bb.rigid_body_entity()).is_ok() {
                        player_normal = *collider_aa.normals().get(0).unwrap();
                    }
                    else if wall_query.get(collider_bb.rigid_body_entity()).is_ok() {
                        player_normal = *collider_aa.normals().get(0).unwrap();
                    }

                }
                else if let Ok(mut player) = player.get_mut(collider_bb.rigid_body_entity()) {
                    
                    if pot_query.get(collider_aa.rigid_body_entity()).is_ok() {
                        player_normal = *collider_bb.normals().get(0).unwrap();
                    }
                    else if wall_query.get(collider_aa.rigid_body_entity()).is_ok() {
                        player_normal = *collider_bb.normals().get(0).unwrap();
                    }
                }
                if player_normal == Vec3::new(0., -1., 0.) {
                    collision_status.top = false;
                }
                if player_normal == Vec3::new(0., 1., 0.) {
                    collision_status.bottom = false;
                }
                if player_normal == Vec3::new(-1., 0., 0.) {
                    collision_status.right = false;
                }
                if player_normal == Vec3::new(1., 0., 0.) {
                    collision_status.left = false;
                }
            }
        }
    }
}

pub fn player_collision_with_spikes (
    mut commands: Commands,
    mut get_game_state: ResMut<GetGameState>,
    mut world_status: ResMut<WorldStatus>,
    mut gravity: ResMut<Gravity>,
    spikes_query: Query<Entity, With<Spikes>>,
    player: Query<(Entity, &Transform), With<Player>>,
    mut collisions: EventReader<CollisionEvent>,
    mut query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>
) {
    for collision in collisions.iter() {
        match collision {
            CollisionEvent::Started(collider_a, collider_b ) => {
                if let Ok((player, tf)) = player.get(collider_a.rigid_body_entity()) {
                    
                    if spikes_query.get(collider_b.rigid_body_entity()).is_ok() {
                        // Despawn the player
                        commands.entity(player).despawn();
                        get_game_state.player_spawned = false;
                        get_game_state.respawn_level = 1;

                        // Spawn the animation
                       commands.spawn().insert(AnimationToSpawn(tf.translation.clone(), Animation::Explosion));
                        
                        // Spawn the current level after a delai (fade out)
                        //get_game_state.level_index -= 1;
                        commands.insert_resource(LevelSelection::Index(get_game_state.level_index));
                        
                        //Reset the gravity
                        if let Ok(mut camera_tf) = query.get_single_mut() { 
                            camera_tf.rotation = Quat::from_axis_angle(Vec3::new(0., 0., 1.), 0.);
                        }
                        *gravity = Gravity::from(Vec3::new(0., -2000., 0.0));
                        world_status.rotation = Vec2::new(1., 0.)
                }
                
                }
                if let Ok((player, tf)) = player.get(collider_b.rigid_body_entity()) {
                    if spikes_query.get(collider_a.rigid_body_entity()).is_ok() {
                        // Despawn the player
                        commands.entity(player).despawn();
                        get_game_state.player_spawned = false;
                        get_game_state.respawn_level = 1;

                        // Spawn the animation
                       commands.spawn().insert(AnimationToSpawn(tf.translation.clone(), Animation::Explosion));
                        
                        // Spawn the current level after a delai (fade out)
                        //get_game_state.level_index -= 1;
                        commands.insert_resource(LevelSelection::Index(get_game_state.level_index));
                        
                        //Reset the gravity
                        if let Ok(mut camera_tf) = query.get_single_mut() { 
                            camera_tf.rotation = Quat::from_axis_angle(Vec3::new(0., 0., 1.), 0.);
                        }
                        *gravity = Gravity::from(Vec3::new(0., -2000., 0.0));
                        world_status.rotation = Vec2::new(1., 0.)
                    }
                }
            }
            CollisionEvent::Stopped(_, _) => (),
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

pub fn player_succeed(
    mut commands: Commands,
    key: Query<Entity, With<Key>>,
    player: Query<(Entity, &Transform), With<Player>>,
    mut collisions: EventReader<CollisionEvent>,
    mut world_status: ResMut<WorldStatus>,
    mut gravity: ResMut<Gravity>,
    mut get_game_state: ResMut<GetGameState>,
    mut query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>
) {
    for collision in collisions.iter() {
        match collision {
            CollisionEvent::Started(collider_a, collider_b ) => {
                if let Ok((player, tf)) = player.get(collider_a.rigid_body_entity()) {
                    if key.get(collider_b.rigid_body_entity()).is_ok() {
                            // Despawn the player
                            commands.entity(player).despawn();
                            get_game_state.player_spawned = false;

                            // Spawn the animation
                           commands.spawn().insert(AnimationToSpawn(tf.translation.clone(), Animation::Explosion));
                            
                            // Spawn the next level after a delai (fade out)
                            get_game_state.level_index += 1;
                            commands.insert_resource(LevelSelection::Index(get_game_state.level_index));
                            
                            //Reset the gravity
                            if let Ok(mut camera_tf) = query.get_single_mut() { 
                                camera_tf.rotation = Quat::from_axis_angle(Vec3::new(0., 0., 1.), 0.);
                            }
                            *gravity = Gravity::from(Vec3::new(0., -2000., 0.0));
                            world_status.rotation = Vec2::new(1., 0.)
                    }
                }
                if let Ok((player, tf)) = player.get(collider_b.rigid_body_entity()) {
                    if key.get(collider_a.rigid_body_entity()).is_ok() {
                        // Despawn the player
                        commands.entity(player).despawn();
                        get_game_state.player_spawned = false;

                        // Spawn the animation
                       commands.spawn().insert(AnimationToSpawn(tf.translation.clone(), Animation::Explosion));
                        
                        // Spawn the next level after a delai (fade out)
                        get_game_state.level_index += 1;
                        commands.insert_resource(LevelSelection::Index(get_game_state.level_index));
                        
                        //Reset the gravity
                        if let Ok(mut camera_tf) = query.get_single_mut() { 
                            camera_tf.rotation = Quat::from_axis_angle(Vec3::new(0., 0., 1.), 0.);
                        }
                        *gravity = Gravity::from(Vec3::new(0., -2000., 0.0));
                        world_status.rotation = Vec2::new(1., 0.)
                    }
                }
            }
            CollisionEvent::Stopped(_, _) => (),
        }
    }
}

pub fn animation_to_spawn_system(
    audio: Res<Audio>,
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    query: Query<(Entity, &AnimationToSpawn), With<AnimationToSpawn>>,
) {
    for (animation_spawn_entity, animation_to_spawn) in query.iter() {
        // animation above the player
        let translation = Vec3::new(
            animation_to_spawn.0.x,
            animation_to_spawn.0.y,
            animation_to_spawn.0.z + 1.,
        );

        //TODO : Choose the correct texture with animation_to_spawn.1 which is an enum Animation
        let texture_atlas = game_textures.explosion.clone();
        let max_index = 6;

        // spawn the sprite
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas,
                transform: Transform {
                    translation,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(FxAnimationTimer(Timer::from_seconds(0.05, true), max_index));

        // despawn the explosionToSpawn
        commands.entity(animation_spawn_entity).despawn();

        //Play level succeed audio
        //audio.play(asset_server.load(EXPLOSION_AUDIO));
    }
}

pub fn animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut FxAnimationTimer, &mut TextureAtlasSprite), With<FxAnimationTimer>>,
) {
    for (entity, mut timer, mut sprite) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index += 1; // move to next frame
            if sprite.index >= timer.1 {
                commands.entity(entity).despawn()
            }
        }
    }
}
