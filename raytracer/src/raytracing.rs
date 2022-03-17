use std::f32::consts::PI;
use std::sync::{RwLock, Arc};

use crate::shape::{Shape, TextureType};
use crate::pixel_color::PixelColor;

use crate::scene::{Scene, LightType};

use nalgebra::{Point3, Vector3, Matrix3, Vector4, Point2};
use parry3d::query::{Ray};

use rand::Rng;
use rand::seq::SliceRandom;
use serde_json::Value;

const SHADOW_BIAS: f32 = 0.001;
const APERTURE_BASE_RESOLUTION: f32 = 800.0;

const CAM_CLIPPING_PLANE_DIST: f32 = 1.0;
const DEFAILT_VIEW_POS: Vector4::<f32> = Vector4::<f32>::new(0.0, 0.0, 0.0, 1.0);

/*
some resources:

Raytracing in general:
https://bheisler.github.io/post/writing-raytracer-in-rust-part-1/
https://raytracing.github.io/books/RayTracingInOneWeekend.html

normal mapping:
https://stackoverflow.com/questions/41015574/raytracing-normal-mapping
http://www.opengl-tutorial.org/intermediate-tutorials/tutorial-13-normal-mapping/
https://lettier.github.io/3d-game-shaders-for-beginners/normal-mapping.html

strengthen for normal mapping:
https://computergraphics.stackexchange.com/questions/5411/correct-way-to-set-normal-strength/5412

DOF (Depth of field):
http://courses.washington.edu/css552/2016.Winter/FinalProjects/2.DOF/Final_Project_Presentation.pdf
https://web.archive.org/web/20160103203728/https://cg.skeelogy.com/depth-of-field-using-raytracing/

monte carlo:
https://www.youtube.com/watch?v=KCYroQVaARs
https://www.youtube.com/watch?v=R9iZzaXUaK4
https://wzhfantastic.github.io/2018/04/09/RayTracingInUnity(PartTwo)/

hemi sphere sampling
https://www.gamedev.net/forums/topic/683176-finding-a-random-point-on-a-sphere-with-spread-and-direction/
*/

pub struct HitResult<'a>
{
    item: &'a dyn Shape,
    dist: f32,
}

pub enum LightningColorType
{
    Ambient,
    Base,
    Specular
}

pub struct Raytracing
{
    scene: Arc<RwLock<Scene>>,

    monte_carlo: bool,

    samples: u16,

    focal_length: f32,
    aperture_size: f32,

    fog_density: f32,
    fog_color: Vector3<f32>,

    max_recursion: u16,
    gamma_correction: bool
}

impl Raytracing
{
    pub fn new(scene: Arc<RwLock<Scene>>) -> Raytracing
    {
        Raytracing
        {
            scene: scene,

            monte_carlo: true,

            samples: 8, //this includes anti aliasing

            focal_length: 8.0,
            aperture_size: 64.0, //64.0 (1 means off)

            fog_density: 0.0,
            fog_color: Vector3::<f32>::new(0.4, 0.4, 0.4),

            max_recursion: 6,
            gamma_correction: false
        }
    }

    pub fn load_settings(&mut self, path: &str)
    {
        let data = std::fs::read_to_string(path);

        if data.is_ok()
        {
            let str = data.unwrap();
            let data = serde_json::from_str::<Value>(&str);
            if data.is_ok()
            {
                let data = data.unwrap();

                if !&data["monte_carlo"].is_null() { self.monte_carlo = data["monte_carlo"].as_bool().unwrap(); }
                if !&data["samples"].is_null() { self.samples = data["samples"].as_u64().unwrap() as u16; }

                if !&data["focal_length"].is_null() { self.focal_length = data["focal_length"].as_f64().unwrap() as f32; }
                if !&data["aperture_size"].is_null() { self.aperture_size = data["aperture_size"].as_f64().unwrap() as f32; }

                if !&data["fog_density"].is_null() { self.fog_density = data["fog_density"].as_f64().unwrap() as f32; }
                if !&data["fog_color"].is_null()
                {
                    self.fog_color.x = data["fog_color"]["r"].as_f64().unwrap() as f32;
                    self.fog_color.y = data["fog_color"]["g"].as_f64().unwrap() as f32;
                    self.fog_color.z = data["fog_color"]["b"].as_f64().unwrap() as f32;
                }

                if !&data["max_recursion"].is_null() { self.max_recursion = data["max_recursion"].as_u64().unwrap() as u16; }
                if !&data["gamma_correction"].is_null() { self.gamma_correction = data["gamma_correction"].as_bool().unwrap(); }
            }
            else
            {
                println!("error can not parse settings file: {}", path);
            }
        }
        else
        {
            println!("error can not load settings file: {}", path);
        }
    }

    pub fn print_settings(&self)
    {
        println!("monte_carlo: {:?}", self.monte_carlo);
        println!("samples: {:?}", self.samples);

        println!("focal_length: {:?}", self.focal_length);
        println!("aperture_size: {:?}", self.aperture_size);

        println!("fog_density: {:?}", self.fog_density);
        println!("fog_color: {:?}", self.fog_color);

        println!("max_recursion: {:?}", self.max_recursion);
        println!("gamma_correction: {:?}", self.gamma_correction);
    }

    pub fn gamma_encode(&self, linear: f32) -> f32
    {
        const GAMMA: f32 = 2.2;
        linear.powf(1.0 / GAMMA)
    }

    pub fn pick(&self, x: i32, y: i32) -> Option<(String, f32)>
    {
        let scene = self.scene.read().unwrap();

        let x_f = x as f32;
        let y_f = y as f32;

        let w = scene.cam.width as f32;
        let h = scene.cam.height as f32;

        //map x/y to -1 <=> +1
        let sensor_x = ((x_f + 0.5) / w) * 2.0 - 1.0;
        let sensor_y = 1.0 - ((y_f + 0.5) / h) * 2.0;

        let mut pixel_pos = Vector4::new(sensor_x, sensor_y, -CAM_CLIPPING_PLANE_DIST, 1.0);
        pixel_pos = scene.cam.projection_inverse * pixel_pos;
        pixel_pos.w = 1.0;

        let mut ray_dir = pixel_pos - DEFAILT_VIEW_POS;
        ray_dir.w = 0.0;

        let origin = scene.cam.view_inverse * pixel_pos;
        let dir = scene.cam.view_inverse * ray_dir;

        let mut ray = Ray::new(Point3::<f32>::from(origin.xyz()), Vector3::<f32>::from(dir.xyz()));
        ray.dir = ray.dir.normalize();

        //scene
        let scene = self.scene.read().unwrap();

        //intersect
        let intersection = self.trace(&scene, &ray, false, false);

        if let Some(intersection) = intersection
        {
            return Some((intersection.2.get_name().to_string(), intersection.0));
        }

        None
    }

    pub fn render(&self, x: i32, y: i32) -> PixelColor
    {
        let scene = self.scene.read().unwrap();

        let x_f = x as f32;
        let y_f = y as f32;

        let w = scene.cam.width as f32;
        let h = scene.cam.height as f32;

        let x_step = 2.0 / w;
        let y_step = 2.0 / h;

        let mut color = Vector3::new(0.0, 0.0, 0.0);

        let mut samples = vec![];

        let mut cell_size = 1;
        if self.samples > 1
        {
            //increase samples value to not exactly match power of two with sampling steps
            //otherweise this would result in crappy visual effects for DOF blur
            cell_size = (self.samples + 2).next_power_of_two() / 2;
        }

        for x_i in 0..cell_size
        {
            for y_i in 0..cell_size
            {
                samples.push((x_i, y_i));
            }
        }

        //randomize
        samples.shuffle(&mut rand::thread_rng());

        //truncate by samples-amout
        samples.truncate(self.samples as usize);

        for sample in &samples
        {
            let x_i = sample.0;
            let y_i = sample.1;

            //calculate the movement arrount the x/y pos to render (based on anti aliasing and apperture)
            let mut x_trans = x_step * x_i as f32 * (1.0 / cell_size as f32);
            let mut y_trans = y_step * y_i as f32 * (1.0 / cell_size as f32);

            //move translation to center if needed
            if self.aperture_size > 1.0 && self.focal_length > 1.0 && self.samples > 1
            {
                x_trans -= x_step / 2.0;
                y_trans -= y_step / 2.0;
            }

            let ray;

            //DOF (depth of field)
            if self.aperture_size > 1.0 && self.focal_length > 1.0
            {
                let aperture_scale = scene.cam.width as f32 / APERTURE_BASE_RESOLUTION;
                x_trans *= self.aperture_size * aperture_scale;
                y_trans *= self.aperture_size * aperture_scale;

                // ***** calculate the center pixel pos
                let center_x = ((x_f + 0.5) / w) * 2.0 - 1.0;
                let center_y = 1.0 - ((y_f + 0.5) / h) * 2.0;

                let mut center_pixel_pos = Vector4::new(center_x, center_y, -CAM_CLIPPING_PLANE_DIST, 1.0);
                center_pixel_pos = scene.cam.projection_inverse * center_pixel_pos;
                center_pixel_pos.w = 1.0;

                let mut ray_dir = center_pixel_pos - DEFAILT_VIEW_POS;
                ray_dir.w = 0.0;

                let origin = scene.cam.view_inverse * DEFAILT_VIEW_POS;
                let dir = (scene.cam.view_inverse * ray_dir).normalize();

                let dist = ray_dir.xyz().magnitude();

                // ***** calculate the focal point
                let dist_perpendicular = CAM_CLIPPING_PLANE_DIST;
                let p = origin + ((dist_perpendicular/(dist/(dist + self.focal_length)))*dir);

                // ***** calculate ray
                let ray_sensor_x = (((x_f + 0.5) / w) * 2.0 - 1.0) + x_trans;
                let ray_sensor_y = (1.0 - ((y_f + 0.5) / h) * 2.0) + y_trans;

                let mut pixel_pos = Vector4::new(ray_sensor_x, ray_sensor_y, -CAM_CLIPPING_PLANE_DIST, 1.0);
                pixel_pos = scene.cam.projection_inverse * pixel_pos;
                pixel_pos.w = 1.0;

                let ray_origin = scene.cam.view_inverse * pixel_pos;
                let mut ray_dir = p - ray_origin; //p is already in view mat space
                ray_dir.w = 0.0;

                ray = Ray::new(Point3::<f32>::from(ray_origin.xyz()), Vector3::<f32>::from(ray_dir.xyz()));
            }
            //with or without anti aliasing and without DOF
            else
            {
                //map x/y to -1 <=> +1
                let sensor_x = (((x_f + 0.5) / w) * 2.0 - 1.0) + x_trans;
                let sensor_y = (1.0 - ((y_f + 0.5) / h) * 2.0) + y_trans;

                let mut pixel_pos = Vector4::new(sensor_x, sensor_y, -CAM_CLIPPING_PLANE_DIST, 1.0);
                pixel_pos = scene.cam.projection_inverse * pixel_pos;
                pixel_pos.w = 1.0;

                let mut ray_dir = pixel_pos - DEFAILT_VIEW_POS;
                ray_dir.w = 0.0;

                let origin = scene.cam.view_inverse * pixel_pos;
                let dir = scene.cam.view_inverse * ray_dir;

                ray = Ray::new(Point3::<f32>::from(origin.xyz()), Vector3::<f32>::from(dir.xyz()));
            }

            color += self.get_color(ray, 1);
        }

        color /= samples.len() as f32;

        //clamp
        color.x.min(1.0);
        color.y.min(1.0);
        color.z.min(1.0);

        if self.gamma_correction
        {
            let r = (self.gamma_encode(color.x) * 255.0) as u8;
            let g = (self.gamma_encode(color.y) * 255.0) as u8;
            let b = (self.gamma_encode(color.z) * 255.0) as u8;

            PixelColor { r: r, g: g, b: b, x: x, y: y }
        }
        else
        {
            let r = (color.x * 255.0) as u8;
            let g = (color.y * 255.0) as u8;
            let b = (color.z * 255.0) as u8;

            PixelColor { r: r, g: g, b: b, x: x, y: y }
        }
    }

    pub fn trace<'a>(&self, scene: &'a Scene, ray: &Ray, stop_on_first_hit: bool, for_shadow: bool) -> Option<(f32, Vector3<f32>, &'a dyn Shape, u32)>
    {
        //find hits (bbox based)
        let mut hits: Vec<HitResult> = vec![];
        for item in &scene.items
        {
            let dist = item.intersect_b_box(&ray);
            if let Some(dist) = dist
            {
                if item.get_basic().visible && item.get_material().alpha > 0.0 && (!for_shadow || item.get_material().cast_shadow)
                {
                    hits.push(HitResult{ item: item.as_ref(), dist: dist });
                }
            }
        }
        if hits.len() == 0
        {
            return None;
        }

        //sort bbox dist (to get the nearest)
        hits.sort_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap());

        let mut best_hit: Option<(f32, Vector3<f32>, &dyn Shape, u32)> = None;

        for item in hits
        {
            let intersection = item.item.intersect(&ray);

            if let Some(intersection) = intersection
            {
                if best_hit.is_none() || best_hit.is_some() && intersection.0 < best_hit.unwrap().0
                {
                    best_hit = Some((intersection.0, intersection.1, item.item, intersection.2));
                }
            }

            //if it should return on first hit
            if best_hit.is_some() && stop_on_first_hit
            {
                return best_hit;
            }
        }

        best_hit
    }

    pub fn create_reflection(&self, normal: Vector3<f32>, incident: Vector3<f32>, intersection: Point3<f32>) -> Ray
    {
        let origin = intersection + (normal * SHADOW_BIAS);
        let dir = incident - (2.0 * incident.dot(&normal) * normal);

        Ray::new(origin, dir)
    }

    pub fn create_transmission(&self, normal: Vector3<f32>, incident: Vector3<f32>, intersection: Point3<f32>, index: f32) -> Option<Ray>
    {
        let mut ref_n = normal;
        let mut eta_t = index;
        let mut eta_i = 1.0f32;
        let mut i_dot_n = incident.dot(&normal);

        if i_dot_n < 0.0
        {
            //outside the surface
            i_dot_n = -i_dot_n;
        }
        else
        {
            //inside the surface; invert the normal and swap the indices of refraction
            ref_n = -normal;
            eta_t = 1.0;
            eta_i = index;
        }

        let eta = eta_i / eta_t;
        let k = 1.0 - (eta * eta) * (1.0 - i_dot_n * i_dot_n);
        if k < 0.0
        {
            None
        }
        else
        {
            let origin = intersection + (ref_n * -SHADOW_BIAS);
            let dir = (incident + i_dot_n * ref_n) * eta - ref_n * k.sqrt();

            Some(Ray::new(origin, dir))
        }
    }

    fn fresnel(&self, incident: Vector3<f32>, normal: Vector3<f32>, index: f32) -> f32
    {
        let i_dot_n = incident.dot(&normal);

        let mut eta_i = 1.0;
        let mut eta_t = index as f32;

        if i_dot_n > 0.0
        {
            eta_i = eta_t;
            eta_t = 1.0;
        }

        let sin_t = eta_i / eta_t * (1.0 - i_dot_n * i_dot_n).max(0.0).sqrt();

        if sin_t > 1.0
        {
            //total internal reflection
            return 1.0;
        }
        else
        {
            let cos_t = (1.0 - sin_t * sin_t).max(0.0).sqrt();
            let cos_i = cos_t.abs();
            let r_s = ((eta_t * cos_i) - (eta_i * cos_t)) / ((eta_t * cos_i) + (eta_i * cos_t));
            let r_p = ((eta_i * cos_i) - (eta_t * cos_t)) / ((eta_i * cos_i) + (eta_t * cos_t));
            return (r_s * r_s + r_p * r_p) / 2.0;
        }
    }

    pub fn jitter(&self, dir: Vector3<f32>, spread: f32) -> Vector3<f32>
    {
        if spread <= 0.0
        {
            return dir;
        }

        /*

        let mut rng = rand::thread_rng();

        //not the perfect solution (it is not angle based) but it works for now
        let mut new_dir = dir;
        new_dir.x += ((2.0 * rng.gen::<f32>()) - 1.0) * spread;
        new_dir.y += ((2.0 * rng.gen::<f32>()) - 1.0) * spread;
        new_dir.z += ((2.0 * rng.gen::<f32>()) - 1.0) * spread;
         */

        /*
        let rot_x = ((rng.gen::<f32>() * PI * 2.0) - PI) * spread;
        let rot_y = ((rng.gen::<f32>() * PI * 2.0) - PI) * spread;
        let rot_z = ((rng.gen::<f32>() * PI * 2.0) - PI) * spread;

        let rotation_mat = Isometry3::rotation(Vector3::new(rot_x, rot_y, rot_z));

        let new_dir = rotation_mat * dir;

        new_dir.normalize()
         */

        let b3 = dir.normalize();

        let diff;
        if b3.x.abs() < 0.5
        {
            diff = Vector3::<f32>::new(1.0, 0.0, 0.0);
        }
        else
        {
            diff = Vector3::<f32>::new(0.0, 1.0, 0.0);
        }

        let b1 = b3.cross(&diff).normalize();
        let b2 = b1.cross(&b3);

        let z = rand::thread_rng().gen_range((spread * PI).cos()..1.0);
        let r = (1.0 - z * z).sqrt();
        let theta = rand::thread_rng().gen_range(-PI..PI);
        let x = r * theta.cos();
        let y = r * theta.sin();

        let new_dir = x * b1 + y * b2 + z * b3;
        //let new_dir = Vector3<f32>::new(x * b1, y * b2)

        new_dir.normalize()
    }

    fn wrap(&self, val: f32, bound: u32) -> u32
    {
        let signed_bound = bound as i32;
        let float_coord = val * bound as f32;
        let wrapped_coord = (float_coord as i32) % signed_bound;
        if wrapped_coord < 0
        {
            (wrapped_coord + signed_bound) as u32
        }
        else
        {
            wrapped_coord as u32
        }
    }

    pub fn get_tex_color(&self, item: &dyn Shape, uv: &Option<Point2<f32>>, tex_type: TextureType) -> Option<Vector4<f32>>
    {
        //texture
        if (*item).get_basic().material.has_texture(tex_type) && uv.is_some()
        {
            let uv = uv.unwrap();

            let tex_dims = (*item).get_basic().material.texture_dimension(tex_type);
            let tex_x = self.wrap(uv.x, tex_dims.0);
            let tex_y = self.wrap(uv.y, tex_dims.1);

            let tex_color = (*item).get_basic().material.get_texture_pixel(tex_x, tex_y, tex_type);

            return Some(tex_color);
        }

        None
    }

    pub fn get_item_color(&self, item: &dyn Shape, uv: &Option<Point2<f32>>, color_type: LightningColorType) -> Vector4<f32>
    {
        let mat = (*item).get_material();

        let mut item_color;
        let tex_type;
        match color_type
        {
            LightningColorType::Ambient =>
            {
                item_color = Vector4::<f32>::new(mat.ambient_color.x, mat.ambient_color.y, mat.ambient_color.z, 1.0);
                tex_type = TextureType::Ambient;
            },
            LightningColorType::Base =>
            {
                item_color = Vector4::<f32>::new(mat.base_color.x, mat.base_color.y, mat.base_color.z, 1.0);
                tex_type = TextureType::Base;
            },
            LightningColorType::Specular =>
            {
                item_color = Vector4::<f32>::new(mat.specular_color.x, mat.specular_color.y, mat.specular_color.z, 1.0);
                tex_type = TextureType::Specular;
            },
        }

        //texture color
        let tex_color = self.get_tex_color(item, uv, tex_type);

        if let Some(tex_color) = tex_color
        {
            item_color.x *= tex_color.x;
            item_color.y *= tex_color.y;
            item_color.z *= tex_color.z;
            item_color.w *= tex_color.w;
        }

        item_color
    }

    pub fn reflect(&self, i: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32>
    {
        //https://www.khronos.org/registry/OpenGL-Refpages/gl4/html/reflect.xhtml
        i - 2.0 * n.dot(&i) * n
    }

    pub fn get_color(&self, ray: Ray, depth: u16) -> Vector3<f32>
    {
        let scene = self.scene.read().unwrap();

        let mut r = ray;
        r.dir = r.dir.normalize();

        //intersect
        let intersection = self.trace(&scene, &r, false, false);

        let mut color = Vector3::new(0.0, 0.0, 0.0);

        if let Some(intersection) = intersection
        {
            let hit_dist = intersection.0;
            let normal = intersection.1;
            let item = intersection.2;
            let face_id = intersection.3;

            let mut surface_normal = normal;
            let hit_point = r.origin + (r.dir * hit_dist);

            //get uv
            let mut uv = None;
            if item.get_material().has_any_texture()
            {
                uv = Some((*item).get_uv(hit_point, face_id));
            }

            //normal mapping
            let normal_tex_color = self.get_tex_color(item, &uv, TextureType::Normal);
            if let Some(normal_tex_color) = normal_tex_color
            {
                let mut tangent = normal.cross(&Vector3::<f32>::new(0.0, 1.0, 0.0));

                if tangent.magnitude()  <= 0.0001
                {
                    tangent = normal.cross(&Vector3::<f32>::new(0.0, 0.0, 1.0));
                }

                tangent = tangent.normalize();
                let bitangent = normal.cross(&tangent).normalize();

                //to tagent space -- n * 2 - 1
                let mut normal_map = normal_tex_color.xyz();
                normal_map.x = (normal_map.x * 2.0) - 1.0;
                normal_map.y = (normal_map.y * 2.0) - 1.0;
                normal_map.z = (normal_map.z * 2.0) - 1.0;

                normal_map.x *= item.get_material().normal_map_strength;
                normal_map.y *= item.get_material().normal_map_strength;

                normal_map = normal_map.normalize();

                let tbn = Matrix3::<f32>::from_columns(&[tangent, bitangent, normal]);

                surface_normal = (tbn * normal_map).normalize();
            }

            //roughness map (overwrites roughness material setting)
            let roughness_tex_color = self.get_tex_color(item, &uv, TextureType::Roughness);
            if self.monte_carlo && (item.get_material().surface_roughness > 0.0 || roughness_tex_color.is_some())
            {
                let mut roughness = item.get_material().surface_roughness;

                if let Some(roughness_tex_color) = roughness_tex_color
                {
                    roughness = (1.0 / PI / 2.0) * roughness_tex_color.x;
                }

                surface_normal = self.jitter(surface_normal, roughness);
            }

            // TODO metallic texture



            //ambient, diffuse, specular colors
            let ambient_color = self.get_item_color(item, &uv, LightningColorType::Ambient);
            let base_color = self.get_item_color(item, &uv, LightningColorType::Base);
            let specular_color = self.get_item_color(item, &uv, LightningColorType::Specular);

            //alpha mapping
            let mut alpha = item.get_material().alpha * base_color.w;
            let alpha_tex_color = self.get_tex_color(item, &uv, TextureType::Alpha);
            if let Some(alpha_tex_color) = alpha_tex_color
            {
                alpha *= alpha_tex_color.x;
            }

            //ambient
            color = ambient_color.xyz();

            //diffuse/specular color
            for light in &scene.lights
            {
                //get direction to light based on light type
                let direction_to_light;

                match light.light_type
                {
                    LightType::Directional => direction_to_light = (-light.dir).normalize(),
                    LightType::Point => direction_to_light = (light.pos - hit_point).normalize(),
                    LightType::Spot => direction_to_light = (light.pos - hit_point).normalize(),
                }

                //lambert
                let dot_light = surface_normal.dot(&direction_to_light).max(0.0);

                let base = base_color * dot_light;

                //phong
                let reflect_dir = self.reflect(-direction_to_light, surface_normal);
                let view_dir = (-r.dir).normalize();
                let spec_dot = reflect_dir.dot(&view_dir).max(0.0);
                let light_power = spec_dot.powf(item.get_material().shininess);

                let specular = specular_color * light_power;

                //light intensity
                let mut intensity;
                match light.light_type
                {
                    LightType::Directional => intensity = light.intensity,
                    LightType::Point =>
                    {
                        let r2 = (light.pos - hit_point).norm() as f32;
                        intensity = light.intensity / (4.0 * PI * r2)
                    },
                    LightType::Spot =>
                    {
                        //use point as base and check angle
                        let r2 = (light.pos - hit_point).norm() as f32;
                        intensity = light.intensity / (4.0 * PI * r2);

                        let light_dir = light.dir.normalize();
                        let dot = (-direction_to_light).dot(&light_dir);
                        let angle = dot.acos();

                        if angle > light.max_angle
                        {
                            intensity = 0.0;
                        }
                    }
                }

                //shadow
                if item.get_material().receive_shadow
                {
                    let shadow_ray_start = hit_point + (surface_normal * SHADOW_BIAS);
                    let mut shadow_ray_dir = direction_to_light;

                    if self.monte_carlo
                    {
                        shadow_ray_dir = self.jitter(shadow_ray_dir, item.get_material().shadow_softness);
                    }

                    let shadow_ray = Ray::new(shadow_ray_start, shadow_ray_dir);
                    let shadow_intersection = self.trace(&scene, &shadow_ray, true, true);

                    let mut in_light = shadow_intersection.is_none();
                    if !in_light && (light.light_type == LightType::Point || light.light_type == LightType::Spot)
                    {
                        let light_dist: Vector3<f32> = light.pos - hit_point;
                        let len = light_dist.norm();

                        in_light = shadow_intersection.unwrap().0 > len
                    }

                    //shadow intensity (including alpha map based shadow check)
                    if !in_light
                    {
                        let shadow_obj = shadow_intersection.unwrap().2;
                        let mut shadow_source_alpha = shadow_obj.get_material().alpha;

                        let shadow_face_id = shadow_intersection.unwrap().3;

                        let shadow_hit_point = shadow_ray.origin + (shadow_ray.dir * shadow_intersection.unwrap().0);

                        let shadow_uv = (*item).get_uv(shadow_hit_point, shadow_face_id);
                        let shadow_alpha_tex_color = self.get_tex_color(shadow_obj, &Some(shadow_uv), TextureType::Alpha);
                        if let Some(shadow_alpha_tex_color) = shadow_alpha_tex_color
                        {
                            shadow_source_alpha *= shadow_alpha_tex_color.x;
                        }

                        intensity = intensity * (1.0 - shadow_source_alpha);
                    }
                }

                //color based on components
                color.x = color.x + ((light.color.x * (specular.x + base.x)) * intensity);
                color.y = color.y + ((light.color.y * (specular.y + base.y)) * intensity);
                color.z = color.z + ((light.color.z * (specular.z + base.z)) * intensity);
            }

            let refraction_index = item.get_material().refraction_index;

            //fresnel
            let kr = self.fresnel(r.dir, surface_normal, refraction_index);

            //reflectivity
            let reflectivity = item.get_material().reflectivity;
            color = color * (1.0 - reflectivity);

            if item.get_material().reflectivity > 0.0 && depth <= self.max_recursion
            {
                let reflection_ray = self.create_reflection(surface_normal, r.dir, hit_point);
                let reflection_color = self.get_color(reflection_ray, depth + 1 );

                //color = color + (reflection_color * reflectivity * kr);
                color = color + (reflection_color * reflectivity);
            }

            //refraction
            if alpha < 1.0 && depth <= self.max_recursion
            {
                let transmission_ray = self.create_transmission(surface_normal, r.dir, hit_point, refraction_index);

                if let Some(transmission_ray) = transmission_ray
                {
                    let refraction_color = self.get_color(transmission_ray, depth + 1);

                    if kr < 1.0
                    {
                        color = (color * alpha) + (refraction_color * (1.0 - kr) * (1.0 - alpha));
                    }
                    else
                    {
                        color = (color * alpha) + (refraction_color * (1.0 - alpha));
                    }
                }
            }
            else if alpha < 1.0
            {
                color = color * alpha;
            }

            //fog
            {
                let fog_amount = (self.fog_density * hit_dist).min(1.0);

                color = ((1.0 - fog_amount) * color) + (self.fog_color * fog_amount);
            }

            //ambient occlusion
            let ambient_occlusion = self.get_tex_color(item, &uv, TextureType::AmbientOcclusion);
            if let Some(ambient_occlusion) = ambient_occlusion
            {
                color.x *= ambient_occlusion.x;
                color.y *= ambient_occlusion.x;
                color.z *= ambient_occlusion.x;
            }
        }

        color
    }
}