use nalgebra::{Isometry3, Vector3, Point2, Point3, ComplexField};

use parry3d::query::{Ray, RayCast};
use parry3d::shape::Ball;

use crate::shape::{Shape, ShapeBasics, Material};

pub struct Sphere
{
    pub basic: ShapeBasics,
    name: String,

    ball: Ball
}

impl Shape for Sphere
{
    fn get_name(&self) -> &String
    {
        &self.name
    }

    fn get_material(&self) -> &Material
    {
        &self.basic.material
    }

    fn get_basic(&self) -> &ShapeBasics
    {
        &self.basic
    }

    fn calc_bbox(&mut self)
    {
        self.basic.b_box = self.ball.aabb(&self.basic.trans);
    }

    fn intersect_b_box(&self, ray: &Ray) -> Option<f32>
    {
        //self.basic.b_box.cast_ray(&self.basic.trans, ray, std::f32::MAX, true)
        let trans = Isometry3::<f32>::identity();
        let solid = !(self.basic.material.alpha < 1.0);
        self.basic.b_box.cast_ray(&trans, ray, std::f32::MAX, solid)
    }

    fn intersect(&self, ray: &Ray) -> Option<(f32, Vector3<f32>, u32)>
    {
        let solid = !(self.basic.material.alpha < 1.0);
        let res = self.ball.cast_ray_and_get_normal(&self.basic.trans, ray, std::f32::MAX, solid);
        if let Some(res) = res
        {
            return Some((res.toi, res.normal, 0))
        }
        None
    }

    fn get_uv(&self, hit: Point3<f32>, _face_id: u32) -> Point2<f32>
    {
        // https://gamedev.stackexchange.com/questions/114412/how-to-get-uv-coordinates-for-sphere-cylindrical-projection

        /*
        let h_vec = (hit - (&self.basic.trans * Point3::<f32>::new(0.0, 0.0, 0.0))).normalize();
        //let n = Vector3::<f32>::new(hit.x, hit.y, hit.z).normalize();

        Point2::<f32>::new
        (
            (1.0 + (h_vec.z.atan2(h_vec.x) as f32) / std::f32::consts::PI) * 0.5,
            (h_vec.y / self.ball.radius).acos() as f32 / std::f32::consts::PI
        )*/

        //https://people.cs.clemson.edu/~dhouse/courses/405/notes/texture-maps.pdf

        let c = &self.basic.trans * Point3::<f32>::new(0.0, 0.0, 0.0);

        let theta = (-(hit.z - c.z)).atan2(hit.x - c.x);
        let u = (theta + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);

        let phi = ((-(hit.y - c.y)) / self.ball.radius).acos();
        let v = phi / std::f32::consts::PI;

        Point2::<f32>::new(u, -v)
    }
}

impl Sphere
{
    pub fn new(r: f32) -> Sphere
    {
        let mut sphere = Sphere
        {
            basic: ShapeBasics::new(),
            name: String::from("Sphere"),
            ball: Ball::new(r)
        };

        sphere.calc_bbox();

        sphere
    }

    pub fn new_with_pos(name: &str, x: f32, y: f32, z: f32, r: f32) -> Sphere
    {
        let mut sphere = Sphere
        {
            basic: ShapeBasics::new(),
            name: String::from(name),
            ball: Ball::new(r)
        };

        //sphere.basic.trans = Isometry3::translation(x, y, z) * Isometry3::rotation(Vector3::new(1.0, 0.0, 0.0));
        sphere.basic.trans = Isometry3::translation(x, y, z);


        sphere.calc_bbox();

        sphere
    }
}