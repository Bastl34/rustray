use nalgebra::{Vector3, Point3, Point2, Isometry3};

use parry3d::query::{Ray, RayCast};
use parry3d::shape::{TriMesh, FeatureId};

use crate::shape::{Shape, ShapeBasics, Material};

pub struct Mesh
{
    pub basic: ShapeBasics,
    name: String,

    mesh: TriMesh,

    uvs: Vec<Point2<f32>>,
    uv_indices: Vec<[u32; 3]>
}

impl Shape for Mesh
{
    fn get_name(&self) -> &String
    {
        &self.name
    }

    fn get_material(&self) -> &Material
    {
        &self.basic.material
    }

    fn get_basic(&self) -> &ShapeBasics
    {
        &self.basic
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

    fn intersect(&self, ray: &Ray) -> Option<(f32, Vector3<f32>, u32)>
    {
        let solid = !(self.basic.material.alpha < 1.0);
        let res = self.mesh.cast_ray_and_get_normal(&self.basic.trans, ray, std::f32::MAX, solid);
        if let Some(res) = res
        {
            let mut face_id = 0;
            //println!("{:?}", res.feature);
            if let FeatureId::Face(i) = res.feature
            {
                //println!("{}", i);
                face_id = i;
            }

            return Some((res.toi, res.normal, face_id))
        }
        None
    }

    fn get_uv(&self, hit: Point3<f32>, face_id: u32) -> Point2<f32>
    {
        // https://stackoverflow.com/questions/23980748/triangle-texture-mapping-with-barycentric-coordinates
        // https://answers.unity.com/questions/383804/calculate-uv-coordinates-of-3d-point-on-plane-of-m.html

        let f_id = (face_id % self.mesh.indices().len() as u32) as usize;

        let face = self.mesh.indices()[f_id];
        let uv_face = self.uv_indices[f_id];

        let i0 = face[0] as usize;
        let i1 = face[1] as usize;
        let i2 = face[2] as usize;

        let i_uv_0 = uv_face[0] as usize;
        let i_uv_1 = uv_face[1] as usize;
        let i_uv_2 = uv_face[2] as usize;

        let mut a = self.mesh.vertices()[i0];
        let mut b = self.mesh.vertices()[i1];
        let mut c = self.mesh.vertices()[i2];

        let a_t = self.uvs[i_uv_0];
        let b_t = self.uvs[i_uv_1];
        let c_t = self.uvs[i_uv_2];

        //apply transformation
        a = &self.basic.trans * a;
        b = &self.basic.trans * b;
        c = &self.basic.trans * c;

        let f1 = a - hit;
        let f2 = b - hit;
        let f3 = c - hit;

        let a = (a-b).cross(&(a-c)).magnitude();
        let a1 = f2.cross(&f3).magnitude() / a;
        let a2 = f3.cross(&f1).magnitude() / a;
        let a3 = f1.cross(&f2).magnitude() / a;

        let part_1 = a_t * a1;
        let part_2 = b_t * a2;
        let part_3 = c_t * a3;

        let uv = Point2::<f32>::new(part_1.x + part_2.x + part_3.x, part_1.y + part_2.y + part_3.y);

        Point2::<f32>::new(uv.x, -uv.y)
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
            mesh: TriMesh::new(vec![], vec![]),
            uvs: vec![],
            uv_indices: vec![]
        };

        mesh.calc_bbox();

        mesh
    }

    pub fn new_with_data(name: &str, vertices: Vec<Point3<f32>>, indices: Vec<[u32; 3]>, uvs: Vec<Point2<f32>>, uv_indices: Vec<[u32; 3]>) -> Mesh
    {
        let mut mesh = Mesh
        {
            basic: ShapeBasics::new(),
            name: String::from(name),
            mesh: TriMesh::new(vertices, indices),
            uvs: uvs,
            uv_indices: uv_indices
        };

        mesh.calc_bbox();

        mesh
    }

    pub fn new_plane(name: &str, x0: Point3<f32>, x1: Point3<f32>, x2: Point3<f32>, x3: Point3<f32>) -> Mesh
    {
        let points_front = vec![ x0, x1, x2, x3, ];

        let uvs_front = vec!
        [
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(0.0, 1.0),
        ];

        let indices_front = vec![[0u32, 1, 2], [0, 2, 3]];
        let uv_indices_front = vec![[0u32, 1, 2], [0, 2, 3]];

        Mesh::new_with_data(name, points_front, indices_front, uvs_front, uv_indices_front)
    }

    pub fn load(path: &str)
    {
        let (models, materials) = tobj::load_obj(&path, &tobj::LoadOptions::default()).unwrap();
    }
}