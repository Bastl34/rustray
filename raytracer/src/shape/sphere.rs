use nalgebra::Isometry3;

use parry3d::shape::Ball;

use crate::shape::Shape;
use crate::shape::ShapeBasics;

pub struct Sphere
{
    basic: ShapeBasics,
    name: String,

    ball: Ball
}

impl Shape for Sphere
{
     fn calc_bbox(&mut self)
    {
        self.basic.b_box = self.ball.aabb(&self.basic.trans);
    }
}

impl Sphere
{
    pub fn new(r: f32) -> Sphere
    {
        Sphere
        {
            basic: ShapeBasics::new(),
            name: String::from("Sphere"),
            ball: Ball::new(r)
        }
    }

    pub fn new_with_pos(x: f32, y: f32, z: f32, r: f32) -> Sphere
    {
        let mut sphere = Sphere
        {
            basic: ShapeBasics::new(),
            name: String::from("Sphere"),
            ball: Ball::new(r)
        };

        sphere.basic.trans = Isometry3::translation(x, y, z);

        sphere
    }
}