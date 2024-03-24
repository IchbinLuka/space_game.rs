use std::collections::VecDeque;

use bevy::{prelude::*, render::render_resource::PrimitiveTopology};

use bevy_rapier3d::{dynamics::Velocity, geometry::CollidingEntities};

use crate::components::gravity::GravityAffected;
use crate::materials::toon::{ApplyToonMaterial, ToonMaterial};
use crate::states::ON_GAME_STARTED;
use crate::ui::fonts::FontsResource;
use crate::ui::theme::default_font;
use crate::{
    components::{
        gravity::{gravity_step, GravitySource},
        movement::MaxSpeed,
    },
    entities::{
        bullet::{Bullet, BulletSpawnEvent, BulletTarget, BulletType},
        planet::Planet,
    },
    states::game_running,
    utils::{misc::CollidingEntitiesExtension, sets::Set},
};

use super::bot::EnemyTarget;
use super::{
    Health, IsPlayer, LastBulletInfo, ParticleSpawnEvent, Spaceship, SpaceshipAssets,
    SpaceshipBundle,
};

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct LastHit(Option<f32>);

fn player_shoot(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    query: Query<(&Transform, &Velocity, Entity, &Spaceship), With<Player>>,
    mut bullet_spawn_events: EventWriter<BulletSpawnEvent>,
    mut last_bullet_info: Local<LastBulletInfo>,
) {
    if !last_bullet_info.timer.finished() {
        last_bullet_info.timer.tick(time.delta());
        return;
    }

    for (transform, velocity, entity, spaceship) in &query {
        if keyboard_input.pressed(KeyCode::Space) {
            // If finished, the timer should wait for the player to shoot before ticking again
            last_bullet_info.timer.tick(time.delta());

            spaceship.shoot(
                &mut last_bullet_info,
                &mut bullet_spawn_events,
                entity,
                transform,
                *velocity,
                BulletType::Player,
            );
        }
    }
}

fn player_setup(mut commands: Commands, assets: Res<SpaceshipAssets>) {
    commands.spawn((
        Player,
        SpaceshipBundle::new(assets.player_ship.clone(), Vec3::ZERO),
        Health::new(100.0),
        MaxSpeed { max_speed: 60.0 },
        LastHit::default(),
        EnemyTarget,
        GravityAffected,
        BulletTarget {
            target_type: BulletType::Bot,
            bullet_damage: Some(10.0),
        },
        ApplyToonMaterial {
            base_material: ToonMaterial {
                filter_scale: 0.0,
                ..default()
            },
        },
    ));
}

fn player_input(
    timer: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform, Entity, &mut Spaceship), IsPlayer>,
    mut particle_spawn: EventWriter<ParticleSpawnEvent>,
) {
    for (mut velocity, mut transform, entity, mut spaceship) in &mut query {
        if keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) {
            velocity.linvel += transform.forward().normalize() * timer.delta_seconds() * 60.0;
            particle_spawn.send(ParticleSpawnEvent {
                entity,
                direction: None,
            });
        }

        if keyboard_input.any_pressed([KeyCode::Left, KeyCode::A, KeyCode::Right, KeyCode::D]) {
            let sign = if keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
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
        emissive: Color::WHITE,
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
        let mesh = Mesh::new(PrimitiveTopology::TriangleStrip);

        let mesh_handle = meshes.add(mesh);
        commands.spawn((
            PbrBundle {
                mesh: mesh_handle,
                material: material.clone(),
                ..default()
            },
            trail,
        ));
    }
}

fn player_trail_update(
    mut trails: Query<(&mut Handle<Mesh>, &mut PlayerTrail, &mut Transform), Without<Player>>,
    player_query: Query<(&Transform, &Spaceship, &GlobalTransform, Entity), IsPlayer>,
    player_changed: Query<(), Changed<Spaceship>>,
    mut assets: ResMut<Assets<Mesh>>,
) {
    const HISTORY_LENGTH: usize = 50;

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
        emissive: Color::WHITE,
        double_sided: true,
        ..default()
    });

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip);
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

fn player_regeneration(mut players: Query<(&mut Health, &LastHit), IsPlayer>, time: Res<Time>) {
    const HEAL_COOLDOWN: f32 = 4.0;

    for (mut health, last_hit) in &mut players {
        if let Some(last_hit) = last_hit.0 {
            if time.elapsed_seconds() - last_hit < HEAL_COOLDOWN {
                continue;
            }
        }

        health.heal(2.0 * time.delta_seconds());
    }
}

fn player_collision(
    mut players: Query<(&CollidingEntities, &mut LastHit), IsPlayer>,
    bullet_query: Query<(), (With<Bullet>, Without<Player>)>,
    time: Res<Time>,
) {
    for (colliding, mut last_hit) in &mut players {
        if colliding.fulfills_query(&bullet_query) {
            last_hit.0 = Some(time.elapsed_seconds());
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
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        display: Display::Flex,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    background_color: Color::rgba(0., 0., 0., 0.3).into(),
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
                                ..default()
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
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            ON_GAME_STARTED,
            (player_setup, player_line_setup, player_trail_setup),
        )
        .add_systems(
            Update,
            (
                player_shoot.in_set(Set::BulletEvents),
                player_input,
                player_line_update,
                player_regeneration,
                player_collision,
                player_trail_update,
                return_to_mission_warning_spawn,
                return_to_mission_warning_update,
                return_to_mission_warning_despawn,
            )
                .run_if(game_running()),
        );
    }
}
