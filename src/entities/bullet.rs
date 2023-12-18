use std::f32::consts::PI;

use bevy::prelude::*;

use crate::Movement;

use super::player::Player;

#[derive(Component)]
pub struct Bullet;

fn bullet_setup(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let bullet_mesh = meshes.add(shape::Circle { radius: 0.1, vertices: 20 }.into());
    let bullet_material = materials.add(StandardMaterial {
        base_color: Color::WHITE, 
        emissive: Color::WHITE,
        unlit: true,
        diffuse_transmission: 1.0, 
        ..default()
    });

    commands.insert_resource(BulletResource {
        bullet_mesh,
        bullet_material,
    });
}

fn bullet_shoot(
    keyboard_input: Res<Input<KeyCode>>, 
    mut query: Query<(&Transform, &Movement, With<Player>)>, 
    mut commands: Commands,
    bullet_res: Res<BulletResource>,
) {
    for (transform, movement, _) in &mut query {
        if keyboard_input.just_pressed(KeyCode::Space) {
            let mut bullet_transform = Transform::from_xyz(transform.translation.x, transform.translation.y, transform.translation.z);
            bullet_transform.rotate_x(-PI * 0.5);
            commands.spawn((
                PbrBundle {
                    mesh: bullet_res.bullet_mesh.clone(),
                    material: bullet_res.bullet_material.clone(),
                    transform: bullet_transform,
                    ..default()
                }, 
                Movement {
                    vel: transform.forward().normalize() * 20.0 + movement.vel,
                    ..default()
                }, 
                Bullet,
            ));
        }
    }
}
struct BulletResource {
    bullet_mesh: Handle<Mesh>,
    bullet_material: Handle<StandardMaterial>,
}
impl Resource for BulletResource {}

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, bullet_setup)
            .add_systems(Update, bullet_shoot);
    }
}