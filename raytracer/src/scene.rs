use nalgebra::{Point2, Point3, Vector3};
use serde_json::{Value};

use easy_gltf::Light::{Directional, Point, Spot};

use image::{ImageBuffer, RgbaImage};

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

                    // ***** other settings
                    let mut visible = true;
                    if !&object["visible"].is_null() { visible = object["visible"].as_bool().unwrap(); }

                    // ***** transformation
                    let mut rotation = Vector3::<f32>::new(0.0, 0.0, 0.0);
                    let mut scale = Vector3::<f32>::new(1.0, 1.0, 1.0);
                    let mut translation = Vector3::<f32>::new(0.0, 0.0, 0.0);

                    if !object["transformation"].is_null()
                    {
                        scale = self.get_vec_from_json_object("scale", &object["transformation"], scale);
                        translation = self.get_vec_from_json_object("translation", &object["transformation"], translation);
                        rotation = self.get_vec_from_json_object("rotation", &object["transformation"], rotation);

                        //to rad
                        rotation.x = rotation.x.to_radians();
                        rotation.y = rotation.y.to_radians();
                        rotation.z = rotation.z.to_radians();
                    }

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
                                    item.get_basic_mut().visible = visible;
                                    item.get_basic_mut().apply_transformation(translation, scale, rotation);
                                }
                            }
                        }
                    }

                    // ***** appy material and properties
                    if let Some(mut shape) = shape
                    {
                        shape.get_basic_mut().material = material;
                        shape.get_basic_mut().visible = visible;
                        shape.get_basic_mut().apply_transformation(translation, scale, rotation);

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

    pub fn load_gltf(&mut self, path: &str) -> Vec<u32>
    {
        let mut loaded_ids: Vec<u32> = vec![];

        let scenes = easy_gltf::load(path).unwrap();
        for scene in scenes
        {
            // ********** light **********
            for light in scene.lights
            {
                match light
                {
                    Point { position, color, intensity } =>
                    {
                        self.lights.push(Box::new(Light
                        {
                            pos: Point3::<f32>::new(position.x, position.y, position.z),
                            dir: Vector3::<f32>::new(0.0, 0.0, 0.0),
                            color: Vector3::<f32>::new(color.x, color.y, color.z),
                            intensity: intensity,
                            max_angle: 0.0,
                            light_type: LightType::Point
                        }));
                    },
                    Directional { direction, color, intensity } =>
                    {
                        self.lights.push(Box::new(Light
                        {
                            pos: Point3::<f32>::new(0.0, 0.0, 0.0),
                            dir: Vector3::<f32>::new(direction.x, direction.y, direction.z),
                            color: Vector3::<f32>::new(color.x, color.y, color.z),
                            intensity: intensity,
                            max_angle: 0.0,
                            light_type: LightType::Directional
                        }));
                    },
                    Spot { position, direction, color, intensity, inner_cone_angle, outer_cone_angle } =>
                    {
                        self.lights.push(Box::new(Light
                        {
                            pos: Point3::<f32>::new(position.x, position.y, position.z),
                            dir: Vector3::<f32>::new(direction.x, direction.y, direction.z),
                            color: Vector3::<f32>::new(color.x, color.y, color.z),
                            intensity: intensity,
                            max_angle: outer_cone_angle,
                            light_type: LightType::Spot
                        }));
                    }
                };
            }

            // ********** camera **********
            if scene.cameras.len() > 2
            {
                println!("only one camera is supported");
            }

            if scene.cameras.len() > 0
            {
                let cam = &scene.cameras[0];
                dbg!(cam);
            }

            // ********** objects **********
            for model in scene.models
            {
                let triangles = model.triangles().unwrap();
                let material = model.material();

                let mut verts: Vec<Point3::<f32>> = vec![];
                let mut uvs: Vec<Point2<f32>> = vec![];
                let mut normals: Vec<Point3<f32>> = vec![];

                let mut indices:Vec<[u32; 3]> = vec![];
                let mut uv_indices: Vec<[u32; 3]> = vec![];
                let mut normals_indices: Vec<[u32; 3]> = vec![];

                let mut index_vert: u32 = 0;
                let mut index_uv: u32 = 0;
                let mut index_normal: u32 = 0;

                for triangle in triangles
                {
                    // ***** data
                    for vertex in triangle
                    {
                        // vertex
                        verts.push(Point3::<f32>::new(vertex.position.x, vertex.position.y, vertex.position.z));

                        // normal
                        if model.has_normals()
                        {
                            normals.push(Point3::<f32>::new(vertex.normal.x, vertex.normal.y, vertex.normal.z));
                        }

                        // texture uv coord
                        uvs.push(Point2::<f32>::new(vertex.tex_coords.x, vertex.tex_coords.y));
                    }

                    // ***** indices
                    indices.push([index_vert, index_vert + 1, index_vert + 2]);
                    index_vert += 3;

                    // normals
                    if model.has_normals()
                    {
                        normals_indices.push([index_normal, index_normal + 1, index_normal + 2]);
                        index_normal += 3;
                    }

                    // texture coords
                    if model.has_tex_coords()
                    {
                        uv_indices.push([index_uv, index_uv + 1, index_uv + 2]);
                        index_uv += 3;
                    }
                }

                let mut item = Mesh::new_with_data("unknown", verts, indices, uvs, uv_indices, normals, normals_indices);

                // ********** material **********
                if let Some(normal_map) = material.normal
                {
                    let tex = normal_map.texture;
                    item.basic.load_texture_buffer(normal_map.texture.as_ref() as &ImageBuffer<image::Rgba<u8>, Vec<u8>>, TextureType::Normal);
                }

                item.get_basic_mut().id = self.get_next_id();
                loaded_ids.push(item.get_basic().id);

                self.items.push(Box::new(item));

                /*
                let vertices = model. vertices();
                let indices = model.indices();
                let indices = model.indices();
                 */
            }

            /*
            println!
            (
                "Cameras: #{}  Lights: #{}  Models: #{}",
                scene.cameras.len(),
                scene.lights.len(),
                scene.models.len()
            )
             */
        }

        loaded_ids
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
            println!(" - {}: {} (visible: {})", item.get_basic().id, item.get_name(), item.get_basic().visible);
        }
    }
}