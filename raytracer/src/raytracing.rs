use crate::shape::Shape;
use crate::pixel_color::PixelColor;

use crate::shape::sphere::Sphere;

use crate::scene::Scene;

use nalgebra::{Perspective3, Isometry3, Point3, Vector3};
use parry3d::query::{Ray};

pub struct HitResult<'a>
{
    item: &'a dyn Shape,
    dist: f32,
}

pub struct Raytracing
{
    scene: Scene,

    width: u32,
    height: u32,

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

            fov: 0.0,
            fov_adjustment: 0.0,

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

    pub fn trace(&self, ray: &Ray, stop_on_first_hit: bool) -> Option<(f32, Vector3<f32>, &dyn Shape)>
    {
        //find hits (bbox based)
        let mut hits: Vec<HitResult> = vec![];
        for item in &self.scene.items
        {
            let dist = item.intersect_b_box(&ray);
            if let Some(dist) = dist
            {
                hits.push(HitResult{ item: item.as_ref(), dist: dist });
            }
        }

        if hits.len() == 0 
        {
            return None;
        }

        //sort bbox dist (to get the nearest)
        hits.sort_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap());

        let mut best_hit: Option<(f32, Vector3<f32>, & dyn Shape)> = None;

        for item in hits
        {
            let intersection = item.item.intersect(&ray);

            if let Some(intersection) = intersection
            {
                if best_hit.is_none() || best_hit.is_some() && intersection.0 < best_hit.unwrap().0
                {
                    best_hit = Some((intersection.0, intersection.1, item.item));
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

    pub fn render(&self, x: i32, y: i32) -> PixelColor
    {
        let light_dir = Vector3::new(1.0f32, -1.0, -1.0).normalize();    
        let light_color = Vector3::new(1.0f32, 1.0, 1.0);
        let light_intensity = 100.0f32;

        let shadow_bias = 0.0001;

        //map x/y to -1 <=> +1
        let sensor_x = ((((x as f32 + 0.5) / self.width as f32) * 2.0 - 1.0) * self.aspect_ratio) * self.fov_adjustment;
        let sensor_y = (1.0 - ((y as f32 + 0.5) / self.height as f32) * 2.0) * self.fov_adjustment;

        let ray = Ray::new(Point3::origin(), Vector3::new(sensor_x, sensor_y, -1.0));

        let intersection = self.trace(&ray, false);

        let mut r = 255;
        let mut g = 0;
        let mut b = 255;

        if let Some(intersection) = intersection
        {
            let hit_dist = intersection.0;
            let normal = intersection.1;
            let item = intersection.2;

            let surface_normal = normal;
            let direction_to_light = (-light_dir).normalize();

            let hit_point = ray.origin + (ray.dir * hit_dist);

            let shadow_ray_start = hit_point + (surface_normal * shadow_bias);
            let shadow_ray = Ray::new(shadow_ray_start, direction_to_light);
            let in_light = self.trace(&shadow_ray, true).is_none();

            //let hit_point = ray.origin + (ray.dir * intersection.0);


            //let light_power = surface_normal.dot(&direction_to_light).max(0.0) * light_intensity;
            //let dot_light = surface_normal.dot(&direction_to_light).max(0.0);
            let dot_light = surface_normal.dot(&direction_to_light).max(0.0);
            //let light_power = surface_normal.dot(&direction_to_light).max(0.0) * light_intensity;
            let light_power = dot_light.powf(item.get_material().shininess);
            //let light_reflected = item.item.get_material().shininess / std::f32::consts::PI;
            //let light_reflected = 1.58 / std::f32::consts::PI;

            let intensity = if in_light { light_intensity } else { 0.0 };

            let item_color = (*item).get_material().anmbient_color;
            let item_light_color = Vector3::new(item_color.x * light_color.x, item_color.y * light_color.y, item_color.z * light_color.z);

            //let color = light_color.cross(&item_color) * light_power * light_intensity;
            //let color = item_color.cross(&light_color) * light_power * light_reflected;
            //let color = item_color.cross(&light_color) * light_power * light_intensity;
            //let test = Vector3::new(item_color.x * light_color.x, item_color.y * light_color.y, item_color.z * light_color.z);
            //let color = test * light_power * light_intensity;

            
            let color = item_light_color * light_power * intensity;

            //todo: clamp

            //let color = (*item.item).get_material().anmbient_color;

            let r_float = color.x * 255.0;
            let g_float = color.y * 255.0;
            let b_float = color.z * 255.0;


            //TODO: alpha blending
            r = r_float as u8;
            g = g_float as u8;
            b = b_float as u8;

        }

        PixelColor { r: r, g: g, b: b, x: x, y: y }
    }
}