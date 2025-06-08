use bevy::prelude::*;
use bevy_enoki::{ParticleEffectHandle, ParticleSpawner, prelude::OneShot};
pub struct ParticleEffectPlugin;

impl Plugin for ParticleEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut cmd: Commands, server: Res<AssetServer>) {
    // bring in your own effect asset from a ron file
    // (hot reload by default)
    // add this when the Particle explodes and you want to play the effect!
    cmd.spawn((
        ParticleSpawner::default(),
        // the effect components holds the baseline
        // effect asset.
        ParticleEffectHandle(server.load("example.explosion.ron")),
        OneShot::Despawn,
    ));
}
