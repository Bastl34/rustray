use nalgebra::{Vector3, Point3, Isometry3};

use parry3d::query::{Ray, RayCast};
use parry3d::shape::TriMesh;

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
        //self.basic.b_box.cast_ray(&self.basic.trans, ray, std::f32::MAX, true)
        let trans = Isometry3::<f32>::identity();
        self.basic.b_box.cast_ray(&trans, ray, std::f32::MAX, false)
    }

    fn intersect(&self, ray: &Ray) -> Option<(f32, Vector3<f32>)>
    {
        let res = self.mesh.cast_ray_and_get_normal(&self.basic.trans, ray, std::f32::MAX, false);
        if let Some(res) = res
        {
            return Some((res.toi, res.normal))
        }
        None
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