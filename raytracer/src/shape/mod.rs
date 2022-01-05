use nalgebra::Isometry3;
use nalgebra::Matrix4;

use parry3d::bounding_volume::AABB;

pub mod sphere;

pub trait Shape
{
    fn calc_bbox(&mut self);
}

pub struct ShapeBasics
{
    trans: Isometry3<f32>,
    tran_inverse: Matrix4<f32>,

    b_box: AABB
}

impl ShapeBasics
{
    pub fn new() -> ShapeBasics
    {
        ShapeBasics
        {
            trans: Isometry3::<f32>::identity(),
            tran_inverse: Matrix4::<f32>::identity(),
            b_box: AABB::new_invalid()
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