use bevy::prelude::*;
use serde_json::*;

use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;
use std::*;

use crate::components::{AnimationTimer, Player, Slime, SpriteSize};
use crate::slime_collision::Side;
use crate::*;
use crate::systems::WorldStatus;

pub struct SlimePlugin;

impl Plugin for SlimePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PostStartup, slime_spawn_system)
            .add_system(player_keyboard_event_system)
            .add_system(slime_sprite_update_system);
            //.add_system(slime_movement_system);
    }
}

fn read_user_from_file<P: AsRef<Path>>(path: P) -> io::Result<Value> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file
    let u = serde_json::from_reader(reader)?;

    Ok(u)
}

/*mut windows: ResMut<Windows>

  //window
  let window = windows.get_primary_mut().unwrap();
  (window.width(), window.height());

*/

fn slime_spawn_system(mut commands: Commands, game_textures: Res<GameTextures>) {
    let v: Value = read_user_from_file("assets/test/simplified/Level_0/data.json").unwrap();
    let x = v["entities"]["Player"][0]["x"].as_f64().unwrap() as f32;
    let y = v["entities"]["Player"][0]["y"].as_f64().unwrap() as f32;
    //let window = windows.get_primary_mut().unwrap();
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: game_textures.player_idle.clone(),
            transform: Transform {
                scale: Vec3::new(PLAYER_SCALE, PLAYER_SCALE, 1.),
                translation: Vec3::new(
                    x - 128. + PLAYER_IDLE_SIZE.0 / 2.,
                    -(y - 112. - 128.) + PLAYER_IDLE_SIZE.1 / 2.,
                    10.,
                ),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Slime {
            side: Side::Bottom,
            side_before: Side::Bottom,
            lenght_on_side: PLAYER_IDLE_SIZE.0 * PLAYER_SCALE,
            depth: 0.,
            is_jumping: false,
            is_walking: false,
            need_new_sprite: false,
            stop_timer: 0,
        })
        .insert(Player)
        .insert(Velocity::from_linear(Vec3::ZERO))
        .insert(SpriteSize {
            val: Vec2::new(
                PLAYER_IDLE_SIZE.0 * PLAYER_SCALE,
                PLAYER_IDLE_SIZE.1 * PLAYER_SCALE,
            ),
        })
        .insert(AnimationTimer(Timer::from_seconds(0.15, true)))
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(
                PLAYER_IDLE_SIZE.0 * PLAYER_SCALE / 2.,
                PLAYER_IDLE_SIZE.1 * PLAYER_SCALE / 2.,
                0.,
            ),
            border_radius: None,
        })
        .insert(RigidBody::Dynamic)
        .insert(RotationConstraints::lock())
        .insert(PhysicMaterial::default());
}

fn slime_movement_system(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform, &mut Slime)>,
) {
    for (velocity, mut transform, mut slime) in query.iter_mut() {
        let translation = &mut transform.translation;
        if slime.stop_timer != 0 {
            slime.stop_timer -= 1;
        } else {
            match slime.side {
                Side::Left => {
                    translation.y += velocity.linear.y * time.delta_seconds() * BASE_SPEED
                }
                Side::Right => {
                    translation.y += velocity.linear.y * time.delta_seconds() * BASE_SPEED
                }
                Side::Top => translation.x += velocity.linear.x * time.delta_seconds() * BASE_SPEED,
                Side::Bottom => {
                    translation.x += velocity.linear.x * time.delta_seconds() * BASE_SPEED
                }
                Side::Inside => (),
            }
        }
    }
}

fn slime_sprite_update_system(
    game_textures: Res<GameTextures>,
    mut query: Query<(
        &mut Transform,
        &mut Handle<TextureAtlas>,
        &mut TextureAtlasSprite,
        &Velocity,
        &mut SpriteSize,
        &mut Slime,
    )>,
) {
    for (mut transform, mut texture_atlas, mut sprite, velocity, mut size, mut slime) in
        query.iter_mut()
    {
        let current_side = slime.side.numerize();
        let side_before = slime.side_before.numerize();
        if current_side != side_before {
            println!("il y a eu changement de coté");
            let offset1 = (size.val.y - size.val.x) / 2. + slime.depth;
            let offset2 = (size.val.x - size.val.y) / 2. + SECURITY_DISTANCE;
            // saut
            /*if slime.is_jumping {

            }
            //changement d'angle
            else*/
            if current_side == (((side_before - 1) % 4) + 4) % 4 {
                println!("on va vers la gauche");
                transform.rotation *= Quat::from_rotation_z(-90_f32.to_radians());
                match slime.side_before {
                    Side::Left => {
                        println!("gauche vers haut ");
                        println!("{:?}", transform.translation);
                        transform.translation.y -= offset1;
                        transform.translation.x += offset2;
                        println!("{:?}", transform.translation);
                    }
                    Side::Right => {
                        println!("droite vers bas");
                        println!("{:?}", transform.translation);
                        transform.translation.y += offset1;
                        transform.translation.x -= offset2;
                        println!("{:?}", transform.translation);
                    }
                    Side::Top => {
                        println!("haut vers droite");
                        println!("{:?}", transform.translation);
                        transform.translation.x -= offset1;
                        transform.translation.y -= offset2;
                        println!("{:?}", transform.translation);
                    }
                    Side::Bottom => {
                        println!("bas vers gauche");
                        println!("{:?}", transform.translation);
                        transform.translation.x += offset1;
                        transform.translation.y += offset2;
                        println!("{:?}", transform.translation);
                    }
                    Side::Inside => (),
                }
            } else {
                println!("on va vers la droite");
                transform.rotation *= Quat::from_rotation_z(90_f32.to_radians());
                match slime.side_before {
                    Side::Left => {
                        println!("gauche vers bas ");
                        println!("{:?}", transform.translation);
                        transform.translation.y += offset1;
                        transform.translation.x += offset2;
                        println!("{:?}", transform.translation);
                    }
                    Side::Right => {
                        println!("droite vers haut");
                        println!("{:?}", transform.translation);
                        transform.translation.y -= offset1;
                        transform.translation.x -= offset2;
                        println!("{:?}", transform.translation);
                    }
                    Side::Top => {
                        println!("haut vers gauche");
                        println!("{:?}", transform.translation);
                        transform.translation.x += offset1;
                        transform.translation.y -= offset2;
                        println!("{:?}", transform.translation);
                    }
                    Side::Bottom => {
                        println!("bas vers droite");
                        println!("{:?}", transform.translation);
                        transform.translation.x -= offset1;
                        transform.translation.y += offset2;
                        println!("{:?}", transform.translation);
                    }
                    Side::Inside => (),
                }
            }
            match slime.side {
                Side::Left => slime.side_before = Side::Left,
                Side::Right => slime.side_before = Side::Right,
                Side::Top => slime.side_before = Side::Top,
                Side::Bottom => slime.side_before = Side::Bottom,
                Side::Inside => (),
            }
        }
        if slime.is_walking {
            if slime.need_new_sprite {
                sprite.index = 0;
                *texture_atlas = game_textures.player_walk.clone();
                *size = SpriteSize {
                    val: Vec2::new(
                        PLAYER_WALK_SIZE.0 * PLAYER_SCALE,
                        PLAYER_WALK_SIZE.1 * PLAYER_SCALE,
                    ),
                };
                slime.need_new_sprite = false;
            }
            //vers la droite
            if velocity.linear.x > 0. {
                if current_side == 2 {
                    sprite.flip_x = true;
                } else {
                    sprite.flip_x = false;
                }
            }
            //vers la gauche
            else if velocity.linear.x < 0. {
                if current_side == 0 {
                    sprite.flip_x = true;
                } else {
                    sprite.flip_x = false;
                }
            }
            //vers le haut
            else if velocity.linear.y > 0. {
                if current_side == 3 {
                    sprite.flip_x = true;
                } else {
                    sprite.flip_x = false;
                }
            }
            //vers le bas
            else {
                sprite.flip_x = current_side == 1;
            }
        } else if slime.need_new_sprite {
            sprite.index = 0;
            *texture_atlas = game_textures.player_idle.clone();
            *size = SpriteSize {
                val: Vec2::new(
                    PLAYER_IDLE_SIZE.0 * PLAYER_SCALE,
                    PLAYER_IDLE_SIZE.1 * PLAYER_SCALE,
                ),
            };
            slime.need_new_sprite = false;
        }
    }
}

pub fn player_keyboard_event_system(
    kb: Res<Input<KeyCode>>,
    world_status: Res<WorldStatus>,
    mut query: Query<(&mut Velocity, &mut Slime), With<Player>>,
) {
    if let Ok((mut velocity, mut slime)) = query.get_single_mut() {
        if !slime.is_jumping {
            let curent_side = slime.side.numerize();
            let left = if kb.pressed(KeyCode::Left) {
                if !slime.is_walking && curent_side != 1 && curent_side != 3 {
                    slime.need_new_sprite = true;
                    slime.is_walking = true;
                }
                1.
            } else {0.};
            let right = if kb.pressed(KeyCode::Right) {
                if !slime.is_walking && curent_side != 1 && curent_side != 3 {
                    slime.need_new_sprite = true;
                    slime.is_walking = true;
                }
                1.
            } else {
                0.
            };

            if world_status.rotation.y == 0. { 
                velocity.linear.x = (right - left) * 200. * world_status.rotation.x; 
            }
            else {
                velocity.linear.y = (right - left) * 200. * world_status.rotation.y; 
            }
            
        }
    
            //TODO : Vérifier l'utilité de ce qui suit
            /*
            velocity.linear.y = if kb.pressed(KeyCode::Down) {
                if !slime.is_walking && curent_side != 0 && curent_side != 2 {
                    slime.need_new_sprite = true;
                    slime.is_walking = true;
                }
                -1.
            } else if kb.pressed(KeyCode::Up) {
                if !slime.is_walking && curent_side != 0 && curent_side != 2 {
                    slime.need_new_sprite = true;
                    slime.is_walking = true;
                }
                1.
            } else {
                0.
            };
        }*/

        //Idle detection
        if velocity.linear.x == 0. && velocity.linear.y == 0. && slime.is_walking {
            slime.need_new_sprite = true;
            slime.is_walking = false;
        }
        /*if kb.pressed(KeyCode::Space) { //saut
            slime.is_jumping = true;
            if slime.side == 0 || slime.side == 2 {
                let sign = -(slime.side -1) as f32;
                velocity.y = sign;
                velocity.x = 0.;
            }
            else {
                let sign = (slime.side -2) as f32;
                velocity.x = sign;
                velocity.y = 0.;
            }
        }*/
    }
}
