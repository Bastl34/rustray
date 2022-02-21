use nalgebra::{Point2, Point3, Vector3};
use serde_json::{Value};

use crate::shape::{Shape, TextureType, Material};

use crate::shape::sphere::Sphere;
use crate::shape::mesh::Mesh;

use std::path::Path;

#[derive(PartialEq)]
pub enum LightType
{
    Directional,
    Point,
    Spot
}

pub struct Light
{
    pub pos: Point3<f32>,
    pub dir: Vector3<f32>,
    pub color: Vector3<f32>,
    pub intensity: f32,
    pub max_angle: f32, //in rad
    pub light_type: LightType
}

pub type ScemeItem = Box<dyn Shape + Send + Sync>;

pub struct Scene
{
    pub item_id: u32,
    pub items: Vec<ScemeItem>,
    pub lights: Vec<Box<Light>>,
}

impl Scene
{
    pub fn new() -> Scene
    {
        Scene
        {
            item_id: 0,
            items: vec![],
            lights: vec![]
        }
    }

    pub fn clear(&mut self)
    {
        self.item_id = 0;
        self.items.clear();
        self.lights.clear();
    }

    pub fn init_with_some_objects(&mut self)
    {
        self.init_objects();

        self.update();
    }

    pub fn get_next_id(&mut self) -> u32
    {
        self.item_id = self.item_id + 1;

        self.item_id
    }

    pub fn load_json(&mut self, path: &str)
    {
        let data = std::fs::read_to_string(path);
        if data.is_ok()
        {
            let str = data.unwrap();
            let scene_data = serde_json::from_str::<Value>(&str);
            if scene_data.is_ok()
            {
                let data = scene_data.unwrap();

                let lights = data["lights"].as_array().unwrap();
                let objects = data["objects"].as_array().unwrap();

                // ********** lights **********
                for light in lights
                {
                    // pos
                    let pos = self.get_point_from_json_object("pos", &light, Point3::<f32>::new(0.0, 0.0, 0.0));

                    // dir
                    let dir = self.get_vec_from_json_object("dir", &light, Vector3::<f32>::new(0.0, 0.0, 0.0));

                    // color
                    let color = self.get_color_from_json_object("color", &light, Vector3::<f32>::new(0.0, 0.0, 0.0));

                    // intensity
                    let intensity =  light["intensity"].as_f64().unwrap() as f32;

                    // max_angle
                    let mut max_angle = 0.0f32;
                    if !light["max_angle"].is_null()
                    {
                        max_angle =  (light["max_angle"].as_f64().unwrap() as f32).to_radians();
                    }

                    // light type
                    let mut light_type = LightType::Point;
                    let light_type_str = light["light_type"].as_str().unwrap();

                    match light_type_str
                    {
                        "point" => { light_type = LightType::Point },
                        "directional" => { light_type = LightType::Directional },
                        "spot" => { light_type = LightType::Spot },
                        _ => {}
                    }

                    self.lights.push(Box::new(Light
                    {
                        pos: pos,
                        dir: dir,
                        color: color,
                        intensity: intensity,
                        max_angle: max_angle,
                        light_type: light_type
                    }));
                }

                // ********** objects **********
                for object in objects
                {
                    let mut shape: Option<ScemeItem> = None;
                    let mut material = Material::new();

                    //type
                    let item_type = object["type"].as_str().unwrap();

                    // name
                    let mut name = "unknown";
                    if !object["name"].is_null()
                    {
                        name = object["name"].as_str().unwrap();
                    }

                    // ***** colors
                    let colors = &object["color"];

                    if !colors.is_null()
                    {
                        // base color
                        material.base_color = self.get_color_from_json_object("base", &colors, material.base_color);

                        // specular color
                        material.specular_color = self.get_color_from_json_object("specular", &colors, material.specular_color);
                        if colors["specular"]["factor"].is_f64()
                        {
                            material.specular_color = material.base_color * colors["specular"]["factor"].as_f64().unwrap() as f32;
                        }

                        // ambient color
                        material.ambient_color = self.get_color_from_json_object("ambient", &colors, material.ambient_color);
                        if colors["ambient"]["factor"].is_f64()
                        {
                            material.ambient_color = material.base_color * colors["ambient"]["factor"].as_f64().unwrap() as f32;
                        }
                    }

                    // ***** material settings
                    if !&object["alpha"].is_null() { material.alpha = object["alpha"].as_f64().unwrap() as f32; }
                    if !&object["shininess"].is_null() { material.shininess = object["shininess"].as_f64().unwrap() as f32; }
                    if !&object["reflectivity"].is_null() { material.reflectivity = object["reflectivity"].as_f64().unwrap() as f32; }
                    if !&object["refraction_index"].is_null() { material.refraction_index = object["refraction_index"].as_f64().unwrap() as f32; }
                    if !&object["normal_map_strength"].is_null() { material.normal_map_strength = object["normal_map_strength"].as_f64().unwrap() as f32; }
                    if !&object["cast_shadow"].is_null() { material.cast_shadow = object["cast_shadow"].as_bool().unwrap(); }
                    if !&object["receive_shadow"].is_null() { material.receive_shadow = object["receive_shadow"].as_bool().unwrap(); }
                    if !&object["shadow_softness"].is_null() { material.shadow_softness = object["shadow_softness"].as_f64().unwrap() as f32; }
                    if !&object["surface_roughness"].is_null() { material.surface_roughness = object["surface_roughness"].as_f64().unwrap() as f32; }
                    if !&object["smooth_shading"].is_null() { material.smooth_shading = object["smooth_shading"].as_bool().unwrap(); }

                    // ***** sphere
                    if item_type == "sphere"
                    {
                        let pos = self.get_point_from_json_object("pos", &object, Point3::<f32>::new(0.0, 0.0, 0.0));

                        let mut radius = 0.0f32;
                        if !object["radius"].is_null()
                        {
                            radius = object["radius"].as_f64().unwrap() as f32;
                        }

                        // create shape
                        shape = Some(Box::new(Sphere::new_with_pos(name, pos.x, pos.y, pos.z, radius)));
                    }
                    // ***** plane
                    else if item_type == "plane"
                    {
                        let p0 = &object["vertices"].as_array().unwrap()[0];
                        let p1 = &object["vertices"].as_array().unwrap()[1];
                        let p2 = &object["vertices"].as_array().unwrap()[2];
                        let p3 = &object["vertices"].as_array().unwrap()[3];

                        shape = Some(Box::new(Mesh::new_plane
                        (
                            name,
                            Point3::<f32>::new(p0["x"].as_f64().unwrap() as f32, p0["y"].as_f64().unwrap() as f32, p0["z"].as_f64().unwrap() as f32),
                            Point3::<f32>::new(p1["x"].as_f64().unwrap() as f32, p1["y"].as_f64().unwrap() as f32, p1["z"].as_f64().unwrap() as f32),
                            Point3::<f32>::new(p2["x"].as_f64().unwrap() as f32, p2["y"].as_f64().unwrap() as f32, p2["z"].as_f64().unwrap() as f32),
                            Point3::<f32>::new(p3["x"].as_f64().unwrap() as f32, p3["y"].as_f64().unwrap() as f32, p3["z"].as_f64().unwrap() as f32),
                        )));
                    }
                    else if item_type == "wavefront"
                    {
                        let path = object["path"].as_str().unwrap();
                        let loaded_ids = self.load_wavefront(path);

                        //apply material diffs
                        for item in & mut self.items
                        {
                            for item_id in &loaded_ids
                            {
                                if *item_id == item.get_basic().id
                                {
                                    item.get_basic_mut().material.apply_diff(&material);
                                }
                            }
                        }
                    }

                    // ***** appy material
                    if let Some(mut shape) = shape
                    {
                        shape.get_basic_mut().material = material;

                        // ***** textures
                        let texture = &object["texture"];

                        if !texture.is_null()
                        {
                            // base
                            if texture["base"].is_string()
                            {
                                shape.get_basic_mut().load_texture(texture["base"].as_str().unwrap(), TextureType::Base);
                            }

                            // ambient
                            if texture["ambient"].is_string()
                            {
                                shape.get_basic_mut().load_texture(texture["ambient"].as_str().unwrap(), TextureType::Ambient);
                            }

                            // specular
                            if texture["specular"].is_string()
                            {
                                shape.get_basic_mut().load_texture(texture["specular"].as_str().unwrap(), TextureType::Specular);
                            }

                            // normal
                            if texture["normal"].is_string()
                            {
                                shape.get_basic_mut().load_texture(texture["normal"].as_str().unwrap(), TextureType::Normal);
                            }

                            // alpha
                            if texture["alpha"].is_string()
                            {
                                shape.get_basic_mut().load_texture(texture["alpha"].as_str().unwrap(), TextureType::Alpha);
                            }
                        }

                        shape.get_basic_mut().id = self.get_next_id();

                        self.items.push(shape);
                    }
                }
            }
            else
            {
                println!("error can not parse json file {}", path);
            }
        }
        else
        {
            println!("error can not load file {}", path);
        }

        self.update();
    }

    pub fn get_color_from_json_object(&self, key: &str, json_obj: &Value, default_data: Vector3::<f32>) -> Vector3::<f32>
    {
        let mut vec = default_data;

        if json_obj.is_null()
        {
            return vec;
        }

        if !json_obj[key].is_null() && !json_obj[key]["r"].is_null() && !json_obj[key]["g"].is_null() && !json_obj[key]["b"].is_null()
        {
            vec.x = json_obj[key]["r"].as_f64().unwrap() as f32;
            vec.y = json_obj[key]["g"].as_f64().unwrap() as f32;
            vec.z = json_obj[key]["b"].as_f64().unwrap() as f32;
        }

        vec
    }

    pub fn get_point_from_json_object(&self, key: &str, json_obj: &Value, default_data: Point3::<f32>) -> Point3::<f32>
    {
        let mut p = default_data;

        if json_obj.is_null()
        {
            return p;
        }

        if !json_obj[key].is_null() && !json_obj[key]["x"].is_null() && !json_obj[key]["y"].is_null() && !json_obj[key]["z"].is_null()
        {
            p.x = json_obj[key]["x"].as_f64().unwrap() as f32;
            p.y = json_obj[key]["y"].as_f64().unwrap() as f32;
            p.z = json_obj[key]["z"].as_f64().unwrap() as f32;
        }

        p
    }

    pub fn get_vec_from_json_object(&self, key: &str, json_obj: &Value, default_data: Vector3::<f32>) -> Vector3::<f32>
    {
        let mut v = default_data;

        if json_obj.is_null()
        {
            return v;
        }

        if !json_obj[key].is_null() && !json_obj[key]["x"].is_null() && !json_obj[key]["y"].is_null() && !json_obj[key]["z"].is_null()
        {
            v.x = json_obj[key]["x"].as_f64().unwrap() as f32;
            v.y = json_obj[key]["y"].as_f64().unwrap() as f32;
            v.z = json_obj[key]["z"].as_f64().unwrap() as f32;
        }

        v
    }

    pub fn init_objects(&mut self)
    {
        // ******************** some spheres ********************
        let mut sphere_back = Box::new(Sphere::new_with_pos("sphere_back", 1.0, 0.0, -10.0, 1.0));
        sphere_back.basic.material.base_color = Vector3::<f32>::new(1.0, 0.0, 0.0);
        sphere_back.basic.material.specular_color = sphere_back.basic.material.base_color * 0.8;
        sphere_back.basic.material.reflectivity = 0.3;
        sphere_back.basic.material.alpha = 0.1;
        sphere_back.basic.material.refraction_index = 1.5;

        let mut sphere_front = Box::new(Sphere::new_with_pos("sphere_front", 0.0, 0.0, -5.0, 3.0));
        sphere_front.basic.material.base_color = Vector3::<f32>::new(1.0, 1.0, 1.0);
        sphere_front.basic.material.specular_color = sphere_front.basic.material.base_color * 0.8;
        sphere_front.basic.material.reflectivity = 0.3;
        sphere_front.basic.material.alpha = 0.1;
        sphere_front.basic.material.refraction_index = 1.5;

        let mut sphere_left = Box::new(Sphere::new_with_pos("sphere_left", -7.0, 0.0, -20.0, 4.0));
        sphere_left.basic.material.base_color = Vector3::<f32>::new(0.0, 1.0, 0.0);
        sphere_left.basic.material.specular_color = sphere_left.basic.material.base_color * 0.8;
        sphere_left.basic.material.reflectivity = 0.5;
        sphere_left.basic.material.alpha = 0.8;
        sphere_left.basic.material.refraction_index = 1.5;

        let mut sphere_right = Box::new(Sphere::new_with_pos("sphere_right", 7.0, -2.5, -18.0, 3.0));
        sphere_right.basic.material.base_color = Vector3::<f32>::new(0.0, 0.0, 1.0);
        sphere_right.basic.material.specular_color = sphere_right.basic.material.base_color * 0.8;
        sphere_right.basic.material.reflectivity = 0.5;
        sphere_right.basic.material.alpha = 0.8;
        sphere_right.basic.material.refraction_index = 1.5;

        let mut sphere_mirror = Box::new(Sphere::new_with_pos("sphere_mirror", -6.0, 2.5, -7.0, 1.0));
        sphere_mirror.basic.material.base_color = Vector3::<f32>::new(1.0, 1.0, 1.0);
        sphere_mirror.basic.material.specular_color = sphere_mirror.basic.material.base_color * 0.8;
        sphere_mirror.basic.material.reflectivity = 1.0;
        sphere_mirror.basic.material.alpha = 1.0;
        sphere_mirror.basic.material.refraction_index = 1.5;

        //let mut sphere_texture = Box::new(Sphere::new_with_pos("sphere_texture", 6.0, -1.0, -5.0, 1.0));
        //let mut sphere_texture = Box::new(Sphere::new_with_pos("sphere_texture", 0.0, -1.0, -7.0, 4.0));
        let mut sphere_texture = Box::new(Sphere::new_with_pos("sphere_texture", 0.0, -1.0, -10.0, 3.0));
        sphere_texture.basic.material.base_color = Vector3::<f32>::new(1.0, 1.0, 1.0);
        sphere_texture.basic.material.specular_color = sphere_texture.basic.material.base_color * 0.8;
        //sphere_texture.basic.material.reflectivity = 0.7;
        sphere_texture.basic.material.reflectivity = 0.5;
        sphere_texture.basic.material.alpha = 0.9;
        sphere_texture.basic.material.refraction_index = 1.0;
        sphere_texture.basic.material.normal_map_strength = 10.0;
        sphere_texture.basic.material.surface_roughness = 0.05;
        //sphere_texture.basic.load_texture("scene/checkerboard.png", TextureType::Base);
        //sphere_texture.basic.load_texture("scene/earth/2k_earth_daymap.jpg", TextureType::Base);
        sphere_texture.basic.load_texture("scene/earth/2k_earth_normal_map.jpg", TextureType::Normal);
        //sphere_texture.basic.load_texture("scene/white.png", TextureType::Normal);
        //sphere_texture.basic.load_texture("scene/checkerboard.png", TextureType::Normal);
        //sphere_texture.basic.load_texture("scene/leather/Leather_Weave_006_basecolor.jpg", TextureType::Base);
        //sphere_texture.basic.load_texture("scene/leather/Leather_Weave_006_opacity.jpg", TextureType::Alpha);

        let mut sphere_not_visible = Box::new(Sphere::new_with_pos("sphere_not_visible", 7.0, 0.0, 10.0, 3.0));
        sphere_not_visible.basic.material.base_color = Vector3::<f32>::new(1.0, 1.0, 1.0);

        let mut sphere_far_away = Box::new(Sphere::new_with_pos("sphere_front", 0.0, 0.0, -50.0, 30.0));
        sphere_far_away.basic.material.base_color = Vector3::<f32>::new(1.0, 1.0, 1.0);
        sphere_far_away.basic.material.specular_color = sphere_front.basic.material.base_color * 0.8;
        sphere_far_away.basic.material.reflectivity = 0.3;
        sphere_far_away.basic.material.alpha = 1.0;

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

        mesh_floor.basic.material.base_color = Vector3::<f32>::new(0.5, 0.5, 1.0);
        mesh_floor.basic.material.specular_color = mesh_floor.basic.material.base_color * 0.8;

        mesh_floor.basic.material.reflectivity = 0.4;
        //mesh_floor.basic.material.surface_roughness = 0.005;
        //mesh_floor.basic.material.shadow_softness = 0.1;
        mesh_floor.basic.load_texture("scene/checkerboard.png", TextureType::Base);

        //back
        let mut mesh_back = Box::new(Mesh::new_plane
        (
            "mesh_back",
            Point3::new(-10.0, -5.5, -20.0),
            Point3::new(10.0, -5.5, -20.0),
            Point3::new(10.0, 5.5, -20.0),
            Point3::new(-10.0, 5.5, -20.0),
        ));

        let uvs = vec!
        [
            Point2::new(0.0, 0.0),
            Point2::new(3.0, 0.0),
            Point2::new(3.0, 3.0),
            Point2::new(0.0, 3.0),
        ];

        mesh_back.uvs = uvs.clone();

        mesh_back.basic.material.base_color = Vector3::<f32>::new(0.5, 0.5, 1.0);
        mesh_back.basic.material.specular_color = mesh_back.basic.material.base_color * 0.8;

        mesh_back.basic.material.reflectivity = 0.4;

        mesh_back.basic.load_texture("scene/floor/base.gif", TextureType::Base);
        //mesh_back.basic.load_texture("scene/floor/bump.gif", TextureType::Normal);
        //mesh_back.basic.load_texture("scene/floor/specular.gif", TextureType::Specular);

        //left
        let mut mesh_left = Box::new(Mesh::new_plane
        (
            "mesh_left",
            Point3::new(-10.0, -5.5, 2.0),
            Point3::new(-10.0, -5.5, -20.0),
            Point3::new(-10.0, 5.5, -20.0),
            Point3::new(-10.0, 5.5, 2.0),
        ));

        mesh_left.uvs = uvs.clone();

        mesh_left.basic.material.base_color = Vector3::<f32>::new(1.0, 0.0, 0.0);
        mesh_left.basic.material.specular_color = mesh_left.basic.material.base_color * 0.8;

        mesh_left.basic.material.reflectivity = 0.4;
        //mesh_left.basic.load_texture("scene/wall/Wall_Stone_022_basecolor.jpg", TextureType::Base);
        //mesh_left.basic.load_texture("scene/wall/Wall_Stone_022_normal.jpg", TextureType::Normal);
        //mesh_left.basic.material.normal_map_strength = 10.0;
        mesh_left.basic.load_texture("scene/floor/base.gif", TextureType::Base);

        //right
        let mut mesh_right = Box::new(Mesh::new_plane
        (
            "mesh_right",
            Point3::new(10.0, -5.5, 2.0),
            Point3::new(10.0, -5.5, -20.0),
            Point3::new(10.0, 5.5, -20.0),
            Point3::new(10.0, 5.5, 2.0),
        ));

        mesh_right.uvs = uvs.clone();

        mesh_right.basic.material.base_color = Vector3::<f32>::new(0.0, 1.0, 0.0);
        mesh_right.basic.material.specular_color = mesh_right.basic.material.base_color * 0.8;
        mesh_right.basic.material.reflectivity = 0.4;
        mesh_right.basic.load_texture("scene/floor/base.gif", TextureType::Base);

        //top
        let mut mesh_top = Box::new(Mesh::new_plane
        (
            "mesh_top",
            Point3::new(-10.0, 5.5, 2.0),
            Point3::new(10.0, 5.5, 2.0),
            Point3::new(10.0, 5.5, -20.0),
            Point3::new(-10.0, 5.5, -20.0),
        ));

        mesh_top.uvs = uvs.clone();

        mesh_top.basic.material.base_color = Vector3::<f32>::new(0.5, 0.5, 1.0);
        mesh_top.basic.material.specular_color = mesh_top.basic.material.base_color * 0.8;
        mesh_top.basic.material.reflectivity = 0.4;
        mesh_top.basic.load_texture("scene/floor/base.gif", TextureType::Base);

        //behind
        let mut mesh_behind = Box::new(Mesh::new_plane
        (
            "mesh_behind",
            Point3::new(-10.0, -5.5, 2.0),
            Point3::new(10.0, -5.5, 2.0),
            Point3::new(10.0, 5.5, 2.0),
            Point3::new(-10.0, 5.5, 2.0),
        ));

        mesh_behind.basic.material.base_color = Vector3::<f32>::new(1.0, 0.5, 0.5);
        mesh_behind.basic.material.specular_color = mesh_behind.basic.material.base_color * 0.8;
        mesh_behind.basic.material.reflectivity = 0.4;

        let mut mesh_front = Box::new(Mesh::new_plane
        (
            "mesh_front",
            Point3::new(-5.0, -2.5, -10.0),
            Point3::new(5.0, -2.5, -10.0),
            Point3::new(5.0, 2.5, -10.0),
            Point3::new(-5.0, 2.5, -10.0),
        ));

        mesh_front.basic.material.base_color = Vector3::<f32>::new(1.0, 1.0, 1.0);
        mesh_front.basic.material.specular_color = mesh_front.basic.material.base_color * 0.8;

        mesh_front.basic.material.reflectivity = 0.3;


        /*
        self.items.push(sphere_back);
        self.items.push(sphere_front);
        self.items.push(sphere_left);
        self.items.push(sphere_right);
        self.items.push(sphere_not_visible);
        self.items.push(sphere_mirror);
        self.items.push(sphere_texture);
         */


        //self.items.push(sphere_far_away);


        //self.items.push(sphere_texture);


        //self.items.push(mesh_floor);
        self.items.push(mesh_back);
        self.items.push(mesh_left);
        self.items.push(mesh_right);
        self.items.push(mesh_top);
        self.items.push(mesh_behind);



        //self.items.push(mesh_front);

        /*
        self.load("scene/monkey.obj");
        self.get_by_name("Suzanne").unwrap().get_basic_mut().trans = nalgebra::Isometry3::translation(0.0, 0.0, -10.0).to_homogeneous();
        self.get_by_name("Suzanne").unwrap().get_basic_mut().material.reflectivity = 0.5;
        self.get_by_name("Suzanne").unwrap().get_basic_mut().material.smooth_shading = true;
        self.get_by_name("Suzanne").unwrap().get_basic_mut().material.alpha = 0.5;
         */


        /*
        self.load_wavefront("scene/kBert_thumbsup_bevel.obj");
        let items = ["kBert_Cube", "Cylinder_Cylinder.001"];

        for item in items
        {
            let item = self.get_by_name(item).unwrap();
            item.get_basic_mut().material.reflectivity = 0.1;
            //item.get_basic_mut().material.shadow_softness = 0.1;
            //item.get_basic_mut().material.alpha = 0.5;
        }

        self.get_by_name("Cylinder_Cylinder.001").unwrap().get_basic_mut().material.smooth_shading = false;
         */

    }

    pub fn load_wavefront(&mut self, path: &str) -> Vec<u32>
    {
        let mut loaded_ids: Vec<u32> = vec![];

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
            let mut normals: Vec<Point3<f32>> = vec![];

            let mut indices:Vec<[u32; 3]> = vec![];
            let mut uv_indices: Vec<[u32; 3]> = vec![];
            let mut normals_indices: Vec<[u32; 3]> = vec![];


            //vertices
            for vtx in 0..mesh.positions.len() / 3
            {
                let x = mesh.positions[3 * vtx];
                let y = mesh.positions[3 * vtx + 1];
                let z = mesh.positions[3 * vtx + 2];

                verts.push(Point3::<f32>::new(x, y, z));
            }

            //normals
            for vtx in 0..mesh.normals.len() / 3
            {
                let x = mesh.normals[3 * vtx];
                let y = mesh.normals[3 * vtx + 1];
                let z = mesh.normals[3 * vtx + 2];

                normals.push(Point3::<f32>::new(x, y, z));
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

            //normals coords indices
            for vtx in 0..mesh.normal_indices.len() / 3
            {
                let i0 = mesh.normal_indices[3 * vtx];
                let i1 = mesh.normal_indices[3 * vtx + 1];
                let i2 = mesh.normal_indices[3 * vtx + 2];

                normals_indices.push([i0, i1, i2]);
            }

            if verts.len() > 0
            {
                let mut item = Mesh::new_with_data(m.name.as_str(), verts, indices, uvs, uv_indices, normals, normals_indices);

                //apply material
                if let Some(mat_id) = mesh.material_id
                {
                    let mat: &tobj::Material = &materials[mat_id];

                    item.basic.material.shininess = mat.shininess;
                    item.basic.material.ambient_color = Vector3::<f32>::new(mat.ambient[0], mat.ambient[1], mat.ambient[2]);
                    item.basic.material.specular_color = Vector3::<f32>::new(mat.specular[0], mat.specular[1], mat.specular[2]);
                    item.basic.material.base_color = Vector3::<f32>::new(mat.diffuse[0], mat.diffuse[1], mat.diffuse[2]);
                    item.basic.material.refraction_index = mat.optical_density;
                    item.basic.material.alpha = mat.dissolve;

                    item.basic.material.ambient_color = item.basic.material.base_color * 0.01;

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
                        item.basic.load_texture(&tex_path, TextureType::Base);
                    }
                }

                item.get_basic_mut().id = self.get_next_id();
                loaded_ids.push(item.get_basic().id);

                self.items.push(Box::new(item));
            }

        }
        loaded_ids
    }

    pub fn update(&mut self)
    {
        for item in & mut self.items
        {
            item.update();
        }
    }

    pub fn get_by_name(&mut self, name: &str) -> Option<&mut ScemeItem>
    {
        for item in & mut self.items
        {
            if item.get_name() == name
            {
                return Some(item);
            }
        }

        None
    }

    pub fn print(&self)
    {
        println!("scene:");
        for item in &self.items
        {
            println!(" - {}: {}", item.get_basic().id, item.get_name());
        }
    }
}