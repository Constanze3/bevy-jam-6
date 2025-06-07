use bevy::{platform::collections::HashMap, prelude::*};

use crate::{asset_tracking::LoadResource, demo::level::level_data::LevelData};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.load_resource::<LevelHandles>();
    app.add_systems(Update, initialize_level_assets);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelHandles {
    #[dependency]
    pub default: Vec<Handle<LevelData>>,
    #[dependency]
    pub custom: Vec<Handle<LevelData>>,
}

impl FromWorld for LevelHandles {
    fn from_world(world: &mut World) -> Self {
        let default_levels: Vec<&'static str> = vec!["0", "1"];
        let custom_levels: Vec<&'static str> = vec![];

        let assets = world.resource::<AssetServer>();

        let default = default_levels
            .into_iter()
            .map(|lv| {
                let path = format!("levels/default/{}.ron", lv);
                assets.load(path)
            })
            .collect();

        let custom = custom_levels
            .into_iter()
            .map(|lv| {
                let path = format!("levels/custom/{}.ron", lv);
                assets.load(path)
            })
            .collect();

        Self { default, custom }
    }
}

#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    pub default: Vec<Handle<LevelData>>,
    pub custom: HashMap<String, Handle<LevelData>>,
}

// Initializes the LevelAssets resource from the raw LevelHandles resource.
fn initialize_level_assets(
    mut events: EventReader<AssetEvent<LevelHandles>>,
    mut commands: Commands,
    mut level_handles_assets: ResMut<Assets<LevelHandles>>,
    levels: Res<Assets<LevelData>>,
) {
    for event in events.read() {
        if let AssetEvent::LoadedWithDependencies { id } = event {
            let level_handles = level_handles_assets.get_mut(*id).unwrap();

            let default = std::mem::take(&mut level_handles.default);
            let custom = std::mem::take(&mut level_handles.default);

            // Load default levels as a sorted Vec<Handle<LevelData>>.
            let map_default = |handles: Vec<Handle<LevelData>>| {
                let mut folder = handles.clone();

                // Each level should be named with numbers.
                // This sorts them by their name.
                folder.sort_by(|h1, h2| {
                    let parse_name = |level: &LevelData| {
                        level
                            .name
                            .parse()
                            .expect("Default level names should be numbers.")
                    };

                    let level1 = levels.get(h1).unwrap();
                    let id1: usize = parse_name(level1);

                    let level2 = levels.get(h2).unwrap();
                    let id2: usize = parse_name(level2);

                    id1.cmp(&id2)
                });

                folder
            };

            // Load custom levels as a HashMap<String, Handle<LevelData>>.
            let map_custom = |handles: Vec<Handle<LevelData>>| {
                handles
                    .iter()
                    .map(|h| {
                        let level = levels.get(h).unwrap();

                        (level.name.clone(), h.clone())
                    })
                    .collect()
            };

            commands.remove_resource::<LevelHandles>();
            commands.insert_resource(LevelAssets {
                default: map_default(default),
                custom: map_custom(custom),
            });
        }
    }
}
