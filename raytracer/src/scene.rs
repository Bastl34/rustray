use std::sync::Mutex;
use std::sync::Arc;

use crate::shape::Shape;

use crate::shape::sphere::Sphere;

pub struct Scene
{
    items: Vec<Box<dyn Shape + Send + Sync>>,
}

impl Scene
{
    pub fn new_arc_mut() -> Arc<Mutex<Scene>>
    {
        std::sync::Arc::new(std::sync::Mutex::new(Scene {items: vec![]}))
    }

    pub fn init_with_some_objects(&mut self)
    {
        let mut sphere = Box::new(Sphere::new_with_pos(0.0, 0.0, -5.0, 1.0));
        sphere.basic.material.anmbient_color.x = 1.0;
        sphere.basic.material.anmbient_color.y = 0.0;
        sphere.basic.material.anmbient_color.z = 0.0;
        sphere.basic.material.anmbient_color.w = 1.0;

        let mut sphere2 = Box::new(Sphere::new_with_pos(-10.0, 10.0, -10.0, 4.0));
        sphere2.basic.material.anmbient_color.x = 0.0;
        sphere2.basic.material.anmbient_color.y = 1.0;
        sphere2.basic.material.anmbient_color.z = 0.0;
        sphere2.basic.material.anmbient_color.w = 1.0;

        let mut sphere3 = Box::new(Sphere::new_with_pos(10.0, -10.0, -10.0, 3.0));
        sphere3.basic.material.anmbient_color.x = 0.0;
        sphere3.basic.material.anmbient_color.y = 0.0;
        sphere3.basic.material.anmbient_color.z = 1.0;
        sphere3.basic.material.anmbient_color.w = 1.0;

        let mut sphere_away = Box::new(Sphere::new_with_pos(10.0, -10.0, 10.0, 3.0));
        sphere_away.basic.material.anmbient_color.x = 1.0;
        sphere_away.basic.material.anmbient_color.y = 1.0;
        sphere_away.basic.material.anmbient_color.z = 1.0;
        sphere_away.basic.material.anmbient_color.w = 1.0;

        self.items.push(sphere);
        self.items.push(sphere2);
        self.items.push(sphere3);
        self.items.push(sphere_away);
    }
}