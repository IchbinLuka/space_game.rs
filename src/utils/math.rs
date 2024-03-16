use bevy::math::Vec3;

/// Returns true if the ray intersects the sphere.
/// `direction` must be a unit vector
pub fn sphere_intersection(
    center: Vec3,
    radius: f32,
    origin: Vec3,
    direction: Vec3,
) -> Option<f32> {
    let oc = origin - center;
    let a = direction.length_squared();
    let b = oc.dot(direction);
    let c = oc.length_squared() - radius * radius;
    let discriminant = b * b - a * c;

    if discriminant < 0. {
        return None;
    }

    let t = (-b - discriminant.sqrt()) / a;
    if t > 0. {
        Some(t)
    } else {
        None
    }
}
