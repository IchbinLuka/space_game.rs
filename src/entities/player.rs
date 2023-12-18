use bevy::prelude::*;

use crate::Movement;

#[derive(Component)]
pub struct Player;

fn player_input(
    timer: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Movement, &mut Transform, With<Player>)>,
) {
    for (mut movement, mut transform, _) in &mut query {
        for key in keyboard_input.get_pressed() {
            match key {
                KeyCode::Up | KeyCode::W => movement.vel += transform.forward().normalize(),
                KeyCode::Down | KeyCode::S => movement.vel -= transform.forward().normalize(),
                KeyCode::Left | KeyCode::A => transform.rotate_y(3.0 * timer.delta_seconds()),
                KeyCode::Right | KeyCode::D => transform.rotate_y(-3.0 * timer.delta_seconds()),
                _ => (),
            }
        }
        if movement.vel.length() > 10.0 {
            movement.vel = movement.vel.normalize() * 10.0;
        }
    }
}

fn player_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb_u8(124, 144, 255).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Player, 
        Movement {
            max_speed: Some(10.0),
            friction: 0.3,
            ..default()
        },
    ));
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, player_setup)
            .add_systems(Update, player_input);
    }
}