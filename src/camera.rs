use bevy::{
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

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Letterboxing>();

    app.add_systems(Startup, spawn_camera);
    app.add_systems(Update, update_letterbox);
}

/// Type for storing 2D sizes.
#[derive(Clone, Copy)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

/// Marker for the camera doing actual rendering to a texture.
#[derive(Component)]
pub struct GameplayCamera;

/// Marker for the camera rendering UI and the texture that the GameplayCamera renders to.
#[derive(Component)]
pub struct MainCamera;

/// Marker for UI node that contains the image that the gameplay is rendered to.
#[derive(Component)]
pub struct GameplayNode;

#[derive(Resource)]
pub struct Letterboxing {
    pub texture_size: Size<u32>,
    pub projection_size: Size<f32>,
    pub aspect_ratio: Size<f32>,
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

/// Calulates the letterboxed size for a certain screen size
/// and aspect ratio.
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
        Name::new("Gameplay Camera"),
        GameplayCamera,
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
        Name::new("Main Camera"),
        MainCamera,
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
