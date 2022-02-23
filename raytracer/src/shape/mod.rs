use nalgebra::{Matrix4, Vector3, Point2, Point3, Rotation3};
use parry3d::query::{Ray};

use parry3d::bounding_volume::AABB;

use image::{DynamicImage, GenericImageView, Pixel, RgbaImage};

use crate::helper::approx_equal;

pub mod sphere;
pub mod mesh;

pub trait Shape
{
    fn get_name(&self) -> &String;
    fn get_material(&self) -> &Material;
    fn get_basic(&self) -> &ShapeBasics;
    fn get_basic_mut(&mut self) -> &mut ShapeBasics;

    fn calc_bbox(&mut self);

    fn intersect_b_box(&self, ray: &Ray) -> Option<f32>;
    fn intersect(&self, ray: &Ray) -> Option<(f32, Vector3<f32>, u32)>;

    fn get_uv(&self, hit: Point3<f32>, face_id: u32) -> Point2<f32>;

    fn update(&mut self)
    {
        self.calc_bbox();
        self.get_basic_mut().calc_inverse();
    }
}

#[derive(Debug)]
pub struct Material
{
    pub ambient_color: Vector3<f32>,
    pub base_color: Vector3<f32>,
    pub specular_color: Vector3<f32>,

    pub texture_ambient_path: String,
    pub texture_base_path: String,
    pub texture_specular_path: String,
    pub texture_normal_path: String,
    pub texture_alpha_path: String,

    pub texture_ambient: DynamicImage,
    pub texture_base: DynamicImage,
    pub texture_specular: DynamicImage,
    pub texture_normal: DynamicImage,
    pub texture_alpha: DynamicImage,

    pub alpha: f32,
    pub shininess: f32,
    pub reflectivity: f32,
    pub refraction_index: f32,

    pub normal_map_strength: f32,

    pub cast_shadow: bool,
    pub receive_shadow: bool,
    pub shadow_softness: f32,

    pub surface_roughness: f32,

    pub smooth_shading: bool,
}

impl Material
{
    pub fn new() -> Material
    {
        Material
        {
            ambient_color: Vector3::<f32>::new(0.0, 0.0, 0.0),
            base_color: Vector3::<f32>::new(1.0, 1.0, 1.0),
            specular_color: Vector3::<f32>::new(0.8, 0.8, 0.8),

            texture_ambient_path: String::new(),
            texture_base_path: String::new(),
            texture_specular_path: String::new(),
            texture_normal_path: String::new(),
            texture_alpha_path: String::new(),

            texture_ambient: DynamicImage::new_rgb8(0,0),
            texture_base: DynamicImage::new_rgb8(0,0),
            texture_specular: DynamicImage::new_rgb8(0,0),
            texture_normal: DynamicImage::new_rgb8(0,0),
            texture_alpha: DynamicImage::new_rgb8(0,0),

            alpha: 1.0,
            shininess: 150.0,
            reflectivity: 0.0,
            refraction_index: 1.0,

            normal_map_strength: 1.0,

            cast_shadow: true,
            receive_shadow: true,
            shadow_softness: 0.01,

            surface_roughness: 0.0,

            smooth_shading: true
        }
    }

    pub fn apply_diff(&mut self, new_mat: &Material)
    {
        let default_material = Material::new();

        // ********** colors **********

        // ambient
        if
            !approx_equal(default_material.ambient_color.x, new_mat.ambient_color.x)
            ||
            !approx_equal(default_material.ambient_color.y, new_mat.ambient_color.y)
            ||
            !approx_equal(default_material.ambient_color.z, new_mat.ambient_color.z)
        {
            self.ambient_color = new_mat.ambient_color;
        }

        // ambient
        if
            !approx_equal(default_material.base_color.x, new_mat.base_color.x)
            ||
            !approx_equal(default_material.base_color.y, new_mat.base_color.y)
            ||
            !approx_equal(default_material.base_color.z, new_mat.base_color.z)
        {
            self.base_color = new_mat.base_color;
        }

        // specular
        if
            !approx_equal(default_material.specular_color.x, new_mat.specular_color.x)
            ||
            !approx_equal(default_material.specular_color.y, new_mat.specular_color.y)
            ||
            !approx_equal(default_material.specular_color.z, new_mat.specular_color.z)
        {
            self.specular_color = new_mat.specular_color;
        }


        // ********** textures **********

        // ambient
        if default_material.texture_ambient_path != new_mat.texture_ambient_path
        {
            self.texture_ambient = new_mat.texture_ambient.clone();
            self.texture_ambient_path = new_mat.texture_ambient_path.clone();
        }

        // base
        if default_material.texture_base_path != new_mat.texture_base_path
        {
            self.texture_base = new_mat.texture_base.clone();
            self.texture_base_path = new_mat.texture_base_path.clone();
        }

        // specular
        if default_material.texture_specular_path != new_mat.texture_specular_path
        {
            self.texture_specular = new_mat.texture_specular.clone();
            self.texture_specular_path = new_mat.texture_specular_path.clone();
        }

        // normal
        if default_material.texture_normal_path != new_mat.texture_normal_path
        {
            self.texture_normal = new_mat.texture_normal.clone();
            self.texture_normal_path = new_mat.texture_normal_path.clone();
        }

        // alpha
        if default_material.texture_alpha_path != new_mat.texture_alpha_path
        {
            self.texture_alpha = new_mat.texture_alpha.clone();
            self.texture_alpha_path = new_mat.texture_alpha_path.clone();
        }

        // ********** other attributes **********
        if !approx_equal(default_material.alpha, new_mat.alpha) { self.alpha = new_mat.alpha; }
        if !approx_equal(default_material.shininess, new_mat.shininess) { self.shininess = new_mat.shininess; }
        if !approx_equal(default_material.reflectivity, new_mat.reflectivity) { self.reflectivity = new_mat.reflectivity; }
        if !approx_equal(default_material.refraction_index, new_mat.refraction_index) { self.refraction_index = new_mat.refraction_index; }

        if !approx_equal(default_material.normal_map_strength, new_mat.normal_map_strength) { self.normal_map_strength = new_mat.normal_map_strength; }

        if default_material.cast_shadow != new_mat.cast_shadow { self.cast_shadow = new_mat.cast_shadow; }
        if default_material.receive_shadow != new_mat.receive_shadow { self.receive_shadow = new_mat.receive_shadow; }
        if !approx_equal(default_material.shadow_softness, new_mat.shadow_softness) { self.shadow_softness = new_mat.shadow_softness; }

        if !approx_equal(default_material.surface_roughness, new_mat.surface_roughness) { self.surface_roughness = new_mat.surface_roughness; }

        if default_material.smooth_shading != new_mat.smooth_shading { self.smooth_shading = new_mat.smooth_shading; }
    }

    pub fn print(&self)
    {
        println!("ambient_color: {:?}", self.ambient_color);
        println!("base_color: {:?}", self.base_color);
        println!("specular_color: {:?}", self.specular_color);

        println!("texture_ambient: {:?}", self.texture_ambient.width() > 0);
        println!("texture_base: {:?}", self.texture_base.width() > 0);
        println!("texture_specular: {:?}", self.texture_specular.width() > 0);
        println!("texture_normal: {:?}", self.texture_normal.width() > 0);
        println!("texture_alpha: {:?}", self.texture_alpha.width() > 0);

        println!("alpha: {:?}", self.alpha);
        println!("shininess: {:?}", self.shininess);
        println!("reflectivity: {:?}", self.reflectivity);
        println!("refraction_index: {:?}", self.refraction_index);

        println!("normal_map_strength: {:?}", self.normal_map_strength);

        println!("cast_shadow: {:?}", self.cast_shadow);
        println!("receive_shadow: {:?}", self.receive_shadow);
        println!("shadow_softness: {:?}", self.shadow_softness);

        println!("surface_roughness: {:?}", self.surface_roughness);

        println!("smooth_shading: {:?}", self.smooth_shading);
    }
}

#[derive(Clone, Copy)]
pub enum TextureType
{
    Base,
    Ambient,
    Specular,
    Normal,
    Alpha
}

pub struct ShapeBasics
{
    pub id: u32,
    pub visible: bool,
    pub trans: Matrix4<f32>,
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
            id: 0,
            visible: true,
            trans: Matrix4::<f32>::identity(),
            tran_inverse: Matrix4::<f32>::identity(),
            b_box: AABB::new_invalid(),
            material: Material::new()
        }
    }

    pub fn get_mat(&mut self) -> Matrix4<f32>
    {
        self.trans
    }

    pub fn apply_transformation(&mut self, translation: Vector3<f32>, scale: Vector3<f32>, rotation: Vector3<f32>)
    {
        let translation = nalgebra::Isometry3::translation(translation.x, translation.y, translation.z).to_homogeneous();
        let scale = Matrix4::new_nonuniform_scaling(&scale);

        //use a different rotation matrix for each axis to get the desired result
        //https://www.reddit.com/r/rust/comments/heuc9k/nalgebras_awkward_euler_angles_method/
        //https://github.com/dimforge/nalgebra/issues/269
        let rotation_x  = Rotation3::from_euler_angles(rotation.x, 0.0, 0.0).to_homogeneous();
        let rotation_y  = Rotation3::from_euler_angles(0.0, rotation.y, 0.0).to_homogeneous();
        let rotation_z  = Rotation3::from_euler_angles(0.0, 0.0, rotation.z).to_homogeneous();

        self.trans = self.trans * translation;
        self.trans = self.trans * scale;
        self.trans = self.trans * rotation_x;
        self.trans = self.trans * rotation_y;
        self.trans = self.trans * rotation_z;

        self.calc_inverse();
    }

    pub fn get_inverse_ray(&self, ray: &Ray) -> Ray
    {
        let ray_inverse_start = self.tran_inverse * ray.origin.to_homogeneous();
        let ray_inverse_dir = self.tran_inverse * ray.dir.to_homogeneous();

        Ray::new(Point3::from_homogeneous(ray_inverse_start).unwrap(), Vector3::from_homogeneous(ray_inverse_dir).unwrap())
    }

    pub fn load_texture(&mut self, path: &str, tex_type: TextureType)
    {
        println!("loading texture: {}", path);

        let tex = image::open(path).unwrap();
        match tex_type
        {
            TextureType::Base =>
            {
                self.material.texture_base_path = path.to_string();
                self.material.texture_base = tex;
            },
            TextureType::Ambient =>
            {
                self.material.texture_ambient_path = path.to_string();
                self.material.texture_ambient = tex;
            },
            TextureType::Specular =>
            {
                self.material.texture_specular_path = path.to_string();
                self.material.texture_specular = tex;
            },
            TextureType::Normal =>
            {
                self.material.texture_normal_path = path.to_string();
                self.material.texture_normal = tex;
            },
            TextureType::Alpha =>
            {
                self.material.texture_alpha_path = path.to_string();
                self.material.texture_alpha = tex;
            },
        }
    }

    pub fn load_texture_buffer(&mut self, image: &RgbaImage, tex_type: TextureType)
    {
        println!("loading texture from buffer");

        match tex_type
        {
            TextureType::Base =>
            {
                self.material.texture_base_path = String::from("from buffer");
                self.material.texture_base = DynamicImage::ImageRgba8(image.clone());
            },
            TextureType::Ambient =>
            {
                self.material.texture_ambient_path = String::from("from buffer");
                self.material.texture_ambient = DynamicImage::ImageRgba8(image.clone());
            },
            TextureType::Specular =>
            {
                self.material.texture_specular_path = String::from("from buffer");
                self.material.texture_specular = DynamicImage::ImageRgba8(image.clone());
            },
            TextureType::Normal =>
            {
                self.material.texture_normal_path = String::from("from buffer");
                self.material.texture_normal = DynamicImage::ImageRgba8(image.clone());
            },
            TextureType::Alpha =>
            {
                self.material.texture_alpha_path = String::from("from buffer");
                self.material.texture_alpha = DynamicImage::ImageRgba8(image.clone());
            },
        }
    }

    pub fn has_texture(&self, tex_type: TextureType) -> bool
    {
        match tex_type
        {
            TextureType::Base => self.material.texture_base.width() > 0,
            TextureType::Ambient => self.material.texture_ambient.width() > 0,
            TextureType::Specular => self.material.texture_specular.width() > 0,
            TextureType::Normal => self.material.texture_normal.width() > 0,
            TextureType::Alpha => self.material.texture_alpha.width() > 0,
        }
    }

    pub fn texture_dimension(&self, tex_type: TextureType) -> (u32, u32)
    {
        match tex_type
        {
            TextureType::Base => self.material.texture_base.dimensions(),
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
            TextureType::Base => { pixel = self.material.texture_base.get_pixel(x, y); },
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
        self.tran_inverse = self.trans.try_inverse().unwrap();
    }
}