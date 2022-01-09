use nalgebra::{Point3};

use crate::shape::Shape;

use crate::shape::sphere::Sphere;
use crate::shape::mesh::Mesh;

pub struct Scene
{
    pub items: Vec<Box<dyn Shape + Send + Sync>>,
}

impl Scene
{
    pub fn new() -> Scene
    {
        Scene { items: vec![] }
    }

    pub fn init_with_some_objects(&mut self)
    {
        let mut sphere = Box::new(Sphere::new_with_pos(0.0, 0.0, -5.0, 1.0));
        sphere.basic.material.anmbient_color.x = 1.0;
        sphere.basic.material.anmbient_color.y = 0.0;
        sphere.basic.material.anmbient_color.z = 0.0;

        let mut sphere_1 = Box::new(Sphere::new_with_pos(0.0, 0.0, -12.0, 5.0));
        sphere_1.basic.material.anmbient_color.x = 0.0;
        sphere_1.basic.material.anmbient_color.y = 1.0;
        sphere_1.basic.material.anmbient_color.z = 1.0;

        let mut sphere2 = Box::new(Sphere::new_with_pos(-10.0, 0.0, -20.0, 4.0));
        sphere2.basic.material.anmbient_color.x = 0.0;
        sphere2.basic.material.anmbient_color.y = 1.0;
        sphere2.basic.material.anmbient_color.z = 0.0;

        let mut sphere2_1 = Box::new(Sphere::new_with_pos(-12.0, 0.0, -25.0, 4.0));
        sphere2_1.basic.material.anmbient_color.x = 1.0;
        sphere2_1.basic.material.anmbient_color.y = 1.0;
        sphere2_1.basic.material.anmbient_color.z = 0.0;

        let mut sphere3 = Box::new(Sphere::new_with_pos(10.0, 0.0, -20.0, 3.0));
        sphere3.basic.material.anmbient_color.x = 0.0;
        sphere3.basic.material.anmbient_color.y = 0.0;
        sphere3.basic.material.anmbient_color.z = 1.0;

        let mut sphere_away = Box::new(Sphere::new_with_pos(10.0, 0.0, 10.0, 3.0));
        sphere_away.basic.material.anmbient_color.x = 1.0;
        sphere_away.basic.material.anmbient_color.y = 1.0;
        sphere_away.basic.material.anmbient_color.z = 1.0;


        let points = vec!
        [
            Point3::new(-1000.0, -5.5, 0.0),
            Point3::new(1000.0, -5.5, 0.0),
            Point3::new(1000.0, -5.5, -1000.0),
            Point3::new(-1000.0, -5.5, -1000.0),
        ];

        let indices = vec![[0u32, 1, 2], [0, 2, 3]];

        let mut mesh = Box::new(Mesh::new_with_data(points, indices));

        mesh.basic.material.anmbient_color.x = 0.5;
        mesh.basic.material.anmbient_color.y = 0.5;
        mesh.basic.material.anmbient_color.z = 1.0;

        self.items.push(sphere);
        self.items.push(sphere_1);
        self.items.push(sphere2);
        self.items.push(sphere2_1);
        self.items.push(sphere3);
        self.items.push(sphere_away);
        self.items.push(mesh);
    }
}