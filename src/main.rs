// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod audio;
mod camera;
mod demo;
#[cfg(feature = "dev")]
mod dev_tools;
mod external;
mod menus;
mod physics;
mod screens;
mod theme;

use bevy::{asset::AssetMetaCheck, prelude::*};
// use bevy_hanabi::HanabiPlugin;
use bevy_rapier2d::{prelude::*, rapier::prelude::IntegrationParameters};

// use crate::demo::particle_effect::ParticleEffectPlugin;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add core plugins.
        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Bevy Jam 6".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
            RapierPhysicsPlugin::<NoUserData>::default()
                .with_custom_initialization(
                    RapierContextInitialization::InitializeDefaultRapierContext {
                        integration_parameters: IntegrationParameters::default(),
                        rapier_configuration: RapierConfiguration {
                            gravity: Vec2::ZERO,
                            physics_pipeline_active: true,
                            query_pipeline_active: true,
                            scaled_shape_subdivision: 10,
                            force_update_from_transform_changes: false,
                        },
                    },
                )
                .with_default_system_setup(false),
        ));

        // app.add_plugins(HanabiPlugin);
        // Add other plugins.
        app.add_plugins((
            physics::plugin,
            camera::plugin,
            asset_tracking::plugin,
            audio::plugin,
            demo::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            menus::plugin,
            screens::plugin,
            theme::plugin,
            // ParticleEffectPlugin,
        ));

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        // Set up the `Pause` state.
        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));
        app.configure_sets(PostUpdate, PausableSystems.run_if(in_state(Pause(false))));

        // Configure Rapier.
        app.configure_sets(
            PostUpdate,
            (
                PhysicsSet::SyncBackend,
                PhysicsSet::StepSimulation,
                PhysicsSet::Writeback,
            )
                .chain()
                .before(TransformSystem::TransformPropagate),
        );

        app.add_systems(
            PostUpdate,
            (
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackend)
                    .in_set(PhysicsSet::SyncBackend)
                    .in_set(PausableSystems),
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::StepSimulation)
                    .in_set(PhysicsSet::StepSimulation)
                    .in_set(PausableSystems),
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::Writeback)
                    .in_set(PhysicsSet::Writeback)
                    .in_set(PausableSystems),
            ),
        );

        app.insert_resource(TimestepMode::Variable {
            max_dt: 1.0 / 60.0,
            time_scale: 1.0,
            substeps: 2,
        });
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

/// Whether or not the game is paused.
#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
struct Pause(pub bool);

/// A system set for systems that shouldn't run while the game is paused.
#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct PausableSystems;
