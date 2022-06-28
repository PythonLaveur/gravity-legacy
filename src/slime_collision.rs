use bevy::prelude::*;

#[derive(Debug)]
pub enum Side {
    Left,
    Right,
    Top,
    Bottom,
    Inside,
}

impl Side {
    pub fn numerize(&self) -> i32 {
        match self {
            Self::Left => 3,
            Self::Right => 1,
            Self::Top => 2,
            Self::Bottom => 0,
            Self::Inside => 4,
        }
    }
}

#[derive(Debug)]
pub struct CollisionState {
    pub side_of_collision: Side,
    pub lenght_side: f32,
    pub depth: f32,
}

pub fn collision_test(
    a_pos: Vec3,
    a_size: Vec2,
    b_pos: Vec3,
    b_size: Vec2,
) -> Option<CollisionState> {
    let a_min = a_pos.truncate() - a_size / 2.0;
    let a_max = a_pos.truncate() + a_size / 2.0;

    let b_min = b_pos.truncate() - b_size / 2.0;
    let b_max = b_pos.truncate() + b_size / 2.0;

    // check to see if the two rectangles are intersecting
    if a_min.x < b_max.x && a_max.x > b_min.x && a_min.y < b_max.y && a_max.y > b_min.y {
        // check to see if we hit on the left or right side
        let (x_collision, x_depth, x_depth_final) =
            if a_min.x < b_min.x && a_max.x > b_min.x && a_max.x < b_max.x {
                (Side::Right, b_min.x - a_max.x, (b_min.x - a_max.x).abs())
            } else if a_min.x > b_min.x && a_min.x < b_max.x && a_max.x > b_max.x {
                (Side::Left, a_min.x - b_max.x, (a_min.x - b_max.x).abs())
            } else if a_size.x < b_size.x {
                (Side::Inside, -f32::INFINITY, (a_size.x).abs())
            } else {
                (Side::Inside, -f32::INFINITY, (b_size.x).abs())
            };

        // check to see if we hit on the top or bottom side
        let (y_collision, y_depth, y_depth_final) =
            if a_min.y < b_min.y && a_max.y > b_min.y && a_max.y < b_max.y {
                (Side::Top, b_min.y - a_max.y, (b_min.y - a_max.y).abs())
            } else if a_min.y > b_min.y && a_min.y < b_max.y && a_max.y > b_max.y {
                (Side::Bottom, a_min.y - b_max.y, (a_min.y - b_max.y).abs())
            } else if a_size.y < b_size.y {
                (Side::Inside, -f32::INFINITY, (a_size.y).abs())
            } else {
                (Side::Inside, -f32::INFINITY, (b_size.y).abs())
            };

        // if we had an "x" and a "y" collision, pick the "primary" side using penetration depth
        if y_depth.abs() < x_depth.abs() {
            Some(CollisionState {
                side_of_collision: y_collision,
                lenght_side: x_depth_final,
                depth: y_depth_final,
            })
        } else {
            Some(CollisionState {
                side_of_collision: x_collision,
                lenght_side: y_depth_final,
                depth: x_depth_final,
            })
        }
    } else {
        None
    }
}
