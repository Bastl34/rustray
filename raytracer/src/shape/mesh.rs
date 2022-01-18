use nalgebra::{Vector3, Point3, Point2, Isometry3};

use parry3d::query::{Ray, RayCast};
use parry3d::shape::{TriMesh, FeatureId};

use crate::shape::{Shape, ShapeBasics, Material};

pub struct Mesh
{
    pub basic: ShapeBasics,
    name: String,

    mesh: TriMesh
}

impl Shape for Mesh
{
    fn get_material(&self) -> &Material
    {
        &self.basic.material
    }

    fn calc_bbox(&mut self)
    {
        self.basic.b_box = self.mesh.aabb(&self.basic.trans);
    }

    fn intersect_b_box(&self, ray: &Ray) -> Option<f32>
    {
        let trans = Isometry3::<f32>::identity();
        let solid = !(self.basic.material.alpha < 1.0);
        self.basic.b_box.cast_ray(&trans, ray, std::f32::MAX, solid)
    }

    fn intersect(&self, ray: &Ray) -> Option<(f32, Vector3<f32>)>
    {
        let solid = !(self.basic.material.alpha < 1.0);
        let res = self.mesh.cast_ray_and_get_normal(&self.basic.trans, ray, std::f32::MAX, solid);
        if let Some(res) = res
        {
            println!("{:?}", res.feature);
            if let FeatureId::Face(i) = res.feature
            {
                //println!("{}", i);
            }

            return Some((res.toi, res.normal))
        }
        None
    }

    fn get_uv(&self, hit: Vector3<f32>, face_id: u32) -> Point2<f32>
    {
        //TODO
        Point2::<f32>::new(0.0, 0.0)
    }
}

impl Mesh
{
    pub fn new() -> Mesh
    {
        let mut mesh = Mesh
        {
            basic: ShapeBasics::new(),
            name: String::from("Mesh"),
            mesh: TriMesh::new(vec![], vec![])
        };

        mesh.calc_bbox();

        mesh
    }

    pub fn new_with_data(vertices: Vec<Point3<f32>>, indices: Vec<[u32; 3]>) -> Mesh
    {
        let mut mesh = Mesh
        {
            basic: ShapeBasics::new(),
            name: String::from("Mesh"),
            mesh: TriMesh::new(vertices, indices)
        };

        mesh.calc_bbox();

        mesh
    }
}