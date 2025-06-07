//! The level selection screen.

use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
};

use crate::{
    demo::level::{Level, level_loading::LevelAssets},
    menus::Menu,
    screens::{Screen, gameplay::SelectedLevel},
    theme::{prelude::InteractionPalette, widget},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Levels), spawn_levels_screen);
}

#[derive(Component)]
struct LevelButton(Level);

fn level_button<E, B, M, I>(text: impl Into<String>, level: Level, action: I) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let action = IntoObserverSystem::into_system(action);

    let none = Color::Srgba(Srgba::hex("#0f0f0f").unwrap());
    let hovered = Color::Srgba(Srgba::hex("#000000").unwrap());
    let pressed = Color::Srgba(Srgba::hex("#000000").unwrap());
    let text_color = Color::Srgba(Srgba::hex("#ffffff").unwrap());

    (
        Name::new("Level Button"),
        Node::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent
                .spawn((
                    LevelButton(level),
                    Node {
                        width: Val::Px(90.0),
                        height: Val::Px(60.0),
                        margin: UiRect::all(Val::Px(2.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Px(10.0)),
                    Name::new("Button Inner"),
                    Button,
                    BackgroundColor(none),
                    InteractionPalette {
                        none,
                        hovered,
                        pressed,
                    },
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        TextFont::from_font_size(40.0),
                        TextColor(text_color),
                        Pickable::IGNORE,
                    )],
                ))
                .observe(action);
        })),
    )
}

fn spawn_levels_screen(mut commands: Commands, level_assets: Res<LevelAssets>) {
    let num_default_levels = level_assets.default.len();

    commands.spawn((
        widget::ui_root("Levels Screen"),
        GlobalZIndex(2),
        StateScoped(Menu::Levels),
        children![
            widget::header("Levels"),
            (
                Name::new("Levels Grid"),
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_content: AlignContent::Start,
                    width: Val::Px(470.0),
                    height: Val::Px(300.0),
                    ..default()
                },
                Pickable::IGNORE,
                Children::spawn(SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                    for i in 0..num_default_levels {
                        parent.spawn(level_button(
                            i.to_string(),
                            Level::Default(i),
                            enter_gameplay_screen,
                        ));
                    }
                })),
            ),
            widget::button("Back", go_back)
        ],
    ));
}

fn enter_gameplay_screen(
    trigger: Trigger<Pointer<Click>>,
    level_button_query: Query<&LevelButton>,
    mut selected_level: ResMut<SelectedLevel>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    let entity = trigger.target();
    let level_button = level_button_query.get(entity).unwrap();

    selected_level.0 = Some(level_button.0.clone());

    next_screen.set(Screen::Gameplay);
}

fn go_back(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
