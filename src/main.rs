use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::prelude::*;
use rand::Rng;
use std::collections::HashMap;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.10, 0.08)))
        .insert_resource(PlayerInventory::default())
        .insert_resource(CraftingRecipes::default())
        .insert_resource(MouseLookState::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tirnanog - Survival Prototype".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                mouse_look,
                update_camera,
                player_attack,
                handle_tree_fall,
                crafting_input,
                inventory_debug,
            ),
        )
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct FollowCamera;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Gatherable {
    resource: ResourceType,
    amount: u32,
    requires_tool: Option<ToolType>,
}

#[derive(Component)]
struct Tree {
    health: i32,
    standing: bool,
}

#[derive(Component)]
struct VelocityY(f32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ResourceType {
    Stick,
    Rock,
    Wood,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ToolType {
    StoneAxe,
}

#[derive(Resource, Default)]
struct PlayerInventory {
    resources: HashMap<ResourceType, u32>,
    tools: HashMap<ToolType, u32>,
}

impl PlayerInventory {
    fn add_resource(&mut self, resource: ResourceType, amount: u32) {
        *self.resources.entry(resource).or_default() += amount;
    }

    fn add_tool(&mut self, tool: ToolType) {
        *self.tools.entry(tool).or_default() += 1;
    }

    fn has_tool(&self, tool: ToolType) -> bool {
        self.tools.get(&tool).copied().unwrap_or(0) > 0
    }

    fn has_resources(&self, recipe: &[(ResourceType, u32)]) -> bool {
        recipe
            .iter()
            .all(|(res, count)| self.resources.get(res).copied().unwrap_or(0) >= *count)
    }

    fn consume_resources(&mut self, recipe: &[(ResourceType, u32)]) {
        for (res, count) in recipe {
            if let Some(v) = self.resources.get_mut(res) {
                *v = v.saturating_sub(*count);
            }
        }
    }
}

#[derive(Resource)]
struct CraftingRecipes {
    stone_axe: Vec<(ResourceType, u32)>,
}

impl Default for CraftingRecipes {
    fn default() -> Self {
        Self {
            stone_axe: vec![(ResourceType::Stick, 3), (ResourceType::Rock, 2)],
        }
    }
}

#[derive(Resource, Default)]
struct MouseLookState {
    yaw: f32,
    pitch: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 15_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(80.0, 80.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.16, 0.30, 0.16),
                ..default()
            }),
            ..default()
        },
        Collider::cuboid(40.0, 0.1, 40.0),
        RigidBody::Fixed,
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.4, 1.0)),
            material: materials.add(Color::srgb(0.3, 0.3, 0.8)),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
        Player,
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-6.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MainCamera,
        FollowCamera,
    ));

    let mut rng = rand::thread_rng();
    for _ in 0..10 {
        let pos = Vec3::new(rng.gen_range(-20.0..20.0), 0.3, rng.gen_range(-20.0..20.0));
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.3, 0.3, 0.3)),
                material: materials.add(Color::srgb(0.50, 0.37, 0.24)),
                transform: Transform::from_translation(pos),
                ..default()
            },
            Gatherable {
                resource: ResourceType::Stick,
                amount: 1,
                requires_tool: None,
            },
        ));
    }

    for _ in 0..10 {
        let pos = Vec3::new(rng.gen_range(-20.0..20.0), 0.4, rng.gen_range(-20.0..20.0));
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere::new(0.4).mesh().uv(16, 12)),
                material: materials.add(Color::srgb(0.45, 0.45, 0.45)),
                transform: Transform::from_translation(pos),
                ..default()
            },
            Gatherable {
                resource: ResourceType::Rock,
                amount: 1,
                requires_tool: None,
            },
        ));
    }

    for _ in 0..8 {
        let x = rng.gen_range(-18.0..18.0);
        let z = rng.gen_range(-18.0..18.0);
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Cylinder::new(0.35, 3.0)),
                material: materials.add(Color::srgb(0.38, 0.22, 0.12)),
                transform: Transform::from_xyz(x, 1.5, z),
                ..default()
            },
            Tree {
                health: 3,
                standing: true,
            },
            Gatherable {
                resource: ResourceType::Wood,
                amount: 3,
                requires_tool: Some(ToolType::StoneAxe),
            },
            Collider::cylinder(1.5, 0.35),
            RigidBody::Fixed,
        ));
    }
}

fn player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut q_player: Query<&mut Transform, With<Player>>,
    cam_q: Query<&Transform, (With<MainCamera>, Without<Player>)>,
) {
    let Ok(mut transform) = q_player.get_single_mut() else {
        return;
    };
    let Ok(cam_transform) = cam_q.get_single() else {
        return;
    };

    let mut direction = Vec3::ZERO;
    let forward = cam_transform.forward().with_y(0.0).normalize_or_zero();
    let right = cam_transform.right().with_y(0.0).normalize_or_zero();

    if keys.pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= right;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += right;
    }

    let speed = 6.0;
    transform.translation += direction.normalize_or_zero() * speed * time.delta_seconds();
    transform.translation.y = 1.0;
}

fn mouse_look(
    mut mouse_events: EventReader<MouseMotion>,
    mut look: ResMut<MouseLookState>,
    buttons: Res<ButtonInput<MouseButton>>,
    window_q: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window_q.get_single() else {
        return;
    };
    if !window.focused && !buttons.pressed(MouseButton::Left) {
        return;
    }
    let mut delta = Vec2::ZERO;
    for evt in mouse_events.read() {
        delta += evt.delta;
    }

    look.yaw -= delta.x * 0.005;
    look.pitch = (look.pitch - delta.y * 0.003).clamp(-0.8, 0.6);
}

fn update_camera(
    look: Res<MouseLookState>,
    mut cam_q: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    player_q: Query<&Transform, With<Player>>,
) {
    let Ok(mut cam) = cam_q.get_single_mut() else {
        return;
    };
    let Ok(player) = player_q.get_single() else {
        return;
    };

    let radius = 12.0;
    let height = 8.0;
    let offset = Vec3::new(
        look.yaw.cos() * radius,
        height + look.pitch * 5.0,
        look.yaw.sin() * radius,
    );
    cam.translation = player.translation + offset;
    cam.look_at(player.translation, Vec3::Y);
}

fn player_attack(
    buttons: Res<ButtonInput<MouseButton>>,
    player_q: Query<&Transform, With<Player>>,
    mut inventory: ResMut<PlayerInventory>,
    mut commands: Commands,
    mut q_gatherables: Query<(Entity, &Transform, &Gatherable, Option<&mut Tree>)>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let Ok(player) = player_q.get_single() else {
        return;
    };

    for (entity, transform, gatherable, tree_opt) in q_gatherables.iter_mut() {
        let dist = player.translation.distance(transform.translation);
        if dist > 2.2 {
            continue;
        }

        if let Some(tool) = gatherable.requires_tool {
            if !inventory.has_tool(tool) {
                info!("Need a Stone Axe to harvest wood.");
                continue;
            }
        }

        if let Some(mut tree) = tree_opt {
            if tree.standing {
                tree.health -= 1;
                info!("Chopping tree... health {}", tree.health);
                if tree.health <= 0 {
                    tree.standing = false;
                    commands.entity(entity).insert((
                        RigidBody::Dynamic,
                        VelocityY(5.0),
                        ExternalImpulse {
                            impulse: Vec3::new(3.0, 3.0, 0.0),
                            torque_impulse: Vec3::new(0.0, 0.0, 2.0),
                        },
                    ));
                    inventory.add_resource(gatherable.resource, gatherable.amount);
                    info!("Tree felled! +{} Wood", gatherable.amount);
                }
                break;
            }
        } else {
            inventory.add_resource(gatherable.resource, gatherable.amount);
            commands.entity(entity).despawn_recursive();
            info!("Collected {:?}", gatherable.resource);
            break;
        }
    }
}

fn handle_tree_fall(
    time: Res<Time>,
    mut commands: Commands,
    mut q_trees: Query<(Entity, &mut Transform, &mut VelocityY, &Tree), Without<Player>>,
) {
    for (entity, mut transform, mut vy, tree) in q_trees.iter_mut() {
        if tree.standing {
            continue;
        }

        vy.0 -= 9.8 * time.delta_seconds();
        transform.translation.y += vy.0 * time.delta_seconds();
        if transform.translation.y <= 0.2 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn crafting_input(
    keys: Res<ButtonInput<KeyCode>>,
    recipes: Res<CraftingRecipes>,
    mut inventory: ResMut<PlayerInventory>,
) {
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }

    if inventory.has_tool(ToolType::StoneAxe) {
        info!("Stone Axe already crafted.");
        return;
    }

    if inventory.has_resources(&recipes.stone_axe) {
        inventory.consume_resources(&recipes.stone_axe);
        inventory.add_tool(ToolType::StoneAxe);
        info!("Crafted Stone Axe! You can now chop trees for wood.");
    } else {
        info!("Not enough resources for Stone Axe. Need 3 sticks and 2 rocks.");
    }
}

fn inventory_debug(keys: Res<ButtonInput<KeyCode>>, inventory: Res<PlayerInventory>) {
    if keys.just_pressed(KeyCode::Tab) {
        info!(
            "Inventory => sticks: {}, rocks: {}, wood: {}, stone_axe: {}",
            inventory
                .resources
                .get(&ResourceType::Stick)
                .copied()
                .unwrap_or(0),
            inventory
                .resources
                .get(&ResourceType::Rock)
                .copied()
                .unwrap_or(0),
            inventory
                .resources
                .get(&ResourceType::Wood)
                .copied()
                .unwrap_or(0),
            inventory
                .tools
                .get(&ToolType::StoneAxe)
                .copied()
                .unwrap_or(0)
        );
    }
}
