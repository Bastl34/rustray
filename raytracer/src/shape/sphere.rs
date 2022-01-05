use nalgebra::Isometry3;

use parry3d::query::{Ray, RayCast};
use parry3d::shape::Ball;

use crate::shape::Shape;
use crate::shape::ShapeBasics;
use crate::shape::Material;

pub struct Sphere
{
    pub basic: ShapeBasics,
    name: String,

    ball: Ball
}

impl Shape for Sphere
{
    fn get_material(&self) -> &Material
    {
        &self.basic.material
    }

    fn calc_bbox(&mut self)
    {
        self.basic.b_box = self.ball.aabb(&self.basic.trans);
    }

    fn intersect_b_box(&self, ray: &Ray) -> Option<f32>
    {
        self.basic.b_box.cast_ray(&self.basic.trans, ray, std::f32::MAX, false)
    }

    fn intersect(&self, ray: &Ray) -> Option<f32>
    {
        self.ball.cast_ray(&self.basic.trans, ray, std::f32::MAX, false)
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