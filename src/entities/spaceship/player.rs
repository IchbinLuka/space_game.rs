use std::collections::VecDeque;

use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};

use bevy_mod_outline::OutlineBundle;
use bevy_rapier3d::{dynamics::Velocity, geometry::Collider};

use crate::{
    components::{
        gravity::{gravity_step, GravityAffected, GravitySource},
        health::{HasShield, Regeneration},
        movement::MaxSpeed,
    },
    entities::{
        bullet::{BulletSpawnEvent, BulletTarget, BulletType},
        explosion::ExplosionEvent,
        planet::Planet,
        powerup::PowerUpAssets,
        turret::Turret,
    },
    materials::{
        blink::BlinkMaterial,
        toon::{replace_with_toon_materials, ToonMaterial},
    },
    states::{game_running, AppState, DespawnOnCleanup, ON_GAME_STARTED},
    ui::{
        fonts::FontsResource,
        minimap::{MinimapAssets, ShowOnMinimap},
        theme::default_font,
    },
    utils::{materials::default_outline, misc::AsCommand, scene::ReplaceMaterialPlugin, sets::Set},
};

use super::bot::EnemyTarget;
use super::{
    Health, IsPlayer, LastBulletInfo, ParticleSpawnEvent, Spaceship, SpaceshipAssets,
    SpaceshipBundle,
};

#[derive(Component)]
pub struct Player;

#[derive(Resource, Default)]
pub struct PlayerInventory {
    pub bombs: u32,
    pub turrets: u32,
}

#[derive(Component)]
pub struct Bomb {
    pub timer: Timer,
}

#[derive(Component)]
pub struct PlayerTurret;

#[derive(Component, Default, Deref, DerefMut)]
pub struct LastHit(pub(crate) Option<f32>);

fn player_shoot(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    query: Query<(&Transform, &Velocity, &Spaceship), With<Player>>,
    mut bullet_spawn_events: EventWriter<BulletSpawnEvent>,
    mut last_bullet_info: Local<LastBulletInfo>,
) {
    if !last_bullet_info.timer.finished() {
        last_bullet_info.timer.tick(time.delta());
        return;
    }

    for (transform, velocity, spaceship) in &query {
        if keyboard_input.pressed(KeyCode::Space) {
            // If finished, the timer should wait for the player to shoot before ticking again
            last_bullet_info.timer.tick(time.delta());

            spaceship.shoot(
                &mut last_bullet_info,
                &mut bullet_spawn_events,
                transform,
                *velocity,
                BulletType::Player,
            );
        }
    }
}

const HEAL_COOLDOWN: f32 = 4.0;
const HEAL_SPEED: f32 = 2.0;

fn spawn_player(
    mut commands: Commands,
    assets: Res<SpaceshipAssets>,
    minimap_assets: Res<MinimapAssets>,
) {
    commands.spawn((
        Player,
        SpaceshipBundle::new(assets.player_ship.clone(), Vec3::ZERO),
        Health::new(100.0),
        MaxSpeed { max_speed: 30.0 },
        LastHit::default(),
        EnemyTarget,
        GravityAffected,
        Regeneration {
            heal_cooldown: HEAL_COOLDOWN,
            regen_speed: HEAL_SPEED,
        },
        ShowOnMinimap {
            sprite: minimap_assets.player_indicator.clone(),
            size: 0.1.into(),
        },
        BulletTarget {
            target_type: BulletType::Bot,
            bullet_damage: Some(10.0),
        },
        DespawnOnCleanup,
    ));

    commands.insert_resource(PlayerInventory::default());
}

pub struct ExhaustCooldown(pub Timer);
impl Default for ExhaustCooldown {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Repeating))
    }
}
impl ExhaustCooldown {
    fn tick(&mut self, time: &Time) {
        if !self.0.finished() {
            self.0.tick(time.delta());
        }
    }
    
    fn can_spawn_particle(&self) -> bool {
        self.0.finished()
    }
}

fn player_input(
    timer: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform, Entity, &mut Spaceship), IsPlayer>,
    mut particle_spawn: EventWriter<ParticleSpawnEvent>,
    mut inventory: ResMut<PlayerInventory>,
    mut commands: Commands,
    powerup_assets: Res<PowerUpAssets>,
    mut exhaust_cooldown: Local<ExhaustCooldown>, 
) {
    exhaust_cooldown.tick(&timer);
    for (mut velocity, mut transform, entity, mut spaceship) in &mut query {
        if keyboard_input.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
            velocity.linvel += transform.forward().normalize() * timer.delta_seconds() * 60.0;
            if exhaust_cooldown.can_spawn_particle() {
                particle_spawn.send(ParticleSpawnEvent {
                    entity,
                    direction: None,
                });
            }
        }

        if keyboard_input.any_pressed([
            KeyCode::ArrowLeft,
            KeyCode::KeyA,
            KeyCode::ArrowRight,
            KeyCode::KeyD,
        ]) {
            let sign = if keyboard_input.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
                1.0
            } else {
                -1.0
            };

            const AUXILIARY_TURN_SPEED: f32 = 3.0;
            const TURN_SPEED: f32 = 5.0;

            let speed = if spaceship.auxiliary_drive {
                AUXILIARY_TURN_SPEED
            } else {
                TURN_SPEED
            };

            transform.rotate_y(sign * speed * timer.delta_seconds());
        }

        if keyboard_input.just_pressed(KeyCode::ShiftLeft) {
            spaceship.auxiliary_drive = !spaceship.auxiliary_drive;
        }

        if keyboard_input.just_pressed(KeyCode::KeyG) && inventory.bombs > 0 {
            inventory.bombs -= 1;

            commands.spawn((
                Bomb {
                    timer: Timer::from_seconds(3.0, TimerMode::Once),
                },
                DespawnOnCleanup,
                SceneBundle {
                    transform: Transform::from_translation(transform.translation),
                    scene: powerup_assets.bomb.clone(),
                    ..default()
                },
                OutlineBundle {
                    outline: default_outline(),
                    ..default()
                },
            ));
        }

        if keyboard_input.just_pressed(KeyCode::KeyT) && inventory.turrets > 0 {
            inventory.turrets -= 1;

            commands.spawn((
                PlayerTurret,
                Turret {
                    bullet_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
                    bullet_type: BulletType::Player,
                    base_orientation: Vec3::Z,
                    rotation_bounds: (f32::NEG_INFINITY, f32::INFINITY),
                },
                DespawnOnCleanup,
                SceneBundle {
                    transform: Transform::from_translation(transform.translation),
                    scene: powerup_assets.turret.clone(),
                    ..default()
                },
                OutlineBundle {
                    outline: default_outline(),
                    ..default()
                },
            ));
        }
    }
}

fn bomb_update(
    mut grenades: Query<(&mut Bomb, &Transform, Entity)>,
    mut bots: Query<(
        &mut Health,
        &GlobalTransform,
        &BulletTarget,
        Option<&Collider>,
        Option<&HasShield>,
    )>,
    time: Res<Time>,
    mut explosion_events: EventWriter<ExplosionEvent>,
    mut commands: Commands,
) {
    for (mut grenade, transform, entity) in &mut grenades {
        grenade.timer.tick(time.delta());
        if !grenade.timer.finished() {
            continue;
        }
        commands.entity(entity).despawn_recursive();
        explosion_events.send(ExplosionEvent {
            position: transform.translation,
            radius: 20.0,
            parent: None,
        });

        for (mut health, bot_transform, bullet_target, collider, has_shield) in &mut bots {
            if bullet_target.target_type != BulletType::Player {
                continue;
            }
            let bot_transform = bot_transform.compute_transform();
            let distance = if let Some(collider) = collider {
                collider.distance_to_point(
                    bot_transform.translation,
                    bot_transform.rotation,
                    transform.translation,
                    true,
                )
            } else {
                let bot_pos = bot_transform.translation;
                (bot_pos - transform.translation).length()
            };

            if distance < 20.0 && has_shield.is_none() {
                health.take_damage(1.0 / f32::min(distance * 2.0, 1.0).powi(2) * 40.0);
            }
        }
    }
}

const PREDICTION_LENGTH: usize = 100;
const LINE_THICKNESS: f32 = 0.1;

#[derive(Component)]
struct PlayerTrail {
    offset: Vec3,
    pos_history: VecDeque<Vec3>,
}

fn player_trail_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: Color::WHITE.into(),
        double_sided: true,
        ..default()
    });

    const TRAILS: [PlayerTrail; 2] = [
        PlayerTrail {
            offset: Vec3 {
                x: -1.,
                y: -0.4,
                z: 0.,
            },
            pos_history: VecDeque::new(),
        },
        PlayerTrail {
            offset: Vec3 {
                x: 1.,
                y: -0.4,
                z: 0.,
            },
            pos_history: VecDeque::new(),
        },
    ];

    for trail in TRAILS {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip, RenderAssetUsages::all());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<Vec3>::new());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<Vec3>::new());

        let mesh_handle = meshes.add(mesh);
        commands.spawn((
            PbrBundle {
                mesh: mesh_handle,
                material: material.clone(),
                ..default()
            },
            trail,
            NotShadowReceiver,
            NotShadowCaster,
            DespawnOnCleanup,
        ));
    }
}

#[derive(Deref, DerefMut)]
struct TrailUpdateTimer(Timer);

impl Default for TrailUpdateTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.01666, TimerMode::Repeating))
    }
}

fn player_trail_update(
    mut trails: Query<(&mut Handle<Mesh>, &mut PlayerTrail, &mut Transform), Without<Player>>,
    player_query: Query<(&Transform, &Spaceship, &GlobalTransform, Entity), IsPlayer>,
    player_changed: Query<(), Changed<Spaceship>>,
    mut assets: ResMut<Assets<Mesh>>,
    mut timer: Local<TrailUpdateTimer>,
    time: Res<Time>,
) {
    const HISTORY_LENGTH: usize = 100;
    timer.tick(time.delta());
    if !timer.just_finished() {
        return;
    }

    let Ok((player_transform, spaceship, player_global, player_entity)) = player_query.get_single()
    else {
        return;
    };

    let player_changed = player_changed.get(player_entity).is_ok();

    if !player_changed && !spaceship.auxiliary_drive {
        return;
    }

    for (mesh, mut trail, mut transform) in &mut trails {
        if player_changed && !spaceship.auxiliary_drive {
            trail.pos_history.clear();
        }

        let current_pos = player_transform.translation - Vec3::Y * 0.1;
        transform.translation = current_pos;

        let Some(mesh) = assets.get_mut(mesh.id()) else {
            continue;
        };

        let offset = trail.offset;
        trail
            .pos_history
            .push_back(player_global.transform_point(offset));

        if trail.pos_history.len() > HISTORY_LENGTH {
            trail.pos_history.pop_front();
        }

        let mut positions = Vec::<Vec3>::with_capacity(HISTORY_LENGTH * 2);

        for (i, pos) in trail.pos_history.iter().enumerate().skip(1) {
            let previous = trail.pos_history[i - 1];

            let perpendicular = (*pos - previous).cross(Vec3::Y).normalize();
            let thickness = (i as f32 / HISTORY_LENGTH as f32).powf(2.0) * LINE_THICKNESS;

            positions.push(*pos - current_pos - perpendicular * thickness);
            positions.push(*pos - current_pos + perpendicular * thickness);
        }

        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, [Vec3::Y].repeat(positions.len()));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    }
}

#[derive(Component)]
struct PlayerLine;

fn player_line_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: Color::WHITE.into(),
        double_sided: true,
        ..default()
    });

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip, RenderAssetUsages::all());
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        [Vec3::ZERO; PREDICTION_LENGTH * 2].to_vec(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        [Vec3::Y; PREDICTION_LENGTH * 2].to_vec(),
    );
    let mesh_handle = meshes.add(mesh);

    commands.spawn((
        PbrBundle {
            mesh: mesh_handle,
            material,
            ..default()
        },
        PlayerLine,
        DespawnOnCleanup,
        NotShadowReceiver,
        NotShadowCaster,
    ));
}

fn player_line_update(
    mut line_query: Query<(&mut Handle<Mesh>, &mut Transform), (With<PlayerLine>, Without<Player>)>,
    player_query: Query<(&Transform, &Velocity, &Spaceship, Entity), IsPlayer>,
    player_changed: Query<(), (Changed<Spaceship>, IsPlayer)>,
    gravity_sources: Query<
        (&Transform, &GravitySource, Option<&Planet>),
        (Without<Player>, Without<PlayerLine>),
    >,
    mut assets: ResMut<Assets<Mesh>>,
) {
    for (mesh_handle, mut transform) in &mut line_query {
        let Some(mesh) = assets.get_mut(mesh_handle.id()) else {
            continue;
        };
        for (player_transform, player_velocity, spaceship, entity) in &player_query {
            if spaceship.auxiliary_drive {
                if player_changed.get(entity).is_ok() {
                    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<Vec3>::new());
                    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<Vec3>::new());
                }
                continue;
            }

            transform.translation = player_transform.translation - Vec3::Y * 0.1;

            let mut positions: Vec<Vec3> = Vec::with_capacity(PREDICTION_LENGTH * 2);
            let player_pos = player_transform.translation;
            let mut current_pos = Vec3::ZERO;
            let mut current_vel = player_velocity.linvel;

            for i in 0..PREDICTION_LENGTH {
                let perpendicular = current_vel.cross(Vec3::Y).normalize();
                let thickness =
                    (1.0 - (i as f32 / PREDICTION_LENGTH as f32).powf(2.0)) * LINE_THICKNESS;

                current_pos += current_vel * 0.02;
                if !gravity_sources.iter().all(|(transform, _, planet)| {
                    let Some(p) = planet else {
                        return true;
                    };
                    (current_pos + player_pos).distance(transform.translation) > p.radius
                }) {
                    break;
                }

                current_vel += gravity_sources
                    .iter()
                    .map(|(gravity_transform, gravity_source, _)| {
                        gravity_step(
                            gravity_transform,
                            gravity_source,
                            0.02,
                            current_pos + player_pos,
                            current_vel,
                        )
                    })
                    .sum::<Vec3>();

                positions.push(current_pos - perpendicular * thickness);
                positions.push(current_pos + perpendicular * thickness);
            }

            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, [Vec3::Y].repeat(positions.len()));
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        }
    }
}

#[derive(Component)]
struct ReturnToMissionWarning {
    timer: Timer,
}

#[derive(Component)]
struct ReturnToMissionWarningText;

const RETURN_TO_MISSION_TIME: u32 = 10;
const MAX_DISTANCE: f32 = 200.0;

fn return_to_mission_warning_despawn(
    players: Query<&Transform, IsPlayer>,
    mut warnings: Query<Entity, With<ReturnToMissionWarning>>,
    mut commands: Commands,
) {
    for transform in &players {
        if transform.translation.length() < MAX_DISTANCE {
            for entity in &mut warnings {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

fn return_to_mission_warning_spawn(
    players: Query<&Transform, IsPlayer>,
    warning: Query<Entity, With<ReturnToMissionWarning>>,
    mut commands: Commands,
    font_res: Res<FontsResource>,
) {
    if warning.iter().next().is_some() {
        return;
    }

    for transform in &players {
        if transform.translation.length() < MAX_DISTANCE {
            continue;
        }
        info!("Player is out of mission area");
        commands
            .spawn((
                DespawnOnCleanup,
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        display: Display::Flex,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: Color::srgba(0., 0., 0., 0.3).into(),
                    ..default()
                },
                ReturnToMissionWarning {
                    timer: Timer::from_seconds(RETURN_TO_MISSION_TIME as f32, TimerMode::Once),
                },
            ))
            .with_children(|c| {
                c.spawn((
                    TextBundle {
                        text: Text::from_section(
                            t!("return_to_mission_area", time = RETURN_TO_MISSION_TIME),
                            TextStyle {
                                font_size: 80.0,
                                color: Color::WHITE,
                                font: default_font(&font_res),
                            },
                        ),
                        ..default()
                    },
                    ReturnToMissionWarningText,
                ));
            });
    }
}

fn return_to_mission_warning_update(
    mut warnings: Query<(Entity, &mut ReturnToMissionWarning), Without<ReturnToMissionWarningText>>,
    mut warning_text: Query<&mut Text, With<ReturnToMissionWarningText>>,
    mut players: Query<&mut Health, IsPlayer>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut warning) in &mut warnings {
        warning.timer.tick(time.delta());

        for mut text in &mut warning_text {
            text.sections[0].value = t!(
                "return_to_mission_area",
                time = warning.timer.remaining_secs().ceil() as u32
            )
            .to_string();
        }

        if warning.timer.finished() {
            commands.entity(entity).despawn_recursive();
            for mut health in &mut players {
                health.kill();
            }
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct PlayerRespawnTimer(pub Timer);

fn player_death(
    players: Query<(Entity, &Health, &Transform), (IsPlayer, Changed<Health>)>,
    mut trails: Query<&mut PlayerTrail>,
    mut commands: Commands,
    mut explosion_events: EventWriter<ExplosionEvent>,
) {
    for (entity, health, transform) in &players {
        if health.is_dead() {
            explosion_events.send(ExplosionEvent {
                position: transform.translation,
                parent: None,
                radius: 10.0,
            });
            commands.entity(entity).despawn_recursive();
            for mut trail in &mut trails {
                trail.pos_history.clear();
            }
            commands.insert_resource(PlayerRespawnTimer(Timer::from_seconds(
                2.0,
                TimerMode::Once,
            )))
        }
    }
}

fn player_respawn(
    mut player_respawn_timer: ResMut<PlayerRespawnTimer>,
    mut commands: Commands,
    time: Res<Time>,
) {
    player_respawn_timer.tick(time.delta());
    if !player_respawn_timer.just_finished() {
        return;
    }

    commands.remove_resource::<PlayerRespawnTimer>();
    commands.add(spawn_player.to_command(()));
}

fn respawn_timer_cleanup(mut commands: Commands) {
    info!("Cleaning up respawn timer");
    commands.remove_resource::<PlayerRespawnTimer>();
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerInventory::default())
            .add_systems(
                ON_GAME_STARTED,
                (spawn_player, player_line_setup, player_trail_setup),
            )
            .add_systems(OnExit(AppState::MainScene), respawn_timer_cleanup)
            .add_plugins((
                ReplaceMaterialPlugin::<Bomb, _>::new(Box::new(|name, current| {
                    if name.as_str() == "light" {
                        return None;
                    }
                    Some(ToonMaterial {
                        color: current.base_color,
                        disable_outline: true,
                        ..default()
                    })
                })),
                ReplaceMaterialPlugin::<Bomb, _>::new(Box::new(|name, _| {
                    if name.as_str() != "light" {
                        return None;
                    }
                    Some(BlinkMaterial {
                        period: 1.0,
                        color_1: Color::srgb(1.0, 0.0, 0.0),
                        color_2: Color::srgb(0.5, 0.0, 0.0),
                    })
                })),
                ReplaceMaterialPlugin::<PlayerTurret, _>::new(replace_with_toon_materials(
                    ToonMaterial {
                        disable_outline: true,
                        ..default()
                    },
                )),
            ))
            .add_systems(
                Update,
                (
                    player_shoot.in_set(Set::BulletEvents),
                    player_input,
                    return_to_mission_warning_spawn,
                    return_to_mission_warning_update,
                    return_to_mission_warning_despawn,
                    player_death,
                    bomb_update,
                    player_respawn.run_if(resource_exists::<PlayerRespawnTimer>),
                )
                    .run_if(game_running()),
            )
            .add_systems(
                Update,
                (player_line_update, player_trail_update)
                    .run_if(game_running().or_else(in_state(AppState::GameOver))),
            );
    }
}
