use crate::shape::Shape;
use crate::pixel_color::PixelColor;

use crate::shape::sphere::Sphere;

use crate::scene::{Scene, LightType};

use nalgebra::{Perspective3, Isometry3, Point3, Vector3};
use parry3d::query::{Ray};

const SHADOW_BIAS: f32 = 0.0001;

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
        //map x/y to -1 <=> +1
        let sensor_x = ((((x as f32 + 0.5) / self.width as f32) * 2.0 - 1.0) * self.aspect_ratio) * self.fov_adjustment;
        let sensor_y = (1.0 - ((y as f32 + 0.5) / self.height as f32) * 2.0) * self.fov_adjustment;

        //create ray
        let ray = Ray::new(Point3::origin(), Vector3::new(sensor_x, sensor_y, -1.0));

        //intersect
        let intersection = self.trace(&ray, false);

        let mut color = Vector3::new(0.0, 0.0, 0.0);

        if let Some(intersection) = intersection
        {
            let hit_dist = intersection.0;
            let normal = intersection.1;
            let item = intersection.2;

            let surface_normal = normal;
            let hit_point = ray.origin + (ray.dir * hit_dist);


            for light in &self.scene.lights
            {
                //get direction to light based on light type
                let direction_to_light;

                match light.light_type
                {
                    LightType::directional => direction_to_light = (-light.dir).normalize(),
                    LightType::point => direction_to_light = (light.pos - hit_point).normalize(),
                }
                
    
                let shadow_ray_start = hit_point + (surface_normal * SHADOW_BIAS);
                let shadow_ray = Ray::new(shadow_ray_start, direction_to_light);
                let shadow_intersection = self.trace(&shadow_ray, true);

                let mut in_light = shadow_intersection.is_none();
                if !in_light && light.light_type == LightType::point
                {
                    let light_dist: Vector3<f32> = light.pos - hit_point;
                    let len = light_dist.norm();
                    
                    in_light = shadow_intersection.unwrap().0 > len
                }
    
                //let hit_point = ray.origin + (ray.dir * intersection.0);
    
    
                //let light_power = surface_normal.dot(&direction_to_light).max(0.0) * light_intensity;
                //let dot_light = surface_normal.dot(&direction_to_light).max(0.0);
                let dot_light = surface_normal.dot(&direction_to_light).max(0.0);
                //let light_power = surface_normal.dot(&direction_to_light).max(0.0) * light_intensity;
                let light_power = dot_light.powf(item.get_material().shininess);
                //let light_reflected = item.item.get_material().shininess / std::f32::consts::PI;
                //let light_reflected = 1.58 / std::f32::consts::PI;
    
                let intensity = if in_light
                {
                    match light.light_type
                    {
                        LightType::directional => light.intensity,
                        LightType::point => 
                        {
                            let r2 = (light.pos - hit_point).norm() as f32;
                            light.intensity / (4.0 * ::std::f32::consts::PI * r2)
                        }
                    }
                }
                else
                {
                    0.0
                };
    
                let item_color = (*item).get_material().anmbient_color;
                let item_light_color = Vector3::new(item_color.x * light.color.x, item_color.y * light.color.y, item_color.z * light.color.z);
    
    
                color = color + (item_light_color * light_power * intensity);

            }
        }

        //TODO: alpha blending + clamp
        let r = (color.x * 255.0) as u8;
        let g = (color.y * 255.0) as u8;
        let b = (color.z * 255.0) as u8;

        PixelColor { r: r, g: g, b: b, x: x, y: y }
    }
}