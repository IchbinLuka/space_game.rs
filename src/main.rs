use bevy::{prelude::*, input::keyboard};


#[derive(Component)]
struct Person;

#[derive(Component)]
struct Name(String);

#[derive(Resource)]
struct GreetTimer(Timer);


fn add_people(mut commands: Commands) {
    commands.spawn((Person, Name("Elaina Proctor".to_string())));
    commands.spawn((Person, Name("Renzo Hume".to_string())));
    commands.spawn((Person, Name("Zayna Nieves".to_string())));
}

fn greet_people(
    time: Res<Time>, 
    mut timer: ResMut<GreetTimer>, 
    query: Query<&Name, With<Person>>
) {
    if timer.0.tick(time.delta()).just_finished() {
        for name in &query {
            print!("hello {}!\n", name.0);
        }
    }
}


pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .add_systems(Startup, add_people)
            .add_systems(Update, greet_people);
    }
}

#[derive(Component)]
struct Cube {
    vel: Vec3,
}

fn cube_input_system(
    keyboard_input: Res<Input<keyboard::KeyCode>>,
    mut query: Query<&mut Cube>,
) {
    for mut cube in &mut query {
        if keyboard_input.just_pressed(keyboard::KeyCode::Left) {
            cube.vel.x -= 1.0;
        }
        if keyboard_input.just_pressed(keyboard::KeyCode::Right) {
            cube.vel.x += 1.0;
        }
    }
}

fn cube_movement_system(time: Res<Time>, mut cubes: Query<(&mut Transform, &mut Cube)>) {
    for (mut transform, mut cube) in &mut cubes {
        transform.translation += cube.vel * time.delta_seconds();
        cube.vel *= 0.9;
    }
}



pub struct ScenePlugin3D;

impl Plugin for ScenePlugin3D {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, scene_setup_3d)
            .add_systems(Update, (cube_input_system, cube_movement_system));
    }
}

fn scene_setup_3d(
    mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Circle::new(4.0).into()),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb_u8(124, 144, 255).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        }, 
        Cube { vel: Vec3::ZERO }
    ));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}


fn main() {
    App::new()
        .add_plugins((DefaultPlugins, HelloPlugin, ScenePlugin3D))
        .run();
}
