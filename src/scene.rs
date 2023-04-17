use bvh::bvh::BVHNode;
use easy_gltf::Projection;
use nalgebra::{Point2, Point3, Vector3};
use parry3d::query::Ray;
use serde_json::{Value};

use easy_gltf::Light::{Directional, Point, Spot};

use image::{DynamicImage, Rgba, RgbaImage, ImageBuffer};

use crate::helper::download;
use crate::post_processing::PostProcessingConfig;
use crate::raytracing::RaytracingConfig;
use crate::shape::{Shape, TextureType, Material, ShapeBasics, MaterialItem};

use crate::shape::sphere::Sphere;
use crate::shape::mesh::Mesh;
use crate::camera::{Camera, OBLIQUE_CAM_POS, DEFAULT_FOV};
use crate::animation::{Animation, Frame, Keyframe};

use std::f32::consts::PI;
use std::path::Path;
use std::sync::{Arc, RwLock};

pub type ScemeItem = Box<dyn Shape + Send + Sync>;
pub type LightItem = Box<Light>;

// ******************** LightType ********************

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum LightType
{
    Directional,
    Point,
    Spot
}

// ******************** Light ********************

pub struct Light
{
    pub enabled: bool,
    pub id: u32,
    pub pos: Point3<f32>,
    pub dir: Vector3<f32>,
    pub color: Vector3<f32>,
    pub intensity: f32,
    pub max_angle: f32, //in rad
    pub light_type: LightType
}

impl Light
{
    pub fn name(&self) -> String
    {
        match self.light_type
        {
            LightType::Point => return "Point".to_string(),
            LightType::Directional => return "Directional".to_string(),
            LightType::Spot => return "Spot".to_string()
        }
    }
}


// ******************** Scene ********************

pub struct Scene
{
    pub item_id: u32, //for scene and light items

    pub cam: Camera,
    pub items: Vec<ScemeItem>,
    pub lights: Vec<LightItem>,
    pub animation: Animation,
    pub materials: Vec<MaterialItem>,

    pub raytracing_config: RaytracingConfig,
    pub post_processing: PostProcessingConfig,

    bvh: bvh::bvh::BVH
}

impl Scene
{
    pub fn new() -> Scene
    {
        Scene
        {
            item_id: 0,

            cam: Camera::new(),
            items: vec![],
            lights: vec![],
            materials: vec![],
            animation: Animation::new(),

            raytracing_config: RaytracingConfig::new(),
            post_processing: PostProcessingConfig::new(),

            bvh: bvh::bvh::BVH { nodes: vec![] }
        }
    }

    pub fn clear(&mut self)
    {
        self.item_id = 0;
        self.items.clear();
        self.lights.clear();
        self.materials.clear();
    }

    pub fn get_next_id(&mut self) -> u32
    {
        self.item_id = self.item_id + 1;

        self.item_id
    }

    pub fn load(&mut self, path: &str) -> Vec<u32>
    {
        let mut loaded_ids: Vec<u32> = vec![];

        // ********** load based on extension **********
        let extension = Path::new(path).extension();

        if extension.is_none()
        {
            println!("can not load {}", path);
            return vec![];
        }
        let extension = extension.unwrap();

        if extension == "json"
        {
            loaded_ids = self.load_json(path);
        }
        else if extension == "gltf" || extension == "glb"
        {
            loaded_ids = self.load_gltf(path);
        }
        else if extension == "obj"
        {
            loaded_ids = self.load_wavefront(path);
        }
        else
        {
            println!("can not load {}", path);
        }

        // ********** update data and bvh **********
        self.init();
        self.update();

        loaded_ids
    }

    pub fn load_json(&mut self, path: &str) -> Vec<u32>
    {
        let mut loaded_ids: Vec<u32> = vec![];

        let data = std::fs::read_to_string(path);
        if data.is_ok()
        {
            let str = data.unwrap();
            let scene_data = serde_json::from_str::<Value>(&str);
            if scene_data.is_ok()
            {
                let data = scene_data.unwrap();

                let camera = &data["camera"];
                let lights = data["lights"].as_array();
                let objects = data["objects"].as_array();
                let animation = &data["animation"];
                let config = &data["config"];
                let post = &data["post"];

                // ********** config **********
                if !config.is_null()
                {
                    if !&config["monte_carlo"].is_null() { self.raytracing_config.monte_carlo = config["monte_carlo"].as_bool().unwrap(); }
                    if !&config["samples"].is_null() { self.raytracing_config.samples = config["samples"].as_u64().unwrap() as u16;}

                    if !&config["focal_length"].is_null() { self.raytracing_config.focal_length = config["focal_length"].as_f64().unwrap() as f32; }
                    if !&config["aperture_size"].is_null() { self.raytracing_config.aperture_size = config["aperture_size"].as_f64().unwrap() as f32; }

                    if !&config["fog_density"].is_null() { self.raytracing_config.fog_density = config["fog_density"].as_f64().unwrap() as f32; }
                    if !&config["fog_color"].is_null()
                    {
                        self.raytracing_config.fog_color.x = config["fog_color"]["r"].as_f64().unwrap() as f32;
                        self.raytracing_config.fog_color.y = config["fog_color"]["g"].as_f64().unwrap() as f32;
                        self.raytracing_config.fog_color.z = config["fog_color"]["b"].as_f64().unwrap() as f32;
                    }

                    if !&config["max_recursion"].is_null() { self.raytracing_config.max_recursion = config["max_recursion"].as_u64().unwrap() as u16; }
                    if !&config["gamma_correction"].is_null() { self.raytracing_config.gamma_correction = config["gamma_correction"].as_bool().unwrap(); }
                }

                // ********** post processing **********
                if !post.is_null()
                {
                    if !&post["cavity"].is_null() { self.post_processing.cavity = post["cavity"].as_bool().unwrap(); }
                    if !&post["outline"].is_null() { self.post_processing.outline = post["outline"].as_bool().unwrap(); }
                }

                // ********** camera **********
                if !camera.is_null()
                {
                    let default_cam = Camera::new();

                    let pos = self.get_point_from_json_object("pos", &camera, default_cam.eye_pos);
                    let up = self.get_vec_from_json_object("up", &camera, default_cam.up);
                    let dir = self.get_vec_from_json_object("dir", &camera, default_cam.dir);

                    self.cam.eye_pos = pos;
                    self.cam.dir = dir;
                    self.cam.up = up;

                    let fov =  camera["fov"].as_f64();
                    let z_near =  camera["z_near"].as_f64();
                    let z_far =  camera["z_far"].as_f64();

                    if let Some(fov) = fov
                    {
                        self.cam.fov = fov.to_radians() as f32;
                    }

                    if let Some(z_near) = z_near
                    {
                        self.cam.clipping_near = z_near as f32;
                    }

                    if let Some(z_far) = z_far
                    {
                        self.cam.clipping_far = z_far as f32;
                    }
                }

                // ********** lights **********
                if let Some(lights) = lights
                {
                    for light in lights
                    {
                        // pos
                        let pos = self.get_point_from_json_object("pos", &light, Point3::<f32>::new(0.0, 0.0, 0.0));

                        // dir
                        let dir = self.get_vec_from_json_object("dir", &light, Vector3::<f32>::new(0.0, -1.0, 0.0));

                        // color
                        let color = self.get_color_from_json_object("color", &light, Vector3::<f32>::new(0.0, 0.0, 0.0));

                        // intensity
                        let intensity =  light["intensity"].as_f64().unwrap() as f32;

                        // max_angle
                        let mut max_angle = PI / 2.0; // just a default value - if type is chaning
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

                        let id = self.get_next_id();
                        self.lights.push(Box::new(Light
                        {
                            enabled: true,
                            id: id,
                            pos: pos,
                            dir: dir,
                            color: color,
                            intensity: intensity,
                            max_angle: max_angle,
                            light_type: light_type
                        }));
                    }
                }

                // ********** objects **********
                if let Some(objects) = objects
                {
                    for object in objects
                    {
                        let mut shape: Option<ScemeItem> = None;

                        let id = self.get_next_id();
                        let mut material = Box::new(Material::new(id, "material"));

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
                        if !&object["roughness"].is_null() { material.roughness = object["roughness"].as_f64().unwrap() as f32; }
                        if !&object["monte_carlo"].is_null() { material.monte_carlo = object["monte_carlo"].as_bool().unwrap(); }
                        if !&object["smooth_shading"].is_null() { material.smooth_shading = object["smooth_shading"].as_bool().unwrap(); }
                        if !&object["reflection_only"].is_null() { material.reflection_only = object["reflection_only"].as_bool().unwrap(); }
                        if !&object["backface_cullig"].is_null() { material.backface_cullig = object["backface_cullig"].as_bool().unwrap(); }

                        // ***** textures
                        let texture = &object["texture"];

                        if !texture.is_null()
                        {
                            // base
                            if texture["base"].is_string()
                            {
                                material.load_texture(texture["base"].as_str().unwrap(), TextureType::Base);
                            }

                            // ambient
                            if texture["ambient"].is_string()
                            {
                                material.load_texture(texture["ambient"].as_str().unwrap(), TextureType::AmbientEmissive);
                            }

                            // specular
                            if texture["specular"].is_string()
                            {
                                material.load_texture(texture["specular"].as_str().unwrap(), TextureType::Specular);
                            }

                            // normal
                            if texture["normal"].is_string()
                            {
                                material.load_texture(texture["normal"].as_str().unwrap(), TextureType::Normal);
                            }

                            // alpha
                            if texture["alpha"].is_string()
                            {
                                material.load_texture(texture["alpha"].as_str().unwrap(), TextureType::Alpha);
                            }

                            // roughness
                            if texture["roughness"].is_string()
                            {
                                material.load_texture(texture["roughness"].as_str().unwrap(), TextureType::Roughness);
                            }

                            // ambient_occlusion
                            if texture["ambient_occlusion"].is_string()
                            {
                                material.load_texture(texture["ambient_occlusion"].as_str().unwrap(), TextureType::AmbientOcclusion);
                            }
                        }

                        // ***** other (basic item) settings
                        let default_mat = Arc::new(RwLock::new(Box::new(Material::new(0, "unknown"))));
                        let default_basic_settings = ShapeBasics::new("", default_mat);
                        let mut visible = default_basic_settings.visible;
                        let mut flip_normals = default_basic_settings.flip_normals;
                        if !&object["visible"].is_null() { visible = object["visible"].as_bool().unwrap(); }
                        if !&object["flip_normals"].is_null() { flip_normals = object["flip_normals"].as_bool().unwrap(); }

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

                        let material_arc = Arc::new(RwLock::new(material));

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
                            let mut sphere = Box::new(Sphere::new_with_pos(name, material_arc.clone(), pos.x, pos.y, pos.z, radius));

                            sphere.get_basic_mut().id = self.get_next_id();
                            loaded_ids.push(sphere.get_basic().id);

                            shape = Some(sphere);
                        }
                        // ***** plane
                        else if item_type == "plane"
                        {
                            let p0 = &object["vertices"].as_array().unwrap()[0];
                            let p1 = &object["vertices"].as_array().unwrap()[1];
                            let p2 = &object["vertices"].as_array().unwrap()[2];
                            let p3 = &object["vertices"].as_array().unwrap()[3];

                            let mut plane = Box::new(Mesh::new_plane
                            (
                                name,
                                material_arc.clone(),
                                Point3::<f32>::new(p0["x"].as_f64().unwrap() as f32, p0["y"].as_f64().unwrap() as f32, p0["z"].as_f64().unwrap() as f32),
                                Point3::<f32>::new(p1["x"].as_f64().unwrap() as f32, p1["y"].as_f64().unwrap() as f32, p1["z"].as_f64().unwrap() as f32),
                                Point3::<f32>::new(p2["x"].as_f64().unwrap() as f32, p2["y"].as_f64().unwrap() as f32, p2["z"].as_f64().unwrap() as f32),
                                Point3::<f32>::new(p3["x"].as_f64().unwrap() as f32, p3["y"].as_f64().unwrap() as f32, p3["z"].as_f64().unwrap() as f32),
                            ));

                            plane.get_basic_mut().id = self.get_next_id();
                            loaded_ids.push(plane.get_basic().id);

                            shape = Some(plane);
                        }
                        else if item_type == "wavefront" || item_type == "json" || item_type == "gltf"
                        {
                            let path = object["path"].as_str().unwrap();
                            let url = object["url"].as_str();

                            if let Some(url) = url
                            {
                                println!("downloading {} to {}", url, path);

                                if !Path::new(path).exists()
                                {
                                    let res = download(url, path);
                                    if res.is_ok()
                                    {
                                        println!("... download finished");
                                    }
                                    else
                                    {
                                        println!("... ERROR while downloading");
                                    }
                                }
                                else
                                {
                                    println!("... skipping download (file is already there)");
                                }
                            }

                            let mut ids = vec![];

                            if item_type == "wavefront"
                            {
                                ids = self.load_wavefront(path);
                            }
                            else if item_type == "json"
                            {
                                ids = self.load_json(path);
                            }
                            else if item_type == "gltf"
                            {
                                ids = self.load_gltf(path);
                            }

                            //apply material diffs
                            for item in & mut self.items
                            {
                                for item_id in &ids
                                {
                                    if *item_id == item.get_basic().id
                                    {
                                        if !object["name"].is_null()
                                        {
                                            item.get_basic_mut().name = name.to_string();
                                        }

                                        item.get_basic_mut().material.write().unwrap().apply_diff(&material_arc.read().unwrap());
                                        item.get_basic_mut().visible = visible;
                                        item.get_basic_mut().flip_normals = flip_normals;
                                        item.get_basic_mut().apply_transformation(translation, scale, rotation);
                                    }
                                }
                            }

                            loaded_ids.extend(ids);
                        }

                        // ***** appy material and properties
                        if let Some(mut shape) = shape
                        {
                            //shape.get_basic_mut().material = material;
                            shape.get_basic_mut().visible = visible;
                            shape.get_basic_mut().flip_normals = flip_normals;
                            shape.get_basic_mut().apply_transformation(translation, scale, rotation);

                            shape.get_basic_mut().id = self.get_next_id();

                            self.items.push(shape);
                            self.materials.push(material_arc.clone());
                        }
                    }
                }

                // ********** animation **********
                if !animation.is_null()
                {
                    let fps = animation["fps"].as_u64();
                    if let Some(fps) = fps
                    {
                        self.animation.fps = fps as u32;
                    }

                    let enabled = animation["enabled"].as_bool();
                    if let Some(enabled) = enabled
                    {
                        self.animation.enabled = enabled;
                    }

                    // ********** keyframes
                    let keyframes = animation["keyframes"].as_array();

                    let mut keyframes_data = vec![];

                    if let Some(keyframes) = keyframes
                    {
                        for keyframe in keyframes
                        {
                            let time = keyframe["time"].as_u64();

                            if time.is_none()
                            {
                                println!("error: kexframe has no timestamp");
                                continue;
                            }

                            let time = time.unwrap();

                            let objects = keyframe["objects"].as_array();

                            let mut keyframe_data = vec![];

                            // ********** keyframe data
                            if let Some(objects) = objects
                            {
                                for object in objects
                                {
                                    let object_name = object["name"].as_str().unwrap();

                                    let mut rotation = None;
                                    let mut scale = None;
                                    let mut translation = None;

                                    if !object["transformation"].is_null()
                                    {
                                        scale = self.get_vec_from_json_object_option("scale", &object["transformation"]);
                                        translation = self.get_vec_from_json_object_option("translation", &object["transformation"]);
                                        rotation = self.get_vec_from_json_object_option("rotation", &object["transformation"]);

                                        //to rad
                                        if let Some(r) = rotation
                                        {
                                            let mut rotation_rad = r;
                                            rotation_rad.x = r.x.to_radians();
                                            rotation_rad.y = r.y.to_radians();
                                            rotation_rad.z = r.z.to_radians();

                                            rotation = Some(rotation_rad);
                                        }
                                    }

                                    let frame = Frame::new(object_name.to_string(), translation, rotation, scale);
                                    keyframe_data.push(frame);
                                }
                            }

                            let keyframe = Keyframe::new(time, keyframe_data);
                            keyframes_data.push(keyframe);
                        }

                        // apply animation data
                        self.animation.keyframes = keyframes_data;
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

        loaded_ids
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

    pub fn get_vec_from_json_object_option(&self, key: &str, json_obj: &Value) -> Option<Vector3::<f32>>
    {
        if json_obj.is_null()
        {
            return None;
        }

        if !json_obj[key].is_null() && !json_obj[key]["x"].is_null() && !json_obj[key]["y"].is_null() && !json_obj[key]["z"].is_null()
        {
            let mut v = Vector3::<f32>::new(0.0, 0.0, 0.0);
            v.x = json_obj[key]["x"].as_f64().unwrap() as f32;
            v.y = json_obj[key]["y"].as_f64().unwrap() as f32;
            v.z = json_obj[key]["z"].as_f64().unwrap() as f32;

            return Some(v);
        }
        else
        {
            return None;
        }
    }

    pub fn load_gltf(&mut self, path: &str) -> Vec<u32>
    {
        let mut loaded_ids: Vec<u32> = vec![];

        let mut double_check_materials: Vec<(Arc<easy_gltf::Material>, u32)> = vec![];

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
                        let id = self.get_next_id();
                        self.lights.push(Box::new(Light
                        {
                            enabled: true,
                            id: id,
                            pos: Point3::<f32>::new(position.x, position.y, position.z),
                            dir: Vector3::<f32>::new(0.0, -1.0, 0.0),
                            color: Vector3::<f32>::new(color.x, color.y, color.z),
                            intensity: intensity / 10.0,
                            max_angle: PI / 2.0, // just some default value - if type is changing
                            light_type: LightType::Point
                        }));
                    },
                    Directional { direction, color, intensity } =>
                    {
                        let id = self.get_next_id();
                        self.lights.push(Box::new(Light
                        {
                            enabled: true,
                            id: id,
                            pos: Point3::<f32>::new(0.0, 0.0, 0.0),
                            dir: Vector3::<f32>::new(direction.x, direction.y, direction.z),
                            color: Vector3::<f32>::new(color.x, color.y, color.z),
                            intensity: intensity,
                            max_angle: PI / 2.0, // just some default value - if type is changing
                            light_type: LightType::Directional
                        }));
                    },
                    Spot { position, direction, color, intensity, inner_cone_angle, outer_cone_angle } =>
                    {
                        println!("TODO: use inner_cone_angle: {}", inner_cone_angle);

                        let id = self.get_next_id();
                        self.lights.push(Box::new(Light
                        {
                            enabled: true,
                            id: id,
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
                println!("only one camera is supported (using first one)");
            }

            if scene.cameras.len() > 0
            {
                let cam = &scene.cameras[0];

                let pos = cam.position();
                let up = cam.up();
                let forward = cam.forward();

                self.cam.eye_pos = Point3::<f32>::new(pos.x, pos.y, pos.z);
                self.cam.dir = Vector3::<f32>::new(-forward.x, -forward.y, -forward.z).normalize();
                self.cam.up = Vector3::<f32>::new(up.x, up.y, up.z).normalize();

                match cam.projection
                {
                    Projection::Perspective { yfov, .. } =>
                    {
                        self.cam.fov = yfov.0;
                    },
                    Projection::Orthographic { .. } =>
                    {
                        !todo!()
                    },
                }

                self.cam.clipping_near = cam.znear;
                self.cam.clipping_far = cam.zfar;
            }

            // ********** objects **********
            for model in scene.models
            {
                let triangles = model.triangles().unwrap();
                let gltf_material = model.material();

                let object_id = self.get_next_id();

                let mut verts: Vec<Point3::<f32>> = vec![];
                let mut uvs: Vec<Point2<f32>> = vec![];
                let mut normals: Vec<Point3<f32>> = vec![];

                let mut indices:Vec<[u32; 3]> = vec![];
                let mut uv_indices: Vec<[u32; 3]> = vec![];
                let mut normals_indices: Vec<[u32; 3]> = vec![];

                let mut index_vert: u32 = 0;
                let mut index_uv: u32 = 0;
                let mut index_normal: u32 = 0;

                let mut reusing_material_object_id = 0;
                for mat in &double_check_materials
                {
                    if Arc::ptr_eq(&mat.0, &gltf_material)
                    {
                        reusing_material_object_id = mat.1;
                        break;
                    }
                }

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
                        if model.has_tex_coords()
                        {
                            uvs.push(Point2::<f32>::new(vertex.tex_coords.x, -vertex.tex_coords.y));
                        }
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

                let material_arc;

                if reusing_material_object_id != 0
                {
                    material_arc = self.get_material_by_id(reusing_material_object_id).unwrap();
                }
                else
                {
                    let material_id = self.get_next_id();
                    material_arc = Arc::new(RwLock::new(Box::new(Material::new(material_id, "unknown"))));

                    // ********** material **********
                    let mut material = material_arc.write().unwrap();

                    let base_color = gltf_material.pbr.base_color_factor;
                    material.base_color = Vector3::<f32>::new(base_color.x, base_color.y, base_color.z);
                    material.specular_color = material.base_color * 0.8; // TODO 🤔

                    material.alpha = base_color.w;
                    material.reflectivity = gltf_material.pbr.metallic_factor * 0.5; // do not use full metallic_factor as reflectivity --> otherwise the object will be just complete mirror if metallic is set to 1.0
                    material.roughness = (1.0 / PI / 2.0) * gltf_material.pbr.roughness_factor;
                    //materialroughness = gltf_material.pbr.roughness_factor;
                    //material.reflectivity = gltf_material.pbr.metallic_factor * (1.0 - material.roughness);

                    // base map
                    if gltf_material.pbr.base_color_texture.is_some()
                    {
                        let img = self.get_dyn_image_from_gltf_material(&gltf_material, TextureType::Base);
                        material.load_texture_buffer(&img, TextureType::Base);
                    }

                    // normal map
                    if gltf_material.normal.is_some()
                    {
                        let img = self.get_dyn_image_from_gltf_material(&gltf_material, TextureType::Normal);
                        material.load_texture_buffer(&img, TextureType::Normal);
                    }

                    // metallic map
                    if gltf_material.pbr.metallic_texture.is_some()
                    {
                        let img = self.get_dyn_image_from_gltf_material(&gltf_material, TextureType::Reflectivity);
                        material.load_texture_buffer(&img, TextureType::Reflectivity);
                    }

                    // emissive
                    if gltf_material.emissive.texture.is_some()
                    {
                        let img = self.get_dyn_image_from_gltf_material(&gltf_material, TextureType::AmbientEmissive);
                        material.load_texture_buffer(&img, TextureType::AmbientEmissive);
                        material.ambient_color.x = gltf_material.emissive.factor.x;
                        material.ambient_color.y = gltf_material.emissive.factor.y;
                        material.ambient_color.z = gltf_material.emissive.factor.z;
                    }

                    // roughness map
                    if gltf_material.pbr.roughness_texture.is_some()
                    {
                        let img = self.get_dyn_image_from_gltf_material(&gltf_material, TextureType::Roughness);
                        material.load_texture_buffer(&img, TextureType::Roughness);
                    }

                    // occlusion map
                    if gltf_material.occlusion.is_some()
                    {
                        let img = self.get_dyn_image_from_gltf_material(&gltf_material, TextureType::AmbientOcclusion);
                        material.load_texture_buffer(&img, TextureType::AmbientOcclusion);
                    }

                    self.materials.push(material_arc.clone());
                    double_check_materials.push((gltf_material.clone(), material_id));
                }


                let mut item = Mesh::new_with_data("unknown", material_arc.clone(), verts, indices, uvs, uv_indices, normals, normals_indices);
                item.get_basic_mut().id = object_id;
                loaded_ids.push(item.get_basic().id);

                self.items.push(Box::new(item));
            }
        }

        loaded_ids
    }

    pub fn get_dyn_image_from_gltf_material(&self, mat: &easy_gltf::Material, tex_type: TextureType) -> DynamicImage
    {
        match tex_type
        {
            TextureType::Base =>
            {
                if let Some(base_map) = &mat.pbr.base_color_texture
                {
                    let width = base_map.width();
                    let height = base_map.height();

                    let mut img: RgbaImage = ImageBuffer::new(width, height);
                    for x in 0..width
                    {
                        for y in 0..height
                        {
                            let pixel = base_map.get_pixel(x, y);

                            img.put_pixel(x, y, Rgba([pixel[0], pixel[1], pixel[2], pixel[3]]));
                        }
                    }

                    return DynamicImage::ImageRgba8(img.clone());
                }
            },
            TextureType::Normal =>
            {
                if let Some(normal_map) = &mat.normal
                {
                    let tex = &normal_map.texture;
                    let width = tex.width();
                    let height = tex.height();

                    let mut img: RgbaImage = ImageBuffer::new(width, height);
                    for x in 0..width
                    {
                        for y in 0..height
                        {
                            let pixel = tex.get_pixel(x, y);
                            img.put_pixel(x, y, Rgba([pixel[0], pixel[1], pixel[2], 255]));
                        }
                    }

                    return DynamicImage::ImageRgba8(img.clone());
                }
            },
            TextureType::Roughness =>
            {
                if let Some(roughness_texture) = &mat.pbr.roughness_texture
                {
                    let tex = roughness_texture;
                    let width = tex.width();
                    let height = tex.height();

                    let mut img: RgbaImage = ImageBuffer::new(width, height);
                    for x in 0..width
                    {
                        for y in 0..height
                        {
                            let pixel = tex.get_pixel(x, y);
                            //let r = (pixel[0] as f32 * mat.pbr.roughness_factor) as u8;
                            let r = pixel[0];
                            img.put_pixel(x, y, Rgba([r, r, r, r]));
                        }
                    }

                    return DynamicImage::ImageRgba8(img.clone());
                }
            },
            TextureType::AmbientOcclusion =>
            {
                if let Some(ambient_occlusion) = &mat.occlusion
                {
                    let tex = &ambient_occlusion.texture;
                    let width = tex.width();
                    let height = tex.height();

                    let mut img: RgbaImage = ImageBuffer::new(width, height);
                    for x in 0..width
                    {
                        for y in 0..height
                        {
                            let pixel = tex.get_pixel(x, y);
                            let occlusion = (pixel[0] as f32 * ambient_occlusion.factor) as u8;
                            img.put_pixel(x, y, Rgba([occlusion, occlusion, occlusion, occlusion]));
                        }
                    }

                    return DynamicImage::ImageRgba8(img.clone());
                }
            },
            TextureType::Reflectivity =>
            {
                if let Some(metallic) = &mat.pbr.metallic_texture
                {
                    let tex = metallic;
                    let width = tex.width();
                    let height = tex.height();

                    let mut img: RgbaImage = ImageBuffer::new(width, height);
                    for x in 0..width
                    {
                        for y in 0..height
                        {
                            let pixel = tex.get_pixel(x, y);
                            //let m = (pixel[0] as f32 * mat.pbr.metallic_factor) as u8;
                            let m = pixel[0];
                            img.put_pixel(x, y, Rgba([m, m, m, m]));
                        }
                    }

                    return DynamicImage::ImageRgba8(img.clone());
                }
            },
            TextureType::AmbientEmissive =>
            {
                if let Some(emissive) = &mat.emissive.texture
                {
                    let tex = emissive;
                    let width = tex.width();
                    let height = tex.height();

                    let mut img: RgbaImage = ImageBuffer::new(width, height);
                    for x in 0..width
                    {
                        for y in 0..height
                        {
                            let pixel = tex.get_pixel(x, y);
                            let r = pixel[0];
                            let g = pixel[1];
                            let b = pixel[2];

                            img.put_pixel(x, y, Rgba([r, g, b, 255]));
                        }
                    }

                    return DynamicImage::ImageRgba8(img.clone());
                }
            },
            _ =>
            {}
        }

        DynamicImage::new_rgb8(0,0)
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
        let wavefront_materials = materials.unwrap();

        let mut double_check_materials: Vec<(usize, u32)> = vec![];

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
                let material_arc;

                //apply material
                if let Some(wavefront_mat_id) = mesh.material_id
                {
                    let mut reusing_material_object_id = 0;
                    for mat in &double_check_materials
                    {
                        if mat.0 == wavefront_mat_id
                        {
                            reusing_material_object_id = mat.1;
                            break;
                        }
                    }

                    if reusing_material_object_id != 0
                    {
                        material_arc = self.get_material_by_id(reusing_material_object_id).unwrap();
                    }
                    else
                    {
                        let material_id = self.get_next_id();
                        material_arc = Arc::new(RwLock::new(Box::new(Material::new(material_id, ""))));

                        let mat: &tobj::Material = &wavefront_materials[wavefront_mat_id];
                        let mut material = material_arc.write().unwrap();
                        material.name = mat.name.clone();

                        material.shininess = mat.shininess;
                        material.ambient_color = Vector3::<f32>::new(mat.ambient[0], mat.ambient[1], mat.ambient[2]);
                        material.specular_color = Vector3::<f32>::new(mat.specular[0], mat.specular[1], mat.specular[2]);
                        material.base_color = Vector3::<f32>::new(mat.diffuse[0], mat.diffuse[1], mat.diffuse[2]);
                        material.refraction_index = mat.optical_density;
                        material.alpha = mat.dissolve;

                        material.ambient_color = material.base_color * 0.01;

                        if let Some(illumination) = mat.illumination_model
                        {
                            if illumination > 2
                            {
                                material.reflectivity = 0.5;
                            }
                        }

                        // base texture
                        if !mat.diffuse_texture.is_empty()
                        {
                            let tex_path = self.get_texture_path(&mat.diffuse_texture, path);
                            dbg!(&tex_path);
                            material.load_texture(&tex_path, TextureType::Base);
                        }

                        // normal texture
                        if !mat.normal_texture.is_empty()
                        {
                            let tex_path = self.get_texture_path(&mat.normal_texture, path);
                            dbg!(&tex_path);
                            material.load_texture(&tex_path, TextureType::Normal);
                        }

                        // ambient texture
                        if !mat.ambient_texture.is_empty()
                        {
                            let tex_path = self.get_texture_path(&mat.ambient_texture, path);
                            dbg!(&tex_path);
                            material.load_texture(&tex_path, TextureType::AmbientEmissive);
                        }

                        // specular texture
                        if !mat.specular_texture.is_empty()
                        {
                            let tex_path = self.get_texture_path(&mat.specular_texture, path);
                            dbg!(&tex_path);
                            material.load_texture(&tex_path, TextureType::Specular);
                        }

                        // specular texture
                        if !mat.dissolve_texture.is_empty()
                        {
                            let tex_path = self.get_texture_path(&mat.dissolve_texture, path);
                            dbg!(&tex_path);
                            material.load_texture(&tex_path, TextureType::Alpha);
                        }

                        // shininess_texture is not supported

                        self.materials.push(material_arc.clone());
                        double_check_materials.push((wavefront_mat_id, material_id));
                    }
                }
                else
                {
                    let material_id = self.get_next_id();
                    material_arc = Arc::new(RwLock::new(Box::new(Material::new(material_id, ""))));
                }

                let mut item = Mesh::new_with_data(m.name.as_str(), material_arc.clone(), verts, indices, uvs, uv_indices, normals, normals_indices);

                item.get_basic_mut().id = self.get_next_id();
                loaded_ids.push(item.get_basic().id);

                self.items.push(Box::new(item));
            }

        }
        loaded_ids
    }

    pub fn add_light(&mut self, pos: Point3::<f32>, dir: Vector3::<f32>, color: Vector3::<f32>, intensity: f32, max_angle: f32, light_type: LightType)
    {
        let id = self.get_next_id();
        self.lights.push(Box::new(Light
        {
            enabled: true,
            id: id,
            pos: pos,
            dir: dir,
            color: color,
            intensity: intensity,
            max_angle: max_angle,
            light_type: light_type
        }));
    }

    pub fn add_default_light(&mut self)
    {
        let id = self.get_next_id();
        self.lights.push(Box::new(Light
        {
            enabled: true,
            id: id,
            pos: Point3::<f32>::new(-2.0, 10.0, 5.0),
            dir: Vector3::<f32>::new(0.0, -1.0, 0.0),
            color: Vector3::<f32>::new(1.0, 1.0, 1.0),
            intensity: 200.0,
            max_angle: PI / 2.0,
            light_type: LightType::Point
        }));
    }

    pub fn find_bottom_y_pos(&self) -> f32
    {
        let mut min_y = std::f32::MAX;

        // get all bounding box points of all objects
        for item in &self.items
        {
            let b_box = item.get_basic().b_box;

            let verts = b_box.vertices();

            let trans = item.get_basic().trans;

            for vert in &verts
            {
                let transformed = trans * vert.to_homogeneous();
                min_y = min_y.min(transformed.y);
            }
        }

        min_y
    }

    pub fn find_optimal_camera_pos(&mut self)
    {
        let mut points = vec![];

        let mut min: Point3::<f32> = Point3::<f32>::new(std::f32::MAX, std::f32::MAX, std::f32::MAX);
        let mut max: Point3::<f32> = Point3::<f32>::new(-std::f32::MAX, -std::f32::MAX, -std::f32::MAX);

        // get all bounding box points of all objects
        for item in &self.items
        {
            let b_box = item.get_basic().b_box;

            let verts = b_box.vertices();

            let trans = item.get_basic().trans;

            for vert in &verts
            {
                let transformed = trans * vert.to_homogeneous();
                points.push(transformed);

                min.x = min.x.min(transformed.x);
                min.y = min.y.min(transformed.y);
                min.z = min.z.min(transformed.z);

                max.x = max.x.max(transformed.x);
                max.y = max.y.max(transformed.y);
                max.z = max.z.max(transformed.z);
            }
        }

        let delta = Point3::<f32>::new((max.x - min.x).abs(), (max.y - min.y).abs(), (max.z - min.z).abs());
        let center = Point3::<f32>::new(min.x + (delta.x / 2.0), min.y + (delta.y / 2.0), min.z + (delta.z / 2.0));

        // camera
        let direction = OBLIQUE_CAM_POS.normalize();

        // set cam pos to center of the object
        self.cam.eye_pos = center;

        let mut factor = 0.0;
        let increment = 0.01;
        let max_factor = 1000.0;

        let padding_factor = 1.001;

        // ********** find best camera position **********
        while factor < max_factor
        {
            let check_pos = center + (direction * factor);

            self.cam.eye_pos.x = check_pos.x;
            self.cam.eye_pos.y = check_pos.y;
            self.cam.eye_pos.z = check_pos.z;

            self.cam.dir.x = -direction.x;
            self.cam.dir.y = -direction.y;
            self.cam.dir.z = -direction.z;

            self.cam.init_matrices();

            let mut in_frustum = true;
            for point in &points
            {
                let p = Point3::<f32>::new(point.x, point.y, point.z);
                if !self.cam.is_point_in_frustum(&p)
                {
                    in_frustum = false;
                    break;
                }
            }

            // apply padding
            if in_frustum
            {
                self.cam.eye_pos = self.cam.eye_pos + (direction * padding_factor);
            }

            if in_frustum
            {
                break;
            }

            factor += increment;
        }

        // ********** find best fov **********
        let mut fov = 0.0;

        let increment = 0.01;
        let max_fov = DEFAULT_FOV;

        let fov_padding = 1.1f32;

        while fov < max_fov
        {
            self.cam.fov = fov.to_radians();
            self.cam.init_matrices();

            let mut in_frustum = true;
            for point in &points
            {
                let p = Point3::<f32>::new(point.x, point.y, point.z);
                if !self.cam.is_point_in_frustum(&p)
                {
                    in_frustum = false;
                    break;
                }
            }

            if in_frustum
            {
                self.cam.fov *= fov_padding;
                break;
            }

            fov += increment;
        }

        self.cam.init_matrices();

    }

    pub fn find_and_set_default_env_if_needed(&mut self)
    {
        if self.cam.is_default_cam()
        {
            println!("default camera options deteced -> trying to find optimal camera pos");
            self.find_optimal_camera_pos();
        }

        if self.lights.len() == 0
        {
            println!("no lights in the scene found --> adding default light");
            self.add_default_light();
        }
    }

    pub fn add_ground_plane(&mut self)
    {
        let y_pos = self.find_bottom_y_pos();

        // add reflexivity ground plane
        self.load_json("scene/floor_reflective.json");

        let basic = self.get_by_name_mut("floor reflective").unwrap().get_basic_mut();
        basic.apply_translation(Vector3::<f32>::new(0.0, y_pos, 0.0));
    }

    pub fn add_environment_sphere(&mut self)
    {
        self.load_json("scene/environment.json");
    }

    pub fn delete_light_by_id(&mut self, id: u32)
    {
        let mut index: Option<usize> = None;

        let mut i: usize = 0;
        for light in &self.lights
        {
            if light.id == id
            {
                index = Some(i);
            }

            i += 1;
        }


        if let Some(index) = index
        {
            self.lights.remove(index);
        }
    }

    pub fn delete_object_by_id(&mut self, id: u32)
    {
        let mut index: Option<usize> = None;

        let mut i: usize = 0;
        for item in &self.items
        {
            if item.get_basic().id == id
            {
                index = Some(i);
            }

            i += 1;
        }


        if let Some(index) = index
        {
            self.items.remove(index);
        }
    }

    pub fn get_material_by_id(&self, id: u32) -> Option<MaterialItem>
    {
        for item in &self.materials
        {
            if item.read().unwrap().id == id
            {
                return Some(item.clone());
            }
        }

        None
    }

    pub fn get_material_by_id_mut(&mut self, id: u32) -> Option<MaterialItem>
    {
        for item in &self.materials
        {
            if item.read().unwrap().id == id
            {
                return Some(item.clone());
            }
        }

        None
    }

    pub fn get_texture_path(&self, tex_path: &String, mtl_path: &str) -> String
    {
        let mut tex_path = tex_path.clone();

        if Path::new(&tex_path).is_relative()
        {
            let parent = Path::new(mtl_path).parent();
            if let Some(parent) = parent
            {
                tex_path = parent.join(tex_path).to_str().unwrap().to_string();
            }
        }

        tex_path
    }

    pub fn init(&mut self)
    {
        for item in & mut self.items
        {
            item.init();
        }
    }

    pub fn update(&mut self)
    {
        for item in & mut self.items
        {
            item.update();
        }

        //update bvh
        let indices = (0..self.items.len()).collect::<Vec<usize>>();
        let expected_node_count = self.items.len() * 2;
        let mut nodes = Vec::with_capacity(expected_node_count);
        BVHNode::build(&mut self.items, &indices, &mut nodes, 0, 0);

        self.bvh.nodes = nodes;
    }

    pub fn frame_exists(&self, frame: u64) -> bool
    {
        self.animation.has_animation() && frame < self.animation.get_frames_amount_to_render()
    }

    pub fn apply_frame(&mut self, frame: u64) -> bool
    {
        if !self.animation.has_animation() || frame > self.animation.get_frames_amount_to_render()
        {
            return false;
        }

        for item in &mut self.items
        {
            let item_trans = self.animation.get_trans_for_frame(frame, item.get_basic().name.to_string());

            if let Some(item_trans) = item_trans
            {
                item.get_basic_mut().apply_mat(&item_trans);
            }
        }

        true
    }

    pub fn get_possible_hits_by_ray(&self, ray: &Ray) -> Vec<&ScemeItem>
    {
        let origin = bvh::Point3::new(ray.origin.x, ray.origin.y, ray.origin.z);
        let direction = bvh::Vector3::new(ray.dir.x, ray.dir.y, ray.dir.z);
        let ray = bvh::ray::Ray::new(origin, direction);

        self.bvh.traverse(&ray, &self.items)
    }

    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut ScemeItem>
    {
        for item in & mut self.items
        {
            if item.get_basic().name == name
            {
                return Some(item);
            }
        }

        None
    }

    pub fn get_obj_by_id(&self, id: u32) -> Option<&ScemeItem>
    {
        for item in & self.items
        {
            if item.get_basic().id == id
            {
                return Some(item);
            }
        }

        None
    }

    pub fn get_obj_by_id_mut(&mut self, id: u32) -> Option<&mut ScemeItem>
    {
        for item in & mut self.items
        {
            if item.get_basic().id == id
            {
                return Some(item);
            }
        }

        None
    }

    pub fn get_light_by_id(&self, id: u32) -> Option<&LightItem>
    {
        for item in & self.lights
        {
            if item.id == id
            {
                return Some(item);
            }
        }

        None
    }

    pub fn get_light_by_id_mut(&mut self, id: u32) -> Option<&mut LightItem>
    {
        for item in & mut self.lights
        {
            if item.id == id
            {
                return Some(item);
            }
        }

        None
    }

    pub fn get_vec_by_name(&mut self, name: &str) -> Vec<&mut ScemeItem>
    {
        let mut vec = vec![];
        for item in & mut self.items
        {
            if item.get_basic().name == name
            {
                vec.push(item);
            }
        }

        vec
    }

    pub fn print(&self)
    {
        println!("");
        println!("config:");
        println!("======");
        self.raytracing_config.print();

        println!("");
        println!("cam:");
        println!("==========");
        self.cam.print();

        println!("");
        println!("lights:");
        println!("==========");
        for light in &self.lights
        {
            let pos = light.pos;
            let dir = light.dir;
            let color = light.color;
            let intensity = light.intensity;
            let max_angle = light.max_angle;
            let light_type = light.light_type;

            println!(" - {:?}: pos: {:?}, dir: {:?}, color: {:?}, intensity: {}, max_angle: {}", light_type, pos, dir, color, intensity, max_angle);
        }

        println!("");
        println!("scene:");
        println!("==========");
        for item in &self.items
        {
            let id = item.get_basic().id;
            let name = item.get_basic().name.clone();
            let visible = item.get_basic().visible;

            let material = item.get_material().read().unwrap();

            let b_tex = material.has_texture(TextureType::Base);
            let am_tex = material.has_texture(TextureType::Alpha);
            let s_tex = material.has_texture(TextureType::Specular);
            let n_tex = material.has_texture(TextureType::Normal);
            let a_tex = material.has_texture(TextureType::Alpha);

            println!(" - {}: {} (visible: {}, bTex: {}, amTex: {}, sTex: {}, nTex: {}, aTex: {})", id, name, visible, b_tex, am_tex, s_tex, n_tex, a_tex);
        }

        println!("");
        println!("animation:");
        println!("==========");
        println!("activated: {}", self.animation.has_animation());
        println!("fps: {}", self.animation.fps);
        println!("frames_to_render: {}", self.animation.get_frames_amount_to_render());
        //dbg!(&self.animation);
    }
}