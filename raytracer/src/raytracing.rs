use crate::shape::Shape;
use crate::pixel_color::PixelColor;
use crate::helper;

pub struct Raytracing
{
    scene: Vec<Box<dyn Shape + Send + Sync>>
}

impl Raytracing
{
    pub fn new() -> Raytracing
    {
        Raytracing
        {
            scene: vec![]
        }
    }

    pub fn render(&self, x: i32, y: i32) -> PixelColor
    {
        let r = helper::rand(0, 255);
        let g = helper::rand(0, 255);
        let b = helper::rand(0, 255);

        PixelColor { r: r, g: g, b: b, x: x, y: y }
    }
}