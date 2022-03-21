use nalgebra::{Vector3, Point3, Point2, Isometry3};

use parry3d::query::{Ray, RayCast};
use parry3d::shape::{TriMesh, FeatureId};

use crate::shape::{Shape, ShapeBasics, Material, TextureType};

pub struct Mesh
{
    pub basic: ShapeBasics,
    pub name: String,

    pub mesh: TriMesh,

    pub uvs: Vec<Point2<f32>>,
    pub uv_indices: Vec<[u32; 3]>,

    pub normals: Vec<Point3<f32>>,
    pub normals_indices: Vec<[u32; 3]>,
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

    fn get_basic_mut(&mut self) -> &mut ShapeBasics
    {
        &mut self.basic
    }

    fn calc_bbox(&mut self)
    {
        let trans = Isometry3::<f32>::identity();
        self.basic.b_box = self.mesh.aabb(&trans);
    }

    fn intersect_b_box(&self, ray: &Ray) -> Option<f32>
    {
        let ray_inverse = self.basic.get_inverse_ray(ray);

        //let solid = !(self.basic.material.alpha < 1.0 || self.basic.material.has_texture(TextureType::Alpha));
        let solid = false;

        self.basic.b_box.cast_local_ray(&ray_inverse, std::f32::MAX, solid)
    }

    fn intersect(&self, ray: &Ray) -> Option<(f32, Vector3<f32>, u32)>
    {
        let ray_inverse = self.basic.get_inverse_ray(ray);

        //let solid = !(self.basic.material.alpha < 1.0 || self.basic.material.has_texture(TextureType::Alpha));
        let solid = false;
        let res = self.mesh.cast_local_ray_and_get_normal(&ray_inverse, std::f32::MAX, solid);
        if let Some(res) = res
        {
            let mut face_id = 0;
            if let FeatureId::Face(i) = res.feature
            {
                face_id = i;
            }

            let mut normal = (self.basic.trans * res.normal.to_homogeneous()).xyz().normalize();

            //use normal based on loaded normal (not on computed normal by parry -- if needed for smooth shading)
            if self.get_material().smooth_shading && self.normals.len() > 0 && self.normals_indices.len() > 0
            {
                let hit = ray.origin + (ray.dir * res.toi);
                normal = self.get_normal(hit, face_id);
                normal = (self.basic.trans * normal.to_homogeneous()).xyz().normalize();

                if self.mesh.is_backface(res.feature)
                {
                    normal = -normal;
                }
            }
            return Some((res.toi, normal, face_id))
        }
        None
    }

    fn get_uv(&self, hit: Point3<f32>, face_id: u32) -> Point2<f32>
    {
        // https://stackoverflow.com/questions/23980748/triangle-texture-mapping-with-barycentric-coordinates
        // https://answers.unity.com/questions/383804/calculate-uv-coordinates-of-3d-point-on-plane-of-m.html

        //transform hit to local coords
        let hit_pos_local = self.basic.tran_inverse * hit.to_homogeneous();
        let hit_pos_local = Point3::<f32>::from_homogeneous(hit_pos_local).unwrap();

        let f_id = (face_id % self.mesh.indices().len() as u32) as usize;

        let face = self.mesh.indices()[f_id];
        let uv_face = self.uv_indices[f_id];

        let i0 = face[0] as usize;
        let i1 = face[1] as usize;
        let i2 = face[2] as usize;

        let i_uv_0 = uv_face[0] as usize;
        let i_uv_1 = uv_face[1] as usize;
        let i_uv_2 = uv_face[2] as usize;

        let a = self.mesh.vertices()[i0].to_homogeneous();
        let b = self.mesh.vertices()[i1].to_homogeneous();
        let c = self.mesh.vertices()[i2].to_homogeneous();

        let a_t = self.uvs[i_uv_0];
        let b_t = self.uvs[i_uv_1];
        let c_t = self.uvs[i_uv_2];

        let a = Point3::<f32>::from_homogeneous(a).unwrap();
        let b = Point3::<f32>::from_homogeneous(b).unwrap();
        let c = Point3::<f32>::from_homogeneous(c).unwrap();

        let f1 = a - hit_pos_local;
        let f2 = b - hit_pos_local;
        let f3 = c - hit_pos_local;

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
            uv_indices: vec![],
            normals: vec![],
            normals_indices: vec![]
        };

        mesh.calc_bbox();

        mesh
    }

    pub fn new_with_data(name: &str, vertices: Vec<Point3<f32>>, indices: Vec<[u32; 3]>, uvs: Vec<Point2<f32>>, uv_indices: Vec<[u32; 3]>, normals: Vec<Point3<f32>>, normals_indices: Vec<[u32; 3]>) -> Mesh
    {
        let mut mesh = Mesh
        {
            basic: ShapeBasics::new(),
            name: String::from(name),
            mesh: TriMesh::new(vertices, indices),
            uvs: uvs,
            uv_indices: uv_indices,
            normals: normals,
            normals_indices: normals_indices
        };

        mesh.calc_bbox();

        //mesh.mesh.recover_topology();
        //mesh.mesh.compute_pseudo_normals();

        mesh
    }

    pub fn new_plane(name: &str, x0: Point3<f32>, x1: Point3<f32>, x2: Point3<f32>, x3: Point3<f32>) -> Mesh
    {
        let points = vec![ x0, x1, x2, x3, ];

        let uvs = vec!
        [
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(0.0, 1.0),
        ];

        let indices = vec![[0u32, 1, 2], [0, 2, 3]];
        let uv_indices = vec![[0u32, 1, 2], [0, 2, 3]];

        Mesh::new_with_data(name, points, indices, uvs, uv_indices, vec![], vec![])
    }

    fn get_normal(&self, hit: Point3<f32>, face_id: u32) -> Vector3<f32>
    {
        // https://stackoverflow.com/questions/23980748/triangle-texture-mapping-with-barycentric-coordinates
        // https://answers.unity.com/questions/383804/calculate-uv-coordinates-of-3d-point-on-plane-of-m.html

        //transform hit to local coords
        let hit_pos_local = self.basic.tran_inverse * hit.to_homogeneous();
        let hit_pos_local = Point3::<f32>::from_homogeneous(hit_pos_local).unwrap();

        let f_id = (face_id % self.mesh.indices().len() as u32) as usize;

        let face = self.mesh.indices()[f_id];
        let normal_face = self.normals_indices[f_id];

        let i0 = face[0] as usize;
        let i1 = face[1] as usize;
        let i2 = face[2] as usize;

        let i_normal_0 = normal_face[0] as usize;
        let i_normal_1 = normal_face[1] as usize;
        let i_normal_2 = normal_face[2] as usize;

        let a = self.mesh.vertices()[i0].to_homogeneous();
        let b = self.mesh.vertices()[i1].to_homogeneous();
        let c = self.mesh.vertices()[i2].to_homogeneous();

        let a_t = self.normals[i_normal_0];
        let b_t = self.normals[i_normal_1];
        let c_t = self.normals[i_normal_2];

        let a = Point3::<f32>::from_homogeneous(a).unwrap();
        let b = Point3::<f32>::from_homogeneous(b).unwrap();
        let c = Point3::<f32>::from_homogeneous(c).unwrap();

        let f1 = a - hit_pos_local;
        let f2 = b - hit_pos_local;
        let f3 = c - hit_pos_local;

        let a = (a-b).cross(&(a-c)).magnitude();
        let a1 = f2.cross(&f3).magnitude() / a;
        let a2 = f3.cross(&f1).magnitude() / a;
        let a3 = f1.cross(&f2).magnitude() / a;

        let part_1 = a_t * a1;
        let part_2 = b_t * a2;
        let part_3 = c_t * a3;

        let normal = Point3::<f32>::new
        (
            part_1.x + part_2.x + part_3.x,
            part_1.y + part_2.y + part_3.y,
            part_1.z + part_2.z + part_3.z,
        );

        Vector3::<f32>::new(normal.x, normal.y, normal.z)
    }
}