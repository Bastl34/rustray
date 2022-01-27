use crate::shape::{Shape, TextureType};
use crate::pixel_color::PixelColor;

use crate::scene::{Scene, LightType};

use nalgebra::{Perspective3, Isometry3, Point3, Vector3, Matrix3};
use parry3d::query::{Ray};

use rand::seq::SliceRandom;

const SHADOW_BIAS: f32 = 0.001;

/*
some resources:

Raytracing in general:
https://bheisler.github.io/post/writing-raytracer-in-rust-part-1/

normal mapping:
https://stackoverflow.com/questions/41015574/raytracing-normal-mapping
http://www.opengl-tutorial.org/intermediate-tutorials/tutorial-13-normal-mapping/
https://lettier.github.io/3d-game-shaders-for-beginners/normal-mapping.html

strengthen for normal mapping:
https://computergraphics.stackexchange.com/questions/5411/correct-way-to-set-normal-strength/5412

DoF:
http://courses.washington.edu/css552/2016.Winter/FinalProjects/2.DOF/Final_Project_Presentation.pdf
*/

pub struct HitResult<'a>
{
    item: &'a dyn Shape,
    dist: f32,
}

pub enum LightningColorType
{
    Ambient,
    Diffuse,
    Specular
}

pub struct Raytracing
{
    scene: Scene,

    width: u32,
    height: u32,

    anti_aliasing: u8,
    samples: u16,

    focal_length: f32,
    aperture_size: f32,

    max_recursion: u16,
    gamma_correction: bool,

    aspect_ratio: f32,

    fov: f32,
    fov_adjustment: f32,

    projection: Perspective3<f32>,
    view: Isometry3<f32>
}

impl Raytracing
{
    pub fn new(scene: Scene) -> Raytracing
    {
        Raytracing
        {
            scene: scene,

            width: 0,
            height: 0,

            aspect_ratio: 0.0,

            anti_aliasing: 16, //16
            samples: 128, //64

            focal_length: 8.0,
            aperture_size: 64.0, //64.0

            max_recursion: 6,
            gamma_correction: false,

            fov: 0.0,
            fov_adjustment: 0.0,

            //TODO: use projection mat instead of manual calc
            projection: Perspective3::<f32>::new(1.0f32, 0.0f32, 0.001, 1000.0),
            view: Isometry3::<f32>::identity()
        }
    }

    pub fn init_camera(&mut self, width: u32, height: u32)
    {
        self.width = width;
        self.height = height;

        self.aspect_ratio = width as f32 / height as f32;
        self.fov = 3.14 / 2.0;

        self.fov_adjustment = (self.fov / 2.0).tan();

        self.projection = Perspective3::new(self.aspect_ratio, self.fov, 0.001, 1000.0);

        let eye    = Point3::new(0.0, 0.0, 1.0);
        let target = Point3::new(1.0, 0.0, 0.0);

        self.view = Isometry3::look_at_rh(&eye, &target, &Vector3::y());
    }

    pub fn gamma_encode(&self, linear: f32) -> f32
    {
        const GAMMA: f32 = 2.2;
        linear.powf(1.0 / GAMMA)
    }

    pub fn render(&self, x: i32, y: i32) -> PixelColor
    {
        let x_f = x as f32;
        let y_f = y as f32;

        let w = self.width as f32;
        let h = self.height as f32;

        let aa_f = self.anti_aliasing as f32;

        let x_step = 2.0 / w;
        let y_step = 2.0 / h;

        let mut color = Vector3::new(0.0, 0.0, 0.0);

        let mut samples = vec![];

        for x_i in 0..self.anti_aliasing
        {
            for y_i in 0..self.anti_aliasing
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
            let mut x_trans = x_step * x_i as f32 * (1.0 / aa_f);
            let mut y_trans = y_step * y_i as f32 * (1.0 / aa_f);

            //move translation to center if needed
            if (self.aperture_size > 1.0 && self.focal_length > 1.0) || self.anti_aliasing > 1
            {
                x_trans -= x_step / 2.0;
                y_trans -= y_step / 2.0;
            }

            let ray;

            //DoF (depth of field)
            if self.aperture_size > 1.0 && self.focal_length > 1.0
            {
                x_trans *= self.aperture_size;
                y_trans *= self.aperture_size;


                let origin = Point3::<f32>::origin();

                //let temp_x = ((((x as f32 + 0.5) / w) * 2.0 - 1.0) * self.aspect_ratio) * self.fov_adjustment;
                //let temp_y = (1.0 - ((y as f32 + 0.5) / h) * 2.0) * self.fov_adjustment;

                let sensor_x = (((((x_f + 0.5) / w) * 2.0 - 1.0)) * self.aspect_ratio) * self.fov_adjustment;
                let sensor_y = ((1.0 - ((y_f + 0.5) / h) * 2.0)) * self.fov_adjustment;

                let dist_perpendicular = 1.0;
                let mut pixel_pos = Point3::new(sensor_x, sensor_y, -dist_perpendicular);
                let dist = (pixel_pos - origin).magnitude();
                let dir = (pixel_pos - origin).normalize();

                let p = origin + ((dist_perpendicular/(dist/(dist + self.focal_length)))*dir);

                let ray_sensor_x = (((((x_f + 0.5) / w) * 2.0 - 1.0) + x_trans) * self.aspect_ratio) * self.fov_adjustment;
                let ray_sensor_y = ((1.0 - ((y_f + 0.5) / h) * 2.0) + y_trans) * self.fov_adjustment;
                pixel_pos = Point3::new(ray_sensor_x, ray_sensor_y, -dist_perpendicular);

                let ray_dir = p - pixel_pos;

                ray = Ray::new(pixel_pos, ray_dir);
            }
            //with or without anti aliasing and without DoF
            else
            {
                //map x/y to -1 <=> +1
                let sensor_x = (((((x_f + 0.5) / w) * 2.0 - 1.0) + x_trans) * self.aspect_ratio) * self.fov_adjustment;
                let sensor_y = ((1.0 - ((y_f + 0.5) / h) * 2.0) + y_trans) * self.fov_adjustment;

                ray = Ray::new(Point3::origin(), Vector3::new(sensor_x, sensor_y, -1.0));
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

    pub fn trace(&self, ray: &Ray, stop_on_first_hit: bool, for_shadow: bool) -> Option<(f32, Vector3<f32>, &dyn Shape, u32)>
    {
        //find hits (bbox based)
        let mut hits: Vec<HitResult> = vec![];
        for item in &self.scene.items
        {
            let dist = item.intersect_b_box(&ray);
            if let Some(dist) = dist
            {
                if item.get_material().alpha > 0.0 && (!for_shadow || item.get_material().cast_shadow)
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

    pub fn get_tex_color(&self, item: &dyn Shape, hit_point: Point3<f32>, face_id: u32, tex_type: TextureType) -> Option<Vector3<f32>>
    {
        //texture
        if (*item).get_basic().has_texture(tex_type)
        {
            let uv = (*item).get_uv(hit_point, face_id);

            let tex_dims = (*item).get_basic().texture_dimension(tex_type);
            let tex_x = self.wrap(uv.x, tex_dims.0);
            let tex_y = self.wrap(uv.y, tex_dims.1);

            let tex_color = (*item).get_basic().get_texture_pixel(tex_x, tex_y, tex_type);

            return Some(tex_color);
        }

        None
    }

    pub fn get_item_color(&self, item: &dyn Shape, hit_point: Point3<f32>, face_id: u32, color_type: LightningColorType) -> Vector3<f32>
    {
        let mat = (*item).get_material();

        let mut item_color;
        let tex_type;
        match color_type
        {
            LightningColorType::Ambient =>
            {
                item_color = mat.ambient_color;
                tex_type = TextureType::Ambient;
            },
            LightningColorType::Diffuse =>
            {
                item_color = mat.diffuse_color;
                tex_type = TextureType::Diffuse;
            },
            LightningColorType::Specular =>
            {
                item_color = mat.specular_color;
                tex_type = TextureType::Specular;
            },
        }

        //texture color
        let tex_color = self.get_tex_color(item, hit_point, face_id, tex_type);

        if let Some(tex_color) = tex_color
        {
            item_color.x *= tex_color.x;
            item_color.y *= tex_color.y;
            item_color.z *= tex_color.z;
        }

        item_color
    }

    pub fn get_color(&self, ray: Ray, depth: u16) -> Vector3<f32>
    {
        //TODO:
        let eye_dir = Vector3::<f32>::new(0.0, 0.0, 0.0);

        let mut r = ray;
        r.dir = r.dir.normalize();

        //intersect
        let intersection = self.trace(&r, false, false);

        let mut color = Vector3::new(0.0, 0.0, 0.0);

        if let Some(intersection) = intersection
        {
            let hit_dist = intersection.0;
            let normal = intersection.1;
            let item = intersection.2;
            let face_id = intersection.3;

            let mut surface_normal = normal;
            let hit_point = r.origin + (r.dir * hit_dist);

            //normal mapping
            let normal_tex_color = self.get_tex_color(item, hit_point, face_id, TextureType::Normal);
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
                let mut normal_map = normal_tex_color;
                normal_map.x = (normal_map.x * 2.0) - 1.0;
                normal_map.y = (normal_map.y * 2.0) - 1.0;
                normal_map.z = (normal_map.z * 2.0) - 1.0;

                normal_map.x *= item.get_material().normal_map_strength;
                normal_map.y *= item.get_material().normal_map_strength;

                normal_map = normal_map.normalize();

                let tbn = Matrix3::<f32>::from_columns(&[tangent, bitangent, normal]);

                surface_normal = (tbn * normal_map).normalize();
            }

            //alpha mapping
            let mut alpha = item.get_material().alpha;
            let alpha_tex_color = self.get_tex_color(item, hit_point, face_id, TextureType::Alpha);
            if let Some(alpha_tex_color) = alpha_tex_color
            {
                alpha *= alpha_tex_color.x;
            }

            //ambient, diffuse, specular colors
            let ambient_color = self.get_item_color(item, hit_point, face_id, LightningColorType::Ambient);
            let diffuse_color = self.get_item_color(item, hit_point, face_id, LightningColorType::Diffuse);
            let specular_color = self.get_item_color(item, hit_point, face_id, LightningColorType::Specular);

            //ambient
            color = ambient_color;

            //diffuse/specular color
            for light in &self.scene.lights
            {
                //get direction to light based on light type
                let direction_to_light;

                match light.light_type
                {
                    LightType::Directional => direction_to_light = (-light.dir).normalize(),
                    LightType::Point => direction_to_light = (light.pos - hit_point).normalize(),
                }

                //lambert shading
                let dot_light = surface_normal.dot(&direction_to_light).max(0.0);

			    let diffuse = diffuse_color * dot_light;

                //phong shading
                let h = (eye_dir + direction_to_light).normalize();
                let dot_viewer = h.dot(&surface_normal).max(0.0);

                let light_power = dot_viewer.powf(item.get_material().shininess);
			    let specular = specular_color * light_power;

                //light intensity
                let mut intensity;
                match light.light_type
                {
                    LightType::Directional => intensity = light.intensity,
                    LightType::Point =>
                    {
                        let r2 = (light.pos - hit_point).norm() as f32;
                        intensity = light.intensity / (4.0 * ::std::f32::consts::PI * r2)
                    }
                }

                //shadow
                if item.get_material().receive_shadow
                {
                    let shadow_ray_start = hit_point + (surface_normal * SHADOW_BIAS);
                    let shadow_ray = Ray::new(shadow_ray_start, direction_to_light);
                    let shadow_intersection = self.trace(&shadow_ray, true, true);

                    let mut in_light = shadow_intersection.is_none();
                    if !in_light && light.light_type == LightType::Point
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

                        let shadow_alpha_tex_color = self.get_tex_color(shadow_obj, shadow_hit_point, shadow_face_id, TextureType::Alpha);
                        if let Some(shadow_alpha_tex_color) = shadow_alpha_tex_color
                        {
                            shadow_source_alpha *= shadow_alpha_tex_color.x;
                        }

                        intensity = intensity * (1.0 - shadow_source_alpha);
                    }
                }

                //color based on components
                color.x = color.x + ((light.color.x * (specular.x + diffuse.x)) * intensity);
                color.y = color.y + ((light.color.y * (specular.y + diffuse.y)) * intensity);
                color.z = color.z + ((light.color.z * (specular.z + diffuse.z)) * intensity);
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
        }

        color
    }
}