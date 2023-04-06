use nalgebra::{Matrix4, Perspective3, Point3, Isometry3, Vector3};

use crate::helper::approx_equal;

const DEFAULT_CAM_POS: Point3::<f32> = Point3::<f32>::new(0.0, 0.0, 0.0);
const DEFAULT_CAM_UP: Vector3::<f32> = Vector3::<f32>::new(0.0, 1.0, 0.0);
const DEFAULT_CAM_DIR: Vector3::<f32> = Vector3::<f32>::new(0.0, 0.0, -1.0);

//pub const OBLIQUE_CAM_POS: Vector3::<f32> = Vector3::<f32>::new(1.0, 0.0, 2.0);
pub const OBLIQUE_CAM_POS: Vector3::<f32> = Vector3::<f32>::new(1.0, 0.5, 1.0);

pub const DEFAULT_FOV: f32 = 90.0f32;

const DEFAULT_CLIPPING_NEAR: f32 = 0.001;
const DEFAULT_CLIPPING_FAR: f32 = 1000.0;

#[derive(Debug, Copy, Clone)]
pub struct Camera
{
    pub width: u32,
    pub height: u32,
    pub aspect_ratio: f32,

    pub fov: f32,

    pub eye_pos: Point3::<f32>,

    pub up: Vector3::<f32>,
    pub dir: Vector3::<f32>,

    pub clipping_near: f32,
    pub clipping_far: f32,

    pub projection: Perspective3<f32>,
    pub view: Matrix4<f32>,

    pub projection_inverse: Matrix4<f32>,
    pub view_inverse: Matrix4<f32>,
}

impl Camera
{
    pub fn new() -> Camera
    {
        Camera
        {
            width: 0,
            height: 0,
            aspect_ratio: 0.0,

            fov: DEFAULT_FOV.to_radians(),

            eye_pos: DEFAULT_CAM_POS,

            up: DEFAULT_CAM_UP,
            dir: DEFAULT_CAM_DIR,

            clipping_near: DEFAULT_CLIPPING_NEAR,
            clipping_far: DEFAULT_CLIPPING_FAR,

            projection: Perspective3::<f32>::new(1.0f32, 0.0f32, DEFAULT_CLIPPING_NEAR, DEFAULT_CLIPPING_FAR),
            view: Matrix4::<f32>::identity(),

            projection_inverse: Matrix4::<f32>::identity(),
            view_inverse: Matrix4::<f32>::identity(),
        }
    }

    pub fn init(&mut self, width: u32, height: u32)
    {
        self.width = width;
        self.height = height;

        self.aspect_ratio = width as f32 / height as f32;

        self.init_matrices();
    }

    pub fn init_matrices(&mut self)
    {
        self.projection = Perspective3::new(self.aspect_ratio, self.fov, self.clipping_near, self.clipping_far);

        //let target = Point3::<f32>::new(self.dir.x, self.dir.y, self.dir.z);
        let target = self.eye_pos + self.dir;

        self.view = Isometry3::look_at_rh(&self.eye_pos, &target, &self.up).to_homogeneous();

        self.projection_inverse = self.projection.inverse();
        self.view_inverse = self.view.try_inverse().unwrap();
    }

    pub fn is_default_cam(&self) -> bool
    {
        (
            approx_equal(self.eye_pos.x, DEFAULT_CAM_POS.x)
            &&
            approx_equal(self.eye_pos.y, DEFAULT_CAM_POS.y)
            &&
            approx_equal(self.eye_pos.z, DEFAULT_CAM_POS.z)
        )
        &&
        (
            approx_equal(self.dir.x, DEFAULT_CAM_DIR.x)
            &&
            approx_equal(self.dir.y, DEFAULT_CAM_DIR.y)
            &&
            approx_equal(self.dir.z, DEFAULT_CAM_DIR.z)
        )
        &&
        (
            approx_equal(self.up.x, DEFAULT_CAM_UP.x)
            &&
            approx_equal(self.up.y, DEFAULT_CAM_UP.y)
            &&
            approx_equal(self.up.z, DEFAULT_CAM_UP.z)
        )
        &&
        approx_equal(self.fov, DEFAULT_FOV.to_radians())
        &&
        approx_equal(self.clipping_near, DEFAULT_CLIPPING_NEAR)
        &&
        approx_equal(self.clipping_far, DEFAULT_CLIPPING_FAR)
    }

    pub fn set_cam_position(&mut self, eye_pos: Point3::<f32>, dir: Vector3::<f32>)
    {
        self.eye_pos = eye_pos;
        self.dir = dir;

        self.init_matrices();
    }

    pub fn is_point_in_frustum(&self, point: &Point3<f32>) -> bool
    {
        let pv = self.projection.to_homogeneous() * self.view;
        let point_clip = pv * point.to_homogeneous();

        // Check if point is inside NDC space (Normalized Device Coordinates Space)
        point_clip.x.abs() <= point_clip.w && point_clip.y.abs() <= point_clip.w && point_clip.z.abs() <= point_clip.w
    }

    pub fn print(&self)
    {
        println!("width: {:?}", self.width);
        println!("height: {:?}", self.height);
        println!("aspect_ratio: {:?}", self.aspect_ratio);

        println!("fov: {:?}", self.fov);

        println!("eye_pos: {:?}", self.eye_pos);

        println!("up: {:?}", self.up);
        println!("dir: {:?}", self.dir);

        println!("clipping_near: {:?}", self.clipping_near);
        println!("clipping_far: {:?}", self.clipping_far);

        println!("projection: {:?}", self.projection);
        println!("view: {:?}", self.view);
    }
}