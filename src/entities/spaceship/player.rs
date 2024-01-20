use bevy::{prelude::*, render::render_resource::PrimitiveTopology};

use bevy_rapier3d::{dynamics::Velocity, geometry::CollidingEntities};

use crate::{
    components::{
        gravity::{gravity_step, GravitySource},
        movement::MaxSpeed,
    },
    entities::{bullet::{BulletSpawnEvent, Bullet}, planet::Planet},
    utils::{sets::Set, misc::CollidingEntitiesExtension},
    AppState,
};

use super::{
    Health, IsPlayer, LastBulletInfo, ParticleSpawnEvent, SpaceshipAssets, SpaceshipBundle,
};

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct LastHit(Option<f32>);

fn player_shoot(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    query: Query<(&Transform, &Velocity, Entity), With<Player>>,
    mut bullet_spawn_events: EventWriter<BulletSpawnEvent>,
    mut last_bullet_info: Local<LastBulletInfo>,
) {
    if !last_bullet_info.timer.finished() {
        last_bullet_info.timer.tick(time.delta());
        return;
    }

    for (transform, velocity, entity) in &query {
        if keyboard_input.pressed(KeyCode::Space) {
            // If finished, the timer should wait for the player to shoot before ticking again
            last_bullet_info.timer.tick(time.delta());
            let side = last_bullet_info.side;

            let pos = transform.translation + transform.rotation.mul_vec3(side.into());
            let mut bullet_transform = Transform::from_translation(pos);

            bullet_transform.rotate(transform.rotation);
            debug!("Spawning bullet");
            bullet_spawn_events.send(BulletSpawnEvent {
                position: bullet_transform,
                entity_velocity: *velocity,
                direction: transform.forward(),
                entity,
            });

            last_bullet_info.side = side.other();
        }
    }
}

fn player_setup(mut commands: Commands, assets: Res<SpaceshipAssets>) {
    commands.spawn((
        Player,
        SpaceshipBundle::new(assets.player_ship.clone(), Vec3::ZERO),
        Health(100.0),
        MaxSpeed { max_speed: 60.0 },
        LastHit(None), 
    ));
}

fn player_input(
    timer: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Transform, Entity), IsPlayer>,
    mut particle_spawn: EventWriter<ParticleSpawnEvent>,
) {
    for (mut velocity, mut transform, entity) in &mut query {
        for key in keyboard_input.get_pressed() {
            match key {
                KeyCode::Up | KeyCode::W => {
                    velocity.linvel += transform.forward().normalize() * timer.delta_seconds() * 60.0;
                    particle_spawn.send(ParticleSpawnEvent { entity });
                }
                KeyCode::Left | KeyCode::A => transform.rotate_y(5.0 * timer.delta_seconds()),
                KeyCode::Right | KeyCode::D => transform.rotate_y(-5.0 * timer.delta_seconds()),
                _ => (),
            }
        }
    }
}

const PREDICTION_LENGTH: usize = 100;
const LINE_THICKNESS: f32 = 0.1;

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
    player_query: Query<(&Transform, &Velocity), IsPlayer>,
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
        for (player_transform, player_velocity) in &player_query {
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

fn player_regeneration(
    mut players: Query<(&mut Health, &LastHit), IsPlayer>,
    time: Res<Time>,
) {
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

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::MainScene),
            (player_setup, player_line_setup),
        )
        .add_systems(
            Update,
            (
                player_shoot.in_set(Set::BulletEvents),
                player_input,
                player_line_update,
                player_regeneration,
                player_collision,
            ).run_if(in_state(AppState::MainScene)),
        );
    }
}
