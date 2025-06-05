// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod asset_tracking;
mod audio;
mod demo;
#[cfg(feature = "dev")]
mod dev_tools;
mod external;
mod menus;
mod screens;
mod theme;

use bevy::{
    asset::AssetMetaCheck,
    image::{TextureFormatPixelInfo, Volume},
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    window::{PrimaryWindow, WindowResized},
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_rapier2d::{prelude::*, rapier::prelude::IntegrationParameters};

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
            RapierDebugRenderPlugin::default(),
        ));

        // Development plugins.
        app.add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        });
        app.add_plugins(WorldInspectorPlugin::new());

        // Add other plugins.
        app.add_plugins((
            asset_tracking::plugin,
            audio::plugin,
            demo::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            menus::plugin,
            screens::plugin,
            theme::plugin,
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

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);
        app.add_systems(Update, update_letterbox);
        app.init_resource::<Letterboxing>();

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

#[derive(Component)]
struct GameplayNode;

#[derive(Resource)]
struct Letterboxing {
    texture_size: Size<u32>,
    projection_size: Size<f32>,
    aspect_ratio: Size<f32>,
}

impl Default for Letterboxing {
    fn default() -> Self {
        Self {
            texture_size: Size::new(1920, 1080),
            projection_size: Size::new(1920.0 / 1.5, 1080.0 / 1.5),
            aspect_ratio: Size::new(16.0, 9.0),
        }
    }
}

#[derive(Clone, Copy)]
struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

fn letterbox(size: Size<f32>, aspect_ratio: Size<f32>) -> Size<f32> {
    let sx = size.width / aspect_ratio.width;
    let sy = size.height / aspect_ratio.height;
    let s = sx.min(sy);

    Size::new(s * aspect_ratio.width, s * aspect_ratio.height)
}

fn spawn_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    letterboxing: Res<Letterboxing>,
) {
    let size = Extent3d {
        width: letterboxing.texture_size.width,
        height: letterboxing.texture_size.height,
        depth_or_array_layers: 1,
    };

    let format = TextureFormat::bevy_default();

    let image = Image {
        data: Some(vec![0; size.volume() * format.pixel_size()]),
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    let image_handle = images.add(image);

    commands.spawn((
        Name::new("Main Camera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::Fixed {
                width: letterboxing.projection_size.width,
                height: letterboxing.projection_size.height,
            },
            ..OrthographicProjection::default_2d()
        }),
        Camera {
            order: 1,
            target: RenderTarget::Image(image_handle.clone().into()),
            ..default()
        },
    ));

    commands.spawn((
        Name::new("Display Camera"),
        Camera2d,
        IsDefaultUiCamera,
        RenderLayers::layer(1),
    ));

    let window = window_query.single().unwrap();
    let window_size = Size::new(window.width(), window.height());
    let size = letterbox(window_size, letterboxing.aspect_ratio);

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
        children![(
            GameplayNode,
            Node {
                width: Val::Px(size.width),
                height: Val::Px(size.height),
                ..default()
            },
            BackgroundColor(Color::srgb(1.0, 0.0, 0.0)),
            children![ImageNode::new(image_handle)]
        )],
        RenderLayers::layer(1),
    ));
}

fn update_letterbox(
    mut events: EventReader<WindowResized>,
    mut gameplay_node_query: Query<&mut Node, With<GameplayNode>>,
    letterboxing: Res<Letterboxing>,
) {
    for event in events.read() {
        let window_size = Size::new(event.width, event.height);
        let size = letterbox(window_size, letterboxing.aspect_ratio);

        let mut node = gameplay_node_query.single_mut().unwrap();
        node.width = Val::Px(size.width);
        node.height = Val::Px(size.height);
    }
}
