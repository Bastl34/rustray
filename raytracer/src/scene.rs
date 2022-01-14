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
        let mut sphere = Box::new(Sphere::new_with_pos(0.0, 0.0, -12.0, 1.0));
        sphere.basic.material.anmbient_color.x = 1.0;
        sphere.basic.material.anmbient_color.y = 0.0;
        sphere.basic.material.anmbient_color.z = 0.0;
        sphere.basic.material.reflectivity = 0.3;
        sphere.basic.material.alpha = 0.1;
        sphere.basic.material.refraction_index = 1.5;

        let mut sphere_1 = Box::new(Sphere::new_with_pos(0.0, 0.0, -5.0, 3.0));
        sphere_1.basic.material.anmbient_color.x = 0.0;
        sphere_1.basic.material.anmbient_color.y = 1.0;
        sphere_1.basic.material.anmbient_color.z = 1.0;
        sphere_1.basic.material.reflectivity = 0.3;
        sphere_1.basic.material.alpha = 0.1;
        sphere_1.basic.material.refraction_index = 1.5;

        let mut sphere2 = Box::new(Sphere::new_with_pos(-7.0, 0.0, -20.0, 4.0));
        sphere2.basic.material.anmbient_color.x = 0.0;
        sphere2.basic.material.anmbient_color.y = 1.0;
        sphere2.basic.material.anmbient_color.z = 0.0;
        sphere2.basic.material.reflectivity = 0.5;

        let mut sphere2_1 = Box::new(Sphere::new_with_pos(-7.0, 0.0, -25.0, 4.0));
        sphere2_1.basic.material.anmbient_color.x = 1.0;
        sphere2_1.basic.material.anmbient_color.y = 1.0;
        sphere2_1.basic.material.anmbient_color.z = 0.0;
        sphere2_1.basic.material.reflectivity = 0.5;

        let mut sphere3 = Box::new(Sphere::new_with_pos(7.0, 0.0, -20.0, 3.0));
        sphere3.basic.material.anmbient_color.x = 0.0;
        sphere3.basic.material.anmbient_color.y = 0.0;
        sphere3.basic.material.anmbient_color.z = 1.0;
        sphere3.basic.material.reflectivity = 0.5;

        let mut sphere_away = Box::new(Sphere::new_with_pos(7.0, 0.0, 10.0, 3.0));
        sphere_away.basic.material.anmbient_color.x = 1.0;
        sphere_away.basic.material.anmbient_color.y = 1.0;
        sphere_away.basic.material.anmbient_color.z = 1.0;
        sphere_away.basic.material.reflectivity = 0.5;

        let points = vec!
        [
            Point3::new(-1000.0, -5.5, 0.0),
            Point3::new(1000.0, -5.5, 0.0),
            Point3::new(1000.0, -5.5, -50.0),
            Point3::new(-1000.0, -5.5, -50.0),
        ];

        let indices = vec![[0u32, 1, 2], [0, 2, 3]];

        let mut mesh = Box::new(Mesh::new_with_data(points, indices));

        mesh.basic.material.anmbient_color.x = 0.5;
        mesh.basic.material.anmbient_color.y = 0.5;
        mesh.basic.material.anmbient_color.z = 1.0;
        mesh.basic.material.reflectivity = 0.5;

        let points2 = vec!
        [
            Point3::new(-1000.0, -50.0, -20.0),
            Point3::new(1000.0, -50.0, -20.0),
            Point3::new(1000.0, 50.0, -20.0),
            Point3::new(-1000.0, 50.0, -20.0),
        ];

        let indices2 = vec![[0u32, 1, 2], [0, 2, 3]];

        let mut mesh2 = Box::new(Mesh::new_with_data(points2, indices2));

        mesh2.basic.material.anmbient_color.x = 0.5;
        mesh2.basic.material.anmbient_color.y = 0.5;
        mesh2.basic.material.anmbient_color.z = 1.0;
        mesh2.basic.material.reflectivity = 0.5;

        let points3 = vec!
        [
            Point3::new(-10.0, -500.0, 0.0),
            Point3::new(-10.0, -500.0, -50.0),
            Point3::new(-10.0, 500.0, -50.0),
            Point3::new(-10.0, 500.0, 0.0),
        ];

        let indices3 = vec![[0u32, 1, 2], [0, 2, 3]];

        let mut mesh3 = Box::new(Mesh::new_with_data(points3, indices3));

        mesh3.basic.material.anmbient_color.x = 1.0;
        mesh3.basic.material.anmbient_color.y = 0.0;
        mesh3.basic.material.anmbient_color.z = 0.0;
        mesh3.basic.material.reflectivity = 0.5;

        let points4 = vec!
        [
            Point3::new(10.0, -500.0, 0.0),
            Point3::new(10.0, -500.0, -50.0),
            Point3::new(10.0, 500.0, -50.0),
            Point3::new(10.0, 500.0, 0.0),
        ];

        let indices4 = vec![[0u32, 1, 2], [0, 2, 3]];

        let mut mesh4 = Box::new(Mesh::new_with_data(points4, indices4));

        mesh4.basic.material.anmbient_color.x = 0.0;
        mesh4.basic.material.anmbient_color.y = 1.0;
        mesh4.basic.material.anmbient_color.z = 0.0;
        mesh4.basic.material.reflectivity = 0.5;

        let points5 = vec!
        [
            Point3::new(-1000.0, 5.5, 0.0),
            Point3::new(1000.0, 5.5, 0.0),
            Point3::new(1000.0, 5.5, -50.0),
            Point3::new(-1000.0, 5.5, -50.0),
        ];

        let indices5 = vec![[0u32, 1, 2], [0, 2, 3]];

        let mut mesh5 = Box::new(Mesh::new_with_data(points5, indices5));

        mesh5.basic.material.anmbient_color.x = 0.5;
        mesh5.basic.material.anmbient_color.y = 0.5;
        mesh5.basic.material.anmbient_color.z = 1.0;
        mesh5.basic.material.reflectivity = 0.5;

        self.items.push(sphere);
        self.items.push(sphere_1);

        /*
        self.items.push(sphere2);
        self.items.push(sphere2_1);
        self.items.push(sphere3);
        self.items.push(sphere_away);
        */
        self.items.push(mesh);
        self.items.push(mesh2);
        self.items.push(mesh3);
        self.items.push(mesh4);
        self.items.push(mesh5);
    }
}