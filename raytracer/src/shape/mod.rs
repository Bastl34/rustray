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
    pub ambient_color: Vector3<f32>,
    pub diffuse_color: Vector3<f32>,
    pub specular_color: Vector3<f32>,
    pub alpha: f32,
    pub shininess: f32,
    pub reflectivity: f32,
    pub refraction_index: f32,

    pub texture_ambient: DynamicImage,
    pub texture_diffuse: DynamicImage,
    pub texture_specular: DynamicImage,
    pub texture_normal: DynamicImage,
    pub texture_alpha: DynamicImage,

    pub normal_map_strength: f32,
}

impl Material
{
    pub fn new() -> Material
    {
        Material
        {
            ambient_color: Vector3::<f32>::new(1.0, 1.0, 1.0),
            diffuse_color: Vector3::<f32>::new(1.0, 1.0, 1.0),
            specular_color: Vector3::<f32>::new(1.0, 1.0, 1.0),
            alpha: 1.0,
            shininess: 1.0,
            reflectivity: 0.0,
            refraction_index: 0.0,

            texture_ambient: DynamicImage::new_rgb8(0,0),
            texture_diffuse: DynamicImage::new_rgb8(0,0),
            texture_specular: DynamicImage::new_rgb8(0,0),
            texture_normal: DynamicImage::new_rgb8(0,0),
            texture_alpha: DynamicImage::new_rgb8(0,0),

            normal_map_strength: 1.0,
        }
    }

    pub fn print(&self)
    {
        println!("ambient_color: {:?}", self.ambient_color);
        println!("diffuse_color: {:?}", self.diffuse_color);
        println!("specular_color: {:?}", self.specular_color);
        println!("alpha: {:?}", self.alpha);
        println!("shininess: {:?}", self.shininess);
        println!("reflectivity: {:?}", self.reflectivity);
        println!("refraction_index: {:?}", self.refraction_index);

        println!("texture_ambient: {:?}", self.texture_ambient.width() > 0);
        println!("texture_diffuse: {:?}", self.texture_diffuse.width() > 0);
        println!("texture_specular: {:?}", self.texture_specular.width() > 0);
        println!("texture_normal: {:?}", self.texture_normal.width() > 0);
        println!("texture_alpha: {:?}", self.texture_alpha.width() > 0);

        println!("normal_map_strength: {:?}", self.normal_map_strength);
    }
}

#[derive(Clone, Copy)]
pub enum TextureType
{
    Diffuse,
    Ambient,
    Specular,
    Normal,
    Alpha
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

    pub fn load_texture(&mut self, path: &str, tex_type: TextureType)
    {
        let tex = image::open(path).unwrap();
        match tex_type
        {
            TextureType::Diffuse => { self.material.texture_diffuse = tex; },
            TextureType::Ambient => { self.material.texture_ambient = tex; },
            TextureType::Specular => { self.material.texture_specular = tex; },
            TextureType::Normal => { self.material.texture_normal = tex; },
            TextureType::Alpha => { self.material.texture_alpha = tex; },
        }
    }

    pub fn has_texture(&self, tex_type: TextureType) -> bool
    {
        match tex_type
        {
            TextureType::Diffuse => self.material.texture_diffuse.dimensions().0 > 0,
            TextureType::Ambient => self.material.texture_ambient.dimensions().0 > 0,
            TextureType::Specular => self.material.texture_specular.dimensions().0 > 0,
            TextureType::Normal => self.material.texture_normal.dimensions().0 > 0,
            TextureType::Alpha => self.material.texture_alpha.dimensions().0 > 0,
        }
    }

    pub fn texture_dimension(&self, tex_type: TextureType) -> (u32, u32)
    {
        match tex_type
        {
            TextureType::Diffuse => self.material.texture_diffuse.dimensions(),
            TextureType::Ambient => self.material.texture_ambient.dimensions(),
            TextureType::Specular => self.material.texture_specular.dimensions(),
            TextureType::Normal => self.material.texture_normal.dimensions(),
            TextureType::Alpha => self.material.texture_alpha.dimensions(),
        }
    }

    pub fn get_texture_pixel(&self, x: u32, y: u32, tex_type: TextureType) -> Vector3<f32>
    {
        if !self.has_texture(tex_type)
        {
            return Vector3::<f32>::new(0.0, 0.0, 0.0);
        }

        let pixel;

        match tex_type
        {
            TextureType::Diffuse => { pixel = self.material.texture_diffuse.get_pixel(x, y); },
            TextureType::Ambient => { pixel = self.material.texture_ambient.get_pixel(x, y); },
            TextureType::Specular => { pixel = self.material.texture_specular.get_pixel(x, y); },
            TextureType::Normal => { pixel = self.material.texture_normal.get_pixel(x, y); },
            TextureType::Alpha => { pixel = self.material.texture_alpha.get_pixel(x, y); },
        }

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