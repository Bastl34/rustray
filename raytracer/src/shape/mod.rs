use nalgebra::{Isometry3, Matrix4, Vector4, Vector3};
use parry3d::query::{Ray};

use parry3d::bounding_volume::AABB;

pub mod sphere;
pub mod mesh;

pub trait Shape
{
    fn get_material(&self) -> &Material;
    fn calc_bbox(&mut self);
    fn intersect_b_box(&self, ray: &Ray) -> Option<f32>;
    fn intersect(&self, ray: &Ray) -> Option<(f32, Vector3<f32>)>;

    fn update(&mut self)
    {
        self.calc_bbox();
    }
}

pub struct Material
{
    pub anmbient_color: Vector3<f32>,
    pub diffuse_color: Vector3<f32>,
    pub specular_color: Vector3<f32>,
    pub alpha: f32,
    pub shininess: f32,
    pub reflectivity: f32,
    pub refraction_index: f32,
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
        }
    }
}

pub struct ShapeBasics
{
    trans: Isometry3<f32>,
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

    pub fn calc_inverse(&mut self)
    {
        //because we are dealing with 4x4 matrices: unwrap should be fine
        self.tran_inverse = self.trans.to_homogeneous().try_inverse().unwrap();
    }
}