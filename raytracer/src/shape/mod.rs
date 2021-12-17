//use crate::shape::bbox::BoundingBox;
//use self::bbox::BoundingBox;

mod bounding_box;

use nalgebra::Matrix4;

use bounding_box::BoundingBox;

pub trait Shape
{
    fn name(&self) -> String
    {
        "no name".to_string()
    }
}

pub struct ShapeBasics
{
    trans: Matrix4<f32>,
    tran_inverse: Matrix4<f32>,

    b_box: BoundingBox
}

impl ShapeBasics
{
    pub fn calc_inverse(&mut self)
    {
        //because we are dealing with 4x4 matrices: unwrap should be fine
        self.tran_inverse = self.trans.try_inverse().unwrap();
    }
}