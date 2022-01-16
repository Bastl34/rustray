use nalgebra::{Point3, Vector3};

use crate::shape::Shape;

use crate::shape::sphere::Sphere;
use crate::shape::mesh::Mesh;

#[derive(PartialEq)]
pub enum LightType
{
    Directional,
    Point
}

pub struct Light
{
    pub pos: Point3<f32>,
    pub dir: Vector3<f32>,
    pub color: Vector3<f32>,
    pub intensity: f32,
    pub light_type: LightType
}

pub struct Scene
{
    pub items: Vec<Box<dyn Shape + Send + Sync>>,
    pub lights: Vec<Box<Light>>,
}

impl Scene
{
    pub fn new() -> Scene
    {
        Scene
        {
            items: vec![],
            lights: vec![]
        }
    }

    pub fn init_with_some_objects(&mut self)
    {
        self.init_objects();
        self.init_lights();
    }

    pub fn init_lights(&mut self)
    {
        /*
        self.lights.push(Box::new(Light
        {
            pos: Point3::new(0.0, 0.0, 0.0),
            dir: Vector3::new(1.0f32, -1.0, -1.0),
            color: Vector3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
            light_type: LightType::Directional
        }));
         */

        self.lights.push(Box::new(Light
        {
            pos: Point3::new(-5.0, -5.0, -5.0),
            dir: Vector3::new(1.0f32, 1.0, -1.0),
            color: Vector3::new(1.0, 1.0, 1.0),
            intensity: 150.0,
            light_type: LightType::Point
        }));

        self.lights.push(Box::new(Light
        {
            pos: Point3::new(5.0, 5.0, -10.0),
            dir: Vector3::new(-1.0f32, -1.0, -1.0),
            color: Vector3::new(1.0, 0.0, 0.0),
            intensity: 150.0,
            light_type: LightType::Point
        }));
    }

    pub fn init_objects(&mut self)
    {
        // ******************** some spheres ********************
        let mut sphere_back = Box::new(Sphere::new_with_pos(1.0, 0.0, -10.0, 1.0));
        sphere_back.basic.material.anmbient_color.x = 1.0;
        sphere_back.basic.material.anmbient_color.y = 0.0;
        sphere_back.basic.material.anmbient_color.z = 0.0;
        sphere_back.basic.material.reflectivity = 0.3;
        sphere_back.basic.material.alpha = 0.1;
        sphere_back.basic.material.refraction_index = 1.5;

        let mut sphere_front = Box::new(Sphere::new_with_pos(0.0, 0.0, -5.0, 3.0));
        sphere_front.basic.material.anmbient_color.x = 1.0;
        sphere_front.basic.material.anmbient_color.y = 1.0;
        sphere_front.basic.material.anmbient_color.z = 1.0;
        sphere_front.basic.material.reflectivity = 0.3;
        sphere_front.basic.material.alpha = 0.1;
        sphere_front.basic.material.refraction_index = 1.5;

        let mut sphere_left = Box::new(Sphere::new_with_pos(-7.0, 0.0, -20.0, 4.0));
        sphere_left.basic.material.anmbient_color.x = 0.0;
        sphere_left.basic.material.anmbient_color.y = 1.0;
        sphere_left.basic.material.anmbient_color.z = 0.0;
        sphere_left.basic.material.reflectivity = 0.5;
        sphere_left.basic.material.alpha = 0.8;
        sphere_left.basic.material.refraction_index = 1.5;

        let mut sphere_right = Box::new(Sphere::new_with_pos(7.0, -2.5, -18.0, 3.0));
        sphere_right.basic.material.anmbient_color.x = 0.0;
        sphere_right.basic.material.anmbient_color.y = 0.0;
        sphere_right.basic.material.anmbient_color.z = 1.0;
        sphere_right.basic.material.reflectivity = 0.5;
        sphere_right.basic.material.alpha = 0.8;
        sphere_right.basic.material.refraction_index = 1.5;

        let mut sphere_mirror = Box::new(Sphere::new_with_pos(-6.0, 2.5, -7.0, 1.0));
        sphere_mirror.basic.material.anmbient_color.x = 1.0;
        sphere_mirror.basic.material.anmbient_color.y = 1.0;
        sphere_mirror.basic.material.anmbient_color.z = 1.0;
        sphere_mirror.basic.material.reflectivity = 1.0;
        sphere_mirror.basic.material.alpha = 1.0;
        sphere_mirror.basic.material.refraction_index = 1.5;

        let mut sphere_not_visible = Box::new(Sphere::new_with_pos(7.0, 0.0, 10.0, 3.0));
        sphere_not_visible.basic.material.anmbient_color.x = 1.0;
        sphere_not_visible.basic.material.anmbient_color.y = 1.0;
        sphere_not_visible.basic.material.anmbient_color.z = 1.0;
        sphere_not_visible.basic.material.reflectivity = 0.5;

        // ******************** some meshes ********************
        let points_floor = vec!
        [
            Point3::new(-1000.0, -5.5, 2.0),
            Point3::new(1000.0, -5.5, 2.0),
            Point3::new(1000.0, -5.5, -50.0),
            Point3::new(-1000.0, -5.5, -50.0),
        ];

        let indices_floor = vec![[0u32, 1, 2], [0, 2, 3]];

        let mut mesh_floor = Box::new(Mesh::new_with_data(points_floor, indices_floor));

        mesh_floor.basic.material.anmbient_color.x = 0.5;
        mesh_floor.basic.material.anmbient_color.y = 0.5;
        mesh_floor.basic.material.anmbient_color.z = 1.0;
        mesh_floor.basic.material.reflectivity = 0.5;

        let points_back = vec!
        [
            Point3::new(-1000.0, -50.0, -20.0),
            Point3::new(1000.0, -50.0, -20.0),
            Point3::new(1000.0, 50.0, -20.0),
            Point3::new(-1000.0, 50.0, -20.0),
        ];

        let indices_back = vec![[0u32, 1, 2], [0, 2, 3]];

        let mut mesh_back = Box::new(Mesh::new_with_data(points_back, indices_back));

        mesh_back.basic.material.anmbient_color.x = 0.5;
        mesh_back.basic.material.anmbient_color.y = 0.5;
        mesh_back.basic.material.anmbient_color.z = 1.0;
        mesh_back.basic.material.reflectivity = 0.5;

        let points_left = vec!
        [
            Point3::new(-10.0, -500.0, 2.0),
            Point3::new(-10.0, -500.0, -50.0),
            Point3::new(-10.0, 500.0, -50.0),
            Point3::new(-10.0, 500.0, 2.0),
        ];

        let indices_left = vec![[0u32, 1, 2], [0, 2, 3]];

        let mut mesh_left = Box::new(Mesh::new_with_data(points_left, indices_left));

        mesh_left.basic.material.anmbient_color.x = 1.0;
        mesh_left.basic.material.anmbient_color.y = 0.0;
        mesh_left.basic.material.anmbient_color.z = 0.0;
        mesh_left.basic.material.reflectivity = 0.5;

        let points_right = vec!
        [
            Point3::new(10.0, -500.0, 2.0),
            Point3::new(10.0, -500.0, -50.0),
            Point3::new(10.0, 500.0, -50.0),
            Point3::new(10.0, 500.0, 2.0),
        ];

        let indices_right = vec![[0u32, 1, 2], [0, 2, 3]];

        let mut mesh_right = Box::new(Mesh::new_with_data(points_right, indices_right));

        mesh_right.basic.material.anmbient_color.x = 0.0;
        mesh_right.basic.material.anmbient_color.y = 1.0;
        mesh_right.basic.material.anmbient_color.z = 0.0;
        mesh_right.basic.material.reflectivity = 0.5;

        let points_top = vec!
        [
            Point3::new(-1000.0, 5.5, 2.0),
            Point3::new(1000.0, 5.5, 2.0),
            Point3::new(1000.0, 5.5, -50.0),
            Point3::new(-1000.0, 5.5, -50.0),
        ];

        let indices_top = vec![[0u32, 1, 2], [0, 2, 3]];

        let mut mesh_top = Box::new(Mesh::new_with_data(points_top, indices_top));

        mesh_top.basic.material.anmbient_color.x = 0.5;
        mesh_top.basic.material.anmbient_color.y = 0.5;
        mesh_top.basic.material.anmbient_color.z = 1.0;
        mesh_top.basic.material.reflectivity = 0.5;

        let points_behind = vec!
        [
            Point3::new(-1000.0, -50.0, 2.0),
            Point3::new(1000.0, -50.0, 2.0),
            Point3::new(1000.0, 50.0, 2.0),
            Point3::new(-1000.0, 50.0, 2.0),
        ];

        let indices_behind = vec![[0u32, 1, 2], [0, 2, 3]];

        let mut mesh_behind = Box::new(Mesh::new_with_data(points_behind, indices_behind));

        mesh_behind.basic.material.anmbient_color.x = 1.0;
        mesh_behind.basic.material.anmbient_color.y = 0.5;
        mesh_behind.basic.material.anmbient_color.z = 0.5;
        mesh_behind.basic.material.reflectivity = 0.5;

        self.items.push(sphere_back);
        self.items.push(sphere_front);
        self.items.push(sphere_left);
        self.items.push(sphere_right);
        self.items.push(sphere_not_visible);
        self.items.push(sphere_mirror);
        self.items.push(mesh_floor);
        self.items.push(mesh_back);
        self.items.push(mesh_left);
        self.items.push(mesh_right);
        self.items.push(mesh_top);
        self.items.push(mesh_behind);
    }
}