use crate::shape::Shape;
use crate::pixel_color::PixelColor;
use crate::helper;

use crate::shape::sphere::Sphere;

use nalgebra::Perspective3;

pub struct Raytracing
{
    scene: Vec<Box<dyn Shape + Send + Sync>>,

    projection: Perspective3<f32>
}

impl Raytracing
{
    pub fn new() -> Raytracing
    {
        Raytracing
        {
            scene: vec![],
            projection: Perspective3::<f32>::new(1.0f32, 0.0f32, 0.001, 1000.0)
        }
    }

    pub fn init_camera(&mut self, width: u32, height: u32)
    {
        let aspect = width as f32 / height as f32;
        let fov = 3.14 / 2.0;
        self.projection = Perspective3::new(aspect, fov, 0.001, 1000.0);
    }

    pub fn init_with_some_objects(&mut self)
    {
        let sphere = Box::new(Sphere::new_with_pos(10.0f32, 10.0f32, 5.0f32, 5.0f32));
        let sphere1 = Box::new(Sphere::new_with_pos(-10.0f32, 10.0f32, 5.0f32, 4.0f32));
        let sphere2 = Box::new(Sphere::new_with_pos(-20.0f32, -20.0f32, -5.0f32, 3.0f32));

        self.scene.push(sphere);
        self.scene.push(sphere1);
        self.scene.push(sphere2);
    }

    pub fn render(&self, x: i32, y: i32) -> PixelColor
    {
        let r = helper::rand(0, 255);
        let g = helper::rand(0, 255);
        let b = helper::rand(0, 255);

        PixelColor { r: r, g: g, b: b, x: x, y: y }
    }
}