use nalgebra::{Isometry3, Matrix4, Vector3, Point2, Point3};
use parry3d::query::{Ray};

use parry3d::bounding_volume::AABB;

use image::{DynamicImage, GenericImageView, Pixel};

pub mod sphere;
pub mod mesh;

pub trait Shape
{
    fn get_name(&self) -> &String;
    fn get_material(&self) -> &Material;
    fn get_basic(&self) -> &ShapeBasics;

    fn calc_bbox(&mut self);
    fn intersect_b_box(&self, ray: &Ray) -> Option<f32>;
    fn intersect(&self, ray: &Ray) -> Option<(f32, Vector3<f32>, u32)>;

    fn get_uv(&self, hit: Point3<f32>, face_id: u32) -> Point2<f32>;

    fn update(&mut self)
    {
        self.calc_bbox();
    }
}

#[derive(Debug)]
pub struct Material
{
    pub anmbient_color: Vector3<f32>,
    pub diffuse_color: Vector3<f32>,
    pub specular_color: Vector3<f32>,
    pub alpha: f32,
    pub shininess: f32,
    pub reflectivity: f32,
    pub refraction_index: f32,

    pub texture: DynamicImage
}

impl Material
{
    pub fn new() -> Material
    {
        Material
        {
            anmbient_color: Vector3::<f32>::new(1.0, 1.0, 1.0),
            diffuse_color: Vector3::<f32>::new(1.0, 1.0, 1.0),
            specular_color: Vector3::<f32>::new(1.0, 1.0, 1.0),
            alpha: 1.0,
            shininess: 1.0,
            reflectivity: 0.0,
            refraction_index: 0.0,

            texture: DynamicImage::new_rgb8(0,0)
        }
    }

    pub fn print(&self)
    {
        println!("anmbient_color: {:?}", self.anmbient_color);
        println!("diffuse_color: {:?}", self.diffuse_color);
        println!("specular_color: {:?}", self.specular_color);
        println!("alpha: {:?}", self.alpha);
        println!("shininess: {:?}", self.shininess);
        println!("reflectivity: {:?}", self.reflectivity);
        println!("refraction_index: {:?}", self.refraction_index);
        println!("texture: {:?}", self.texture.width() > 0);

    }
}

pub struct ShapeBasics
{
    pub trans: Isometry3<f32>,
    tran_inverse: Matrix4<f32>,

    b_box: AABB,

    pub material: Material
}

impl ShapeBasics
{
    pub fn new() -> ShapeBasics
    {
        ShapeBasics
        {
            trans: Isometry3::<f32>::identity(),
            tran_inverse: Matrix4::<f32>::identity(),
            b_box: AABB::new_invalid(),
            material: Material::new()
        }
    }

    pub fn get_mat(&mut self) -> Matrix4<f32>
    {
        self.trans.to_homogeneous()
    }

    pub fn load_texture(&mut self, path: &str)
    {
        self.material.texture = image::open(path).unwrap();
    }

    pub fn has_texture(&self) -> bool
    {
        self.material.texture.dimensions().0 > 0
    }

    pub fn texture_dimension(&self) -> (u32, u32)
    {
        self.material.texture.dimensions()
    }

    pub fn get_texture_pixel(&self, x: u32, y: u32) -> Vector3<f32>
    {
        if !self.has_texture()
        {
            return Vector3::<f32>::new(0.0, 0.0, 0.0);
        }

        let pixel = self.material.texture.get_pixel(x, y);
        let rgb = pixel.to_rgb();

        Vector3::<f32>::new
        (
            (rgb[0] as f32) / 255.0,
            (rgb[1] as f32) / 255.0,
            (rgb[2] as f32) / 255.0
        )
    }

    pub fn calc_inverse(&mut self)
    {
        //because we are dealing with 4x4 matrices: unwrap should be fine
        self.tran_inverse = self.trans.to_homogeneous().try_inverse().unwrap();
    }
}