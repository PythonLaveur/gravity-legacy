#![allow(unused)]
use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioApp, AudioChannel, AudioPlugin, AudioSource};

use crate::components::*;
use crate::GameState;

pub struct GameAudioPlugin;

pub struct AudioState {
    pub bgm_handle: Handle<AudioSource>,
    pub spawn_handle: Handle<AudioSource>,
    pub slim_pot_handle: Handle<AudioSource>,
    pub move_handle: Handle<AudioSource>,
    pub slim_plat_handle: Handle<AudioSource>,
    pub pot_plat_handle: Handle<AudioSource>,
    pub slim_plosion_handle: Handle<AudioSource>,
    pub world_rotation_handle: Handle<AudioSource>,
    pub succeed_handle: Handle<AudioSource>,
    pub slim_pot_cd: f32,
    pub move_cd: f32,
    pub slim_plat_cd: f32,
    pub pot_plat_cd: f32,
    pub world_rotation_cd: f32,
}
pub struct SpawnEvent;
pub struct SlimPotEvent;
pub struct MoveEvent;
pub struct SlimPlatEvent;
pub struct PotPlatEvent;
pub struct SlimPlosionEvent;
pub struct WorldRotationEvent;
pub struct SucceedEvent;

struct Volume {
    volumeb: f32,
    volumes: f32,
}
/*impl AudioState {
    pub fn getvolume() -> f32 {
        &Self.volume
    }
    pub fn setvolume(&mut self,v:f32){
        &Self{self.volume}.=v;
}
}*/

pub struct BGM;
pub struct SFX;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AudioPlugin)
            .add_startup_system_to_stage(StartupStage::PreStartup, load_audio)
            .add_audio_channel::<BGM>()
            .add_audio_channel::<SFX>()
            .add_system(volume_control)
            .add_startup_system(start_bgm_music)
            .add_system(start_move_sfx)
            .add_system(start_slim_plat_sfx)
            .add_system(start_slim_pot_sfx)
            .add_system(start_pot_plat_sfx)
            .add_system(start_slim_plosion_sfx)
            .add_system(start_world_rotation_sfx)
            .add_system(start_succeed_sfx)
            .add_event::<SpawnEvent>()
            .add_event::<SlimPotEvent>()
            .add_event::<SlimPlatEvent>()
            .add_event::<PotPlatEvent>()
            .add_event::<MoveEvent>()
            .add_event::<SlimPlosionEvent>()
            .add_event::<WorldRotationEvent>()
            .add_event::<SucceedEvent>();
    }
}

fn volume_control(
    keyboard: Res<Input<KeyCode>>,
    b: Res<AudioChannel<BGM>>,
    s: Res<AudioChannel<SFX>>,
    mut v: ResMut<Volume>,
) {
    if keyboard.just_pressed(KeyCode::O) {
        v.volumeb += 0.10;
    }
    if keyboard.just_pressed(KeyCode::L) {
        v.volumeb -= 0.10;
    }
    v.volumeb = v.volumeb.clamp(0.0, 1.0);
    b.set_volume(v.volumeb);
    if keyboard.just_pressed(KeyCode::P) {
        v.volumes += 0.10;
    }
    if keyboard.just_pressed(KeyCode::M) {
        v.volumes -= 0.10;
    }
    v.volumes = v.volumes.clamp(0.0, 1.0);
    s.set_volume(v.volumes);
}

fn start_bgm_music(b: Res<AudioChannel<BGM>>, audio_state: Res<AudioState>) {
    b.play_looped(audio_state.bgm_handle.clone());
}

fn start_spawn_sfx(
    s: Res<AudioChannel<SFX>>,
    mut audio_state: ResMut<AudioState>,
    mut event: EventReader<SpawnEvent>,
) {
    if event.iter().count() > 0 {
        s.play(audio_state.spawn_handle.clone());
    }
}

fn start_move_sfx(
    s: Res<AudioChannel<SFX>>,
    mut audio_state: ResMut<AudioState>,
    mut event: EventReader<MoveEvent>,
) {
    if event.iter().count() > 0 {
        s.play(audio_state.move_handle.clone());
    }
}

fn start_slim_pot_sfx(
    s: Res<AudioChannel<SFX>>,
    mut audio_state: ResMut<AudioState>,
    mut event: EventReader<SlimPotEvent>,
) {
    if event.iter().count() > 0 {
        s.play(audio_state.slim_pot_handle.clone());
    }
}

fn start_slim_plat_sfx(
    s: Res<AudioChannel<SFX>>,
    mut audio_state: ResMut<AudioState>,
    mut event: EventReader<SlimPlatEvent>,
) {
    if event.iter().count() > 0 {
        s.play(audio_state.slim_plat_handle.clone());
    }
}

fn start_pot_plat_sfx(
    s: Res<AudioChannel<SFX>>,
    mut audio_state: ResMut<AudioState>,
    mut event: EventReader<PotPlatEvent>,
) {
    if event.iter().count() > 0 {
        s.play(audio_state.pot_plat_handle.clone());
    }
}
fn start_world_rotation_sfx(
    s: Res<AudioChannel<SFX>>,
    mut audio_state: ResMut<AudioState>,
    mut event: EventReader<WorldRotationEvent>,
) {
    if event.iter().count() > 0 {
        s.play(audio_state.world_rotation_handle.clone());
        println!("play world rotation");
    }
}

fn start_slim_plosion_sfx(
    s: Res<AudioChannel<SFX>>,
    mut audio_state: ResMut<AudioState>,
    mut event: EventReader<SlimPlosionEvent>,
) {
    if event.iter().count() > 0 {
        s.play(audio_state.slim_plosion_handle.clone());
    }
}

fn start_succeed_sfx(
    s: Res<AudioChannel<SFX>>,
    mut audio_state: ResMut<AudioState>,
    mut event: EventReader<SucceedEvent>,
) {
    if event.iter().count() > 0 {
        s.play(audio_state.succeed_handle.clone());
    }
}

fn load_audio(
    mut commands: Commands,

    //b: Res<AudioChannel<BGM>>,
    //s: Res<AudioChannel<SFX>>,
    assets: Res<AssetServer>,
) {
    let bgm_handle = assets.load("Audio/Hyper.ogg");
    let spawn_handle = assets.load("Audio/pot_plat.wav");
    let slim_pot_handle = assets.load("Audio/slim_pot.wav");
    let move_handle = assets.load("Audio/move.wav");
    let slim_plosion_handle = assets.load("Audio/slim_plosion.wav");
    let pot_plat_handle = assets.load("Audio/pot_plat.wav");
    let slim_plat_handle = assets.load("Audio/slim_plat.wav");
    let world_rotation_handle = assets.load("Audio/world_rotation.wav");
    let succeed_handle = assets.load("Audio/pot_plat.wav");

    //let bgm_channel = AudioChannel::new("bgm".to_string());

    //let sfx_channel = AudioChannel::new("sfx".to_string());
    let volumeb = 0.5;
    let volumes = 0.5;
    //b.set_volume(volume);

    //s.set_volume(volume);

    commands.insert_resource(AudioState {
        bgm_handle: bgm_handle,
        spawn_handle: spawn_handle,
        slim_pot_handle: slim_pot_handle,
        move_handle: move_handle,
        slim_plat_handle: slim_plat_handle,
        pot_plat_handle: pot_plat_handle,
        slim_plosion_handle: slim_plosion_handle,
        world_rotation_handle: world_rotation_handle,
        succeed_handle: succeed_handle,
        slim_pot_cd: 0.,
        move_cd: 0.,
        slim_plat_cd: 0.,
        pot_plat_cd: 0.,
        world_rotation_cd: 0.,
    });
    commands.insert_resource(Volume {
        volumeb: volumeb,
        volumes: volumes,
    });
}
