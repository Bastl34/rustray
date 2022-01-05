use crate::shape::Shape;
use crate::pixel_color::PixelColor;
use crate::helper;

use crate::shape::sphere::Sphere;

use nalgebra::{Perspective3, Isometry3, Point3, Vector3};
use parry3d::query::{Ray};

pub struct Raytracing
{
    scene: Vec<Box<dyn Shape + Send + Sync>>,

    width: u32,
    height: u32,

    aspect_ratio: f32,

    fov: f32,
    fov_adjustment: f32,

    projection: Perspective3<f32>,
    view: Isometry3<f32>
}

impl Raytracing
{
    pub fn new() -> Raytracing
    {
        Raytracing
        {
            scene: vec![],

            width: 0,
            height: 0,
            aspect_ratio: 0.0,

            fov: 0.0,
            fov_adjustment: 0.0,

            projection: Perspective3::<f32>::new(1.0f32, 0.0f32, 0.001, 1000.0),
            view: Isometry3::<f32>::identity()
        }
    }

    pub fn init_camera(&mut self, width: u32, height: u32)
    {
        self.width = width;
        self.height = height;

        self.aspect_ratio = width as f32 / height as f32;
        self.fov = 3.14 / 2.0;

        self.fov_adjustment = (self.fov / 2.0).tan();

        self.projection = Perspective3::new(self.aspect_ratio, self.fov, 0.001, 1000.0);

        let eye    = Point3::new(0.0, 0.0, 1.0);
        let target = Point3::new(1.0, 0.0, 0.0);

        self.view = Isometry3::look_at_rh(&eye, &target, &Vector3::y());
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

        self.scene.push(sphere);
        self.scene.push(sphere2);
        self.scene.push(sphere3);
    }

    pub fn render(&self, x: i32, y: i32) -> PixelColor
    {
        //map x/y to -1 <=> +1
        let sensor_x = ((((x as f32 + 0.5) / self.width as f32) * 2.0 - 1.0) * self.aspect_ratio) * self.fov_adjustment;
        let sensor_y = (1.0 - ((y as f32 + 0.5) / self.height as f32) * 2.0) * self.fov_adjustment;

        let ray = Ray::new(Point3::origin(), Vector3::new(sensor_x, sensor_y, -1.0));

        let mut r = 0;
        let mut g = 0;
        let mut b = 0;

        for item in &self.scene
        {
            let intersection = item.intersect(&ray);

            if let Some(intersection) = intersection
            {
                let r_float = (*item).get_material().anmbient_color.x * 255.0;
                let g_float = (*item).get_material().anmbient_color.y * 255.0;
                let b_float = (*item).get_material().anmbient_color.z * 255.0;
                let alpha = (*item).get_material().anmbient_color.w;

                r = r_float as u8;
                g = g_float as u8;
                b = b_float as u8;
            }
        }

        PixelColor { r: r, g: g, b: b, x: x, y: y }
    }
}