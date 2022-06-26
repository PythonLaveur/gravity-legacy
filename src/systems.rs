use crate::MAP_LDTK;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
//use heron::*;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera = OrthographicCameraBundle::new_2d();
    commands.spawn_bundle(camera);

    //Enable to recall the setup but ignoring the code before
    asset_server.watch_for_changes().unwrap();

    //Load the levels
    let ldtk_handle = asset_server.load(MAP_LDTK);

    //Spawning the levels
    commands.spawn_bundle(LdtkWorldBundle {
        ldtk_handle,
        ..Default::default()
    });
}
