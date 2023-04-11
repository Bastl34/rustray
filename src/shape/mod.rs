use bvh::aabb::Bounded;
use bvh::bounding_hierarchy::BHShape;
use nalgebra::{Matrix4, Vector3, Point2, Point3, Rotation3, Vector4};
use parry3d::query::{Ray};

use parry3d::bounding_volume::Aabb;

use image::{DynamicImage, GenericImageView, Pixel};

use crate::helper::approx_equal;

pub mod sphere;
pub mod mesh;

pub trait Shape
{
    fn get_material(&self) -> &Material;
    fn get_basic(&self) -> &ShapeBasics;
    fn get_basic_mut(&mut self) -> &mut ShapeBasics;

    fn calc_bbox(&mut self);

    fn intersect_b_box(&self, ray: &Ray, force_not_solid: bool) -> Option<f32>;
    fn intersect(&self, ray: &Ray, force_not_solid: bool) -> Option<(f32, Vector3<f32>, u32)>;

    fn get_uv(&self, hit: Point3<f32>, face_id: u32) -> Point2<f32>;

    fn update(&mut self)
    {
        self.calc_bbox();
        self.get_basic_mut().calc_inverse();
    }

    fn init(&mut self)
    {
        self.calc_bbox();
        self.get_basic_mut().calc_inverse();
        self.get_basic_mut().init_animation_data();
    }
}

impl Bounded for Box<(dyn Shape + Send + Sync + 'static)>
{
    fn aabb(&self) -> bvh::aabb::AABB
    {
        let aabb = self.get_basic().b_box;
        let verts = aabb.vertices();

        let trans = self.get_basic().trans;

        let mut min = verts[0];
        let mut max = verts[0];

        for vert in &verts
        {
            let transformed = trans * vert.to_homogeneous();

            min.x = min.x.min(transformed.x);
            min.y = min.y.min(transformed.y);
            min.z = min.z.min(transformed.z);

            max.x = max.x.max(transformed.x);
            max.y = max.y.max(transformed.y);
            max.z = max.z.max(transformed.z);
        }

        let min = bvh::Point3::new(min.x, min.y, min.z);
        let max = bvh::Point3::new(max.x, max.y, max.z);

        bvh::aabb::AABB::with_bounds(min, max)
    }
}

impl BHShape for Box<(dyn Shape + Send + Sync + 'static)>
{
    fn set_bh_node_index(&mut self, index: usize)
    {
        self.get_basic_mut().node_index = index;
    }

    fn bh_node_index(&self) -> usize
    {
        self.get_basic().node_index
    }
}


#[derive(Debug)]
pub struct Material
{
    pub ambient_color: Vector3<f32>,
    pub base_color: Vector3<f32>,
    pub specular_color: Vector3<f32>,

    pub texture_ambient: DynamicImage,
    pub texture_base: DynamicImage,
    pub texture_specular: DynamicImage,
    pub texture_normal: DynamicImage,
    pub texture_alpha: DynamicImage,
    pub texture_roughness: DynamicImage,
    pub texture_ambient_occlusion: DynamicImage,
    pub texture_reflectivity: DynamicImage,

    pub alpha: f32,
    pub shininess: f32,
    pub reflectivity: f32,
    pub refraction_index: f32,

    pub normal_map_strength: f32,

    pub cast_shadow: bool,
    pub receive_shadow: bool,
    pub shadow_softness: f32,

    pub monte_carlo: bool,

    pub roughness: f32, //degree in rad (max PI/2)

    pub smooth_shading: bool,

    pub reflection_only: bool,
    pub backface_cullig: bool
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

            texture_ambient: DynamicImage::new_rgb8(0,0),
            texture_base: DynamicImage::new_rgb8(0,0),
            texture_specular: DynamicImage::new_rgb8(0,0),
            texture_normal: DynamicImage::new_rgb8(0,0),
            texture_alpha: DynamicImage::new_rgb8(0,0),
            texture_roughness: DynamicImage::new_rgb8(0,0),
            texture_ambient_occlusion: DynamicImage::new_rgb8(0,0),
            texture_reflectivity: DynamicImage::new_rgb8(0,0),

            alpha: 1.0,
            shininess: 150.0,
            reflectivity: 0.0,
            refraction_index: 1.0,

            normal_map_strength: 1.0,

            cast_shadow: true,
            receive_shadow: true,
            shadow_softness: 0.01,

            roughness: 0.0,

            monte_carlo: true,

            smooth_shading: true,

            reflection_only: false,
            backface_cullig: true,
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

        // base
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
        if default_material.texture_ambient != new_mat.texture_ambient
        {
            self.texture_ambient = new_mat.texture_ambient.clone();
        }

        // base
        if default_material.texture_base != new_mat.texture_base
        {
            self.texture_base = new_mat.texture_base.clone();
        }

        // specular
        if default_material.texture_specular != new_mat.texture_specular
        {
            self.texture_specular = new_mat.texture_specular.clone();
        }

        // normal
        if default_material.texture_normal != new_mat.texture_normal
        {
            self.texture_normal = new_mat.texture_normal.clone();
        }

        // alpha
        if default_material.texture_alpha != new_mat.texture_alpha
        {
            self.texture_alpha = new_mat.texture_alpha.clone();
        }

        // roughness
        if default_material.texture_roughness != new_mat.texture_roughness
        {
            self.texture_roughness = new_mat.texture_roughness.clone();
        }

        // ambient_occlusion
        if default_material.texture_ambient_occlusion != new_mat.texture_ambient_occlusion
        {
            self.texture_ambient_occlusion = new_mat.texture_ambient_occlusion.clone();
        }

        // metallic
        if default_material.texture_reflectivity != new_mat.texture_reflectivity
        {
            self.texture_reflectivity = new_mat.texture_reflectivity.clone();
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

        if !approx_equal(default_material.roughness, new_mat.roughness) { self.roughness = new_mat.roughness; }

        if default_material.monte_carlo != new_mat.monte_carlo { self.monte_carlo = new_mat.monte_carlo; }

        if default_material.smooth_shading != new_mat.smooth_shading { self.smooth_shading = new_mat.smooth_shading; }

        if default_material.reflection_only != new_mat.reflection_only { self.reflection_only = new_mat.reflection_only; }
        if default_material.backface_cullig != new_mat.backface_cullig { self.backface_cullig = new_mat.backface_cullig; }
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
        println!("texture_roughness: {:?}", self.texture_roughness.width() > 0);
        println!("texture_ambient_occlusion: {:?}", self.texture_ambient_occlusion.width() > 0);
        println!("texture_reflectivity: {:?}", self.texture_reflectivity.width() > 0);

        println!("alpha: {:?}", self.alpha);
        println!("shininess: {:?}", self.shininess);
        println!("reflectivity: {:?}", self.reflectivity);
        println!("refraction_index: {:?}", self.refraction_index);

        println!("normal_map_strength: {:?}", self.normal_map_strength);

        println!("cast_shadow: {:?}", self.cast_shadow);
        println!("receive_shadow: {:?}", self.receive_shadow);
        println!("shadow_softness: {:?}", self.shadow_softness);

        println!("roughness: {:?}", self.roughness);

        println!("monte_carlo: {:?}", self.monte_carlo);

        println!("smooth_shading: {:?}", self.smooth_shading);

        println!("reflection_only: {:?}", self.reflection_only);
        println!("backface_cullig: {:?}", self.backface_cullig);
    }

    pub fn remove_texture(&mut self, tex_type: TextureType)
    {
        match tex_type
        {
            TextureType::Base =>
            {
                self.texture_base = DynamicImage::new_rgb8(0,0);
            },
            TextureType::AmbientEmissive =>
            {
                self.texture_ambient = DynamicImage::new_rgb8(0,0);
            },
            TextureType::Specular =>
            {
                self.texture_specular = DynamicImage::new_rgb8(0,0);
            },
            TextureType::Normal =>
            {
                self.texture_normal = DynamicImage::new_rgb8(0,0);
            },
            TextureType::Alpha =>
            {
                self.texture_alpha = DynamicImage::new_rgb8(0,0);
            },
            TextureType::Roughness =>
            {
                self.texture_roughness = DynamicImage::new_rgb8(0,0);
            },
            TextureType::AmbientOcclusion =>
            {
                self.texture_ambient_occlusion = DynamicImage::new_rgb8(0,0);
            },
            TextureType::Reflectivity =>
            {
                self.texture_reflectivity = DynamicImage::new_rgb8(0,0);
            },
        }
    }

    pub fn load_texture(&mut self, path: &str, tex_type: TextureType)
    {
        println!("loading texture: {}", path);

        let tex = image::open(path).unwrap();
        match tex_type
        {
            TextureType::Base =>
            {
                self.texture_base = tex;
            },
            TextureType::AmbientEmissive =>
            {
                self.texture_ambient = tex;
            },
            TextureType::Specular =>
            {
                self.texture_specular = tex;
            },
            TextureType::Normal =>
            {
                self.texture_normal = tex;
            },
            TextureType::Alpha =>
            {
                self.texture_alpha = tex;
            },
            TextureType::Roughness =>
            {
                self.texture_roughness = tex;
            },
            TextureType::AmbientOcclusion =>
            {
                self.texture_ambient_occlusion = tex;
            },
            TextureType::Reflectivity =>
            {
                self.texture_reflectivity = tex;
            },
        }
    }

    pub fn load_texture_buffer(&mut self, image: &DynamicImage, tex_type: TextureType)
    {
        println!("loading texture from buffer: {:?}", tex_type);

        match tex_type
        {
            TextureType::Base =>
            {
                self.texture_base = image.clone();
            },
            TextureType::AmbientEmissive =>
            {
                self.texture_ambient = image.clone();
            },
            TextureType::Specular =>
            {
                self.texture_specular = image.clone();
            },
            TextureType::Normal =>
            {
                self.texture_normal = image.clone();
            },
            TextureType::Alpha =>
            {
                self.texture_alpha = image.clone();
            },
            TextureType::Roughness =>
            {
                self.texture_roughness = image.clone();
            },
            TextureType::AmbientOcclusion =>
            {
                self.texture_ambient_occlusion = image.clone();
            },
            TextureType::Reflectivity =>
            {
                self.texture_reflectivity = image.clone();
            },
        }
    }

    pub fn has_any_texture(&self) -> bool
    {
        self.texture_base.width() > 0
        ||
        self.texture_ambient.width() > 0
        ||
        self.texture_specular.width() > 0
        ||
        self.texture_normal.width() > 0
        ||
        self.texture_alpha.width() > 0
        ||
        self.texture_roughness.width() > 0
        ||
        self.texture_ambient_occlusion.width() > 0
        ||
        self.texture_reflectivity.width() > 0
    }

    pub fn has_texture(&self, tex_type: TextureType) -> bool
    {
        match tex_type
        {
            TextureType::Base => self.texture_base.width() > 0,
            TextureType::AmbientEmissive => self.texture_ambient.width() > 0,
            TextureType::Specular => self.texture_specular.width() > 0,
            TextureType::Normal => self.texture_normal.width() > 0,
            TextureType::Alpha => self.texture_alpha.width() > 0,
            TextureType::Roughness => self.texture_roughness.width() > 0,
            TextureType::AmbientOcclusion => self.texture_ambient_occlusion.width() > 0,
            TextureType::Reflectivity => self.texture_reflectivity.width() > 0
        }
    }

    pub fn texture_dimension(&self, tex_type: TextureType) -> (u32, u32)
    {
        match tex_type
        {
            TextureType::Base => self.texture_base.dimensions(),
            TextureType::AmbientEmissive => self.texture_ambient.dimensions(),
            TextureType::Specular => self.texture_specular.dimensions(),
            TextureType::Normal => self.texture_normal.dimensions(),
            TextureType::Alpha => self.texture_alpha.dimensions(),
            TextureType::Roughness => self.texture_roughness.dimensions(),
            TextureType::AmbientOcclusion => self.texture_ambient_occlusion.dimensions(),
            TextureType::Reflectivity => self.texture_reflectivity.dimensions()
        }
    }

    pub fn get_texture_pixel(&self, x: u32, y: u32, tex_type: TextureType) -> Vector4<f32>
    {
        if !self.has_texture(tex_type)
        {
            return Vector4::<f32>::new(0.0, 0.0, 0.0, 1.0);
        }

        let pixel;

        match tex_type
        {
            TextureType::Base => { pixel = self.texture_base.get_pixel(x, y); },
            TextureType::AmbientEmissive => { pixel = self.texture_ambient.get_pixel(x, y); },
            TextureType::Specular => { pixel = self.texture_specular.get_pixel(x, y); },
            TextureType::Normal => { pixel = self.texture_normal.get_pixel(x, y); },
            TextureType::Alpha => { pixel = self.texture_alpha.get_pixel(x, y); },
            TextureType::Roughness => { pixel = self.texture_roughness.get_pixel(x, y); },
            TextureType::AmbientOcclusion => { pixel = self.texture_ambient_occlusion.get_pixel(x, y); },
            TextureType::Reflectivity => { pixel = self.texture_reflectivity.get_pixel(x, y); }
        }

        let rgba = pixel.to_rgba();

        Vector4::<f32>::new
        (
            (rgba[0] as f32) / 255.0,
            (rgba[1] as f32) / 255.0,
            (rgba[2] as f32) / 255.0,
            (rgba[3] as f32) / 255.0
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TextureType
{
    Base,
    AmbientEmissive,
    Specular,
    Normal,
    Alpha,
    Roughness,
    AmbientOcclusion,
    Reflectivity,
}

pub struct AnimationData
{
    pub trans_initial: Matrix4<f32>,
}

impl AnimationData
{
    pub fn new() -> AnimationData
    {
        AnimationData
        {
            trans_initial: Matrix4::<f32>::identity()
        }
    }
}

pub struct ShapeBasics
{
    pub id: u32,
    pub name: String,
    pub visible: bool,
    pub trans: Matrix4<f32>,
    tran_inverse: Matrix4<f32>,

    pub b_box: Aabb,

    pub material: Material,

    pub animation_data: AnimationData,

    pub node_index: usize,
}

impl ShapeBasics
{
    pub fn new(name: &str) -> ShapeBasics
    {
        ShapeBasics
        {
            id: 0,
            name: name.to_string(),
            visible: true,
            trans: Matrix4::<f32>::identity(),
            tran_inverse: Matrix4::<f32>::identity(),
            b_box: Aabb::new_invalid(),
            material: Material::new(),
            animation_data: AnimationData::new(),

            node_index: 0
        }
    }

    pub fn get_mat(&mut self) -> Matrix4<f32>
    {
        self.trans
    }

    pub fn get_transformation(trans: &Matrix4<f32>, translation: Vector3<f32>, scale: Vector3<f32>, rotation: Vector3<f32>) -> Matrix4<f32>
    {
        let mut trans = trans.clone();

        let translation = nalgebra::Isometry3::translation(translation.x, translation.y, translation.z).to_homogeneous();
        let scale = Matrix4::new_nonuniform_scaling(&scale);

        //use a different rotation matrix for each axis to get the desired result
        //https://www.reddit.com/r/rust/comments/heuc9k/nalgebras_awkward_euler_angles_method/
        //https://github.com/dimforge/nalgebra/issues/269
        let rotation_x  = Rotation3::from_euler_angles(rotation.x, 0.0, 0.0).to_homogeneous();
        let rotation_y  = Rotation3::from_euler_angles(0.0, rotation.y, 0.0).to_homogeneous();
        let rotation_z  = Rotation3::from_euler_angles(0.0, 0.0, rotation.z).to_homogeneous();

        //let rotation  = Rotation3::new(rotation).to_homogeneous();

        trans = trans * translation;
        trans = trans * scale;
        //trans = trans * rotation;
        trans = trans * rotation_z;
        trans = trans * rotation_y;
        trans = trans * rotation_x;

        trans
    }

    pub fn apply_transformation(&mut self, translation: Vector3<f32>, scale: Vector3<f32>, rotation: Vector3<f32>)
    {
        self.trans = ShapeBasics::get_transformation(&self.trans, translation, scale, rotation);

        self.calc_inverse();
    }

    pub fn apply_translation(&mut self, translation: Vector3<f32>)
    {
        let scale = Vector3::<f32>::new(1.0, 1.0, 1.0);
        let rotation = Vector3::<f32>::new(0.0, 0.0, 0.0);

        self.trans = ShapeBasics::get_transformation(&self.trans, translation, scale, rotation);

        self.calc_inverse();
    }

    pub fn apply_mat(&mut self, trans: &Matrix4<f32>)
    {
        self.trans = trans.clone();

        self.calc_inverse();
    }

    pub fn get_inverse_ray(&self, ray: &Ray) -> Ray
    {
        let ray_inverse_start = self.tran_inverse * ray.origin.to_homogeneous();
        let ray_inverse_dir = self.tran_inverse * ray.dir.to_homogeneous();

        Ray::new(Point3::from_homogeneous(ray_inverse_start).unwrap(), Vector3::from_homogeneous(ray_inverse_dir).unwrap())
    }

    pub fn calc_inverse(&mut self)
    {
        //because we are dealing with 4x4 matrices: unwrap should be fine
        self.tran_inverse = self.trans.try_inverse().unwrap();
    }

    pub fn init_animation_data(&mut self)
    {
        self.animation_data.trans_initial = self.trans;
    }
}