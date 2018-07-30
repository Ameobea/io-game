use std::f32::INFINITY;

use nalgebra::{distance, Isometry2, Point2, Vector2};

fn cross(v: Vector2<f32>, w: Vector2<f32>) -> f32 {
    (v.x * w.y) - (v.y * w.x)
}

pub fn ray_collision(
    ray_origin: Point2<f32>,
    ray_dir: Vector2<f32>,
    vertices: &[Point2<f32>],
    isometry: &Isometry2<f32>,
) -> Option<(Point2<f32>, f32)> {
    let mut last_transformed_pt = isometry * vertices[0];
    let (nearest_collision, smallest_distance) = vertices
        .into_iter()
        .skip(1)
        .map(|point| {
            // Translate + rotate segment in accordance with the entity's isometry
            let seg_start = last_transformed_pt;
            let seg_end = isometry * point;
            last_transformed_pt = seg_end;

            let seg_dir = seg_end - seg_start;
            let dir_cross = cross(ray_dir, seg_dir);
            let ray_intersection_mag = cross(seg_start - ray_origin, seg_dir) / dir_cross;
            let seg_intersection_mag = cross(seg_start - ray_origin, ray_dir) / dir_cross;

            if dir_cross != 0.
                && 0. <= ray_intersection_mag
                && 0. <= seg_intersection_mag
                && seg_intersection_mag <= 1.
            {
                ray_origin + (ray_intersection_mag * ray_dir)
            } else {
                Point2::new(INFINITY, INFINITY)
            }
        }).fold(
            (Point2::new(INFINITY, INFINITY), INFINITY),
            |(closest_collision, smallest_distance), collision_coords| {
                let collision_distance = distance(&ray_origin, &collision_coords);
                if collision_distance < smallest_distance {
                    (collision_coords, collision_distance)
                } else {
                    (closest_collision, smallest_distance)
                }
            },
        );

    if smallest_distance == INFINITY {
        None
    } else {
        Some((nearest_collision, smallest_distance))
    }
}
