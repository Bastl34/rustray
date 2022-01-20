use nalgebra::{Point2, Point3, Vector3, Isometry3};

use crate::shape::Shape;

use crate::shape::sphere::Sphere;
use crate::shape::mesh::Mesh;

use std::path::Path;

#[derive(PartialEq)]
pub enum LightType
{
    Directional,
    Point
}

pub struct Light
{
    pub pos: Point3<f32>,
    pub dir: Vector3<f32>,
    pub color: Vector3<f32>,
    pub intensity: f32,
    pub light_type: LightType
}

pub struct Scene
{
    pub items: Vec<Box<dyn Shape + Send + Sync>>,
    pub lights: Vec<Box<Light>>,
}

impl Scene
{
    pub fn new() -> Scene
    {
        Scene
        {
            items: vec![],
            lights: vec![]
        }
    }

    pub fn init_with_some_objects(&mut self)
    {
        self.init_objects();
        self.init_lights();
    }

    pub fn init_lights(&mut self)
    {
        /*
        self.lights.push(Box::new(Light
        {
            pos: Point3::new(0.0, 0.0, 0.0),
            dir: Vector3::new(1.0f32, -1.0, -1.0),
            color: Vector3::new(1.0, 1.0, 1.0),
            intensity: 1.0,
            light_type: LightType::Directional
        }));
         */

        self.lights.push(Box::new(Light
        {
            pos: Point3::new(-5.0, -5.0, -5.0),
            dir: Vector3::new(1.0f32, 1.0, -1.0),
            color: Vector3::new(1.0, 1.0, 1.0),
            intensity: 150.0,
            light_type: LightType::Point
        }));

        self.lights.push(Box::new(Light
        {
            pos: Point3::new(-2.0, -2.0, -15.0),
            dir: Vector3::new(1.0f32, 1.0, -1.0),
            color: Vector3::new(1.0, 1.0, 1.0),
            intensity: 150.0,
            light_type: LightType::Point
        }));

        self.lights.push(Box::new(Light
        {
            pos: Point3::new(5.0, 5.0, -10.0),
            dir: Vector3::new(-1.0f32, -1.0, -1.0),
            color: Vector3::new(1.0, 0.0, 0.0),
            intensity: 150.0,
            light_type: LightType::Point
        }));
    }

    pub fn init_objects(&mut self)
    {
        // ******************** some spheres ********************
        let mut sphere_back = Box::new(Sphere::new_with_pos("sphere_back", 1.0, 0.0, -10.0, 1.0));
        sphere_back.basic.material.anmbient_color.x = 1.0;
        sphere_back.basic.material.anmbient_color.y = 0.0;
        sphere_back.basic.material.anmbient_color.z = 0.0;
        sphere_back.basic.material.reflectivity = 0.3;
        sphere_back.basic.material.alpha = 0.1;
        sphere_back.basic.material.refraction_index = 1.5;

        let mut sphere_front = Box::new(Sphere::new_with_pos("sphere_front", 0.0, 0.0, -5.0, 3.0));
        sphere_front.basic.material.anmbient_color.x = 1.0;
        sphere_front.basic.material.anmbient_color.y = 1.0;
        sphere_front.basic.material.anmbient_color.z = 1.0;
        sphere_front.basic.material.reflectivity = 0.3;
        sphere_front.basic.material.alpha = 0.1;
        sphere_front.basic.material.refraction_index = 1.5;

        let mut sphere_left = Box::new(Sphere::new_with_pos("sphere_left", -7.0, 0.0, -20.0, 4.0));
        sphere_left.basic.material.anmbient_color.x = 0.0;
        sphere_left.basic.material.anmbient_color.y = 1.0;
        sphere_left.basic.material.anmbient_color.z = 0.0;
        sphere_left.basic.material.reflectivity = 0.5;
        sphere_left.basic.material.alpha = 0.8;
        sphere_left.basic.material.refraction_index = 1.5;

        let mut sphere_right = Box::new(Sphere::new_with_pos("sphere_right", 7.0, -2.5, -18.0, 3.0));
        sphere_right.basic.material.anmbient_color.x = 0.0;
        sphere_right.basic.material.anmbient_color.y = 0.0;
        sphere_right.basic.material.anmbient_color.z = 1.0;
        sphere_right.basic.material.reflectivity = 0.5;
        sphere_right.basic.material.alpha = 0.8;
        sphere_right.basic.material.refraction_index = 1.5;

        let mut sphere_mirror = Box::new(Sphere::new_with_pos("sphere_mirror", -6.0, 2.5, -7.0, 1.0));
        sphere_mirror.basic.material.anmbient_color.x = 1.0;
        sphere_mirror.basic.material.anmbient_color.y = 1.0;
        sphere_mirror.basic.material.anmbient_color.z = 1.0;
        sphere_mirror.basic.material.reflectivity = 1.0;
        sphere_mirror.basic.material.alpha = 1.0;
        sphere_mirror.basic.material.refraction_index = 1.5;

        let mut sphere_texture = Box::new(Sphere::new_with_pos("sphere_texture", 6.0, -1.0, -5.0, 1.0));
        sphere_texture.basic.material.anmbient_color.x = 1.0;
        sphere_texture.basic.material.anmbient_color.y = 1.0;
        sphere_texture.basic.material.anmbient_color.z = 1.0;
        sphere_texture.basic.material.reflectivity = 0.3;
        sphere_texture.basic.material.alpha = 0.7;
        sphere_mirror.basic.material.refraction_index = 1.5;
        sphere_texture.basic.load_texture("scene/2k_earth_daymap.jpg");

        let mut sphere_not_visible = Box::new(Sphere::new_with_pos("sphere_not_visible", 7.0, 0.0, 10.0, 3.0));
        sphere_not_visible.basic.material.anmbient_color.x = 1.0;
        sphere_not_visible.basic.material.anmbient_color.y = 1.0;
        sphere_not_visible.basic.material.anmbient_color.z = 1.0;
        sphere_not_visible.basic.material.reflectivity = 0.5;

        // ******************** some meshes ********************
        //floor
        let mut mesh_floor = Box::new(Mesh::new_plane
        (
            "mesh_floor",
            Point3::new(-10.0, -5.5, 2.0),
            Point3::new(10.0, -5.5, 2.0),
            Point3::new(10.0, -5.5, -20.0),
            Point3::new(-10.0, -5.5, -20.0),
        ));

        mesh_floor.basic.material.anmbient_color.x = 0.5;
        mesh_floor.basic.material.anmbient_color.y = 0.5;
        mesh_floor.basic.material.anmbient_color.z = 1.0;
        mesh_floor.basic.material.reflectivity = 0.4;
        mesh_floor.basic.load_texture("scene/checkerboard.png");

        //back
        let mut mesh_back = Box::new(Mesh::new_plane
        (
            "mesh_back",
            Point3::new(-10.0, -5.5, -20.0),
            Point3::new(10.0, -5.5, -20.0),
            Point3::new(10.0, 5.5, -20.0),
            Point3::new(-10.0, 5.5, -20.0),
        ));

        mesh_back.basic.material.anmbient_color.x = 0.5;
        mesh_back.basic.material.anmbient_color.y = 0.5;
        mesh_back.basic.material.anmbient_color.z = 1.0;
        mesh_back.basic.material.reflectivity = 0.4;

        //left
        let mut mesh_left = Box::new(Mesh::new_plane
        (
            "mesh_left",
            Point3::new(-10.0, -5.5, 2.0),
            Point3::new(-10.0, -5.5, -20.0),
            Point3::new(-10.0, 5.5, -20.0),
            Point3::new(-10.0, 5.5, 2.0),
        ));

        mesh_left.basic.material.anmbient_color.x = 1.0;
        mesh_left.basic.material.anmbient_color.y = 0.0;
        mesh_left.basic.material.anmbient_color.z = 0.0;
        mesh_left.basic.material.reflectivity = 0.4;

        //right
        let mut mesh_right = Box::new(Mesh::new_plane
        (
            "mesh_right",
            Point3::new(10.0, -5.5, 2.0),
            Point3::new(10.0, -5.5, -20.0),
            Point3::new(10.0, 5.5, -20.0),
            Point3::new(10.0, 5.5, 2.0),
        ));

        mesh_right.basic.material.anmbient_color.x = 0.0;
        mesh_right.basic.material.anmbient_color.y = 1.0;
        mesh_right.basic.material.anmbient_color.z = 0.0;
        mesh_right.basic.material.reflectivity = 0.4;

        //top
        let mut mesh_top = Box::new(Mesh::new_plane
        (
            "mesh_top",
            Point3::new(-10.0, 5.5, 2.0),
            Point3::new(10.0, 5.5, 2.0),
            Point3::new(10.0, 5.5, -20.0),
            Point3::new(-10.0, 5.5, -20.0),
        ));

        mesh_top.basic.material.anmbient_color.x = 0.5;
        mesh_top.basic.material.anmbient_color.y = 0.5;
        mesh_top.basic.material.anmbient_color.z = 1.0;
        mesh_top.basic.material.reflectivity = 0.4;

        //behind
        let mut mesh_behind = Box::new(Mesh::new_plane
        (
            "mesh_behind",
            Point3::new(-10.0, -5.5, 2.0),
            Point3::new(10.0, -5.5, 2.0),
            Point3::new(10.0, 5.5, 2.0),
            Point3::new(-10.0, 5.5, 2.0),
        ));

        mesh_behind.basic.material.anmbient_color.x = 1.0;
        mesh_behind.basic.material.anmbient_color.y = 0.5;
        mesh_behind.basic.material.anmbient_color.z = 0.5;
        mesh_behind.basic.material.reflectivity = 0.4;

        let mut mesh_front = Box::new(Mesh::new_plane
        (
            "mesh_front",
            Point3::new(-5.0, -2.5, -10.0),
            Point3::new(5.0, -2.5, -10.0),
            Point3::new(5.0, 2.5, -10.0),
            Point3::new(-5.0, 2.5, -10.0),
        ));

        mesh_front.basic.material.anmbient_color.x = 1.0;
        mesh_front.basic.material.anmbient_color.y = 1.0;
        mesh_front.basic.material.anmbient_color.z = 1.0;
        mesh_front.basic.material.reflectivity = 0.3;
        mesh_front.basic.load_texture("scene/2k_earth_daymap.jpg");

/*
        self.items.push(sphere_back);
        self.items.push(sphere_front);
        self.items.push(sphere_left);
        self.items.push(sphere_right);
        self.items.push(sphere_not_visible);
        self.items.push(sphere_mirror);
        self.items.push(sphere_texture);
*/

        self.items.push(mesh_floor);
        self.items.push(mesh_back);
        self.items.push(mesh_left);
        self.items.push(mesh_right);
        self.items.push(mesh_top);
        self.items.push(mesh_behind);


        //self.items.push(mesh_front);

        self.load("scene/kBert_thumbsup_bevel.obj");

        //let mut k_bert = self.get_by_name("kBert_Cube").unwrap();
        //k_bert.borrow_mut().
        //k_bert.borrow_mut().get_material().reflectivity = 0.5;
    }

    pub fn load(&mut self, path: &str)
    {
        let options = &tobj::LoadOptions
        {
            triangulate: true,
            ..Default::default()
        };

        let (models, materials) = tobj::load_obj(&path, options).unwrap();
        let materials = materials.unwrap();

        for (_i, m) in models.iter().enumerate()
        {
            let mesh = &m.mesh;

            if mesh.texcoord_indices.len() > 0 && mesh.indices.len() != mesh.texcoord_indices.len()
            {
                println!("Error can not load {}, because of indices mismatch", m.name.as_str());
                continue;
            }

            let mut verts: Vec<Point3::<f32>> = vec![];
            let mut uvs: Vec<Point2<f32>> = vec![];

            let mut indices:Vec<[u32; 3]> = vec![];
            let mut uv_indices: Vec<[u32; 3]> = vec![];

            //vertices
            for vtx in 0..mesh.positions.len() / 3
            {
                let x = mesh.positions[3 * vtx];
                let y = mesh.positions[3 * vtx + 1];
                let z = mesh.positions[3 * vtx + 2];

                verts.push(Point3::<f32>::new(x, y, z));
            }

            //tex coords
            for vtx in 0..mesh.texcoords.len() / 2
            {
                let x = mesh.texcoords[2 * vtx];
                let y = mesh.texcoords[2 * vtx + 1];

                uvs.push(Point2::<f32>::new(x, y));
            }

            //indices
            for vtx in 0..mesh.indices.len() / 3
            {
                let i0 = mesh.indices[3 * vtx];
                let i1 = mesh.indices[3 * vtx + 1];
                let i2 = mesh.indices[3 * vtx + 2];

                indices.push([i0, i1, i2]);
            }            

            //tex coords indices
            for vtx in 0..mesh.texcoord_indices.len() / 3
            {
                let i0 = mesh.texcoord_indices[3 * vtx];
                let i1 = mesh.texcoord_indices[3 * vtx + 1];
                let i2 = mesh.texcoord_indices[3 * vtx + 2];

                uv_indices.push([i0, i1, i2]);
            }            
            

            if verts.len() > 0
            {
                let mut item = Mesh::new_with_data(m.name.as_str(), verts, indices, uvs, uv_indices);

                //apply material
                if let Some(mat_id) = mesh.material_id
                {
                    let mat: &tobj::Material = &materials[mat_id];

                    //item.basic.material.shininess = mat.shininess;
                    item.basic.material.anmbient_color = Vector3::<f32>::new(mat.ambient[0], mat.ambient[1], mat.ambient[2]);
                    item.basic.material.specular_color = Vector3::<f32>::new(mat.specular[0], mat.specular[1], mat.specular[2]);
                    item.basic.material.diffuse_color = Vector3::<f32>::new(mat.diffuse[0], mat.diffuse[1], mat.diffuse[2]);
                    item.basic.material.refraction_index = mat.optical_density;
                    item.basic.material.alpha = mat.dissolve;

                    //TODO:
                    item.basic.material.reflectivity = 0.5;
                    item.basic.material.alpha = 0.5;

                    if let Some(illumination) = mat.illumination_model
                    {
                        if illumination > 2
                        {
                            item.basic.material.reflectivity = 0.5;
                        }
                    }

                    //texture
                    if !mat.diffuse_texture.is_empty()
                    {
                        let mut tex_path = mat.diffuse_texture.clone();

                        if Path::new(&tex_path).is_relative()
                        {
                            let parent = Path::new(path).parent();
                            if let Some(parent) = parent
                            {
                                tex_path = parent.join(tex_path).to_str().unwrap().to_string();
                            }
                        }
                        item.basic.load_texture(&tex_path);
                    }
                }

                item.basic.material.print();

                self.items.push(Box::new(item));
            }

        }
    }

    pub fn print(&self)
    {
        println!("scene:");
        for item in &self.items
        {
            println!(" - {}", item.get_name());
        }
    }
}