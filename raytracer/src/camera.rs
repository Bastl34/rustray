use nalgebra::{Matrix4, Perspective3, Point3, Isometry3, Vector3};

#[derive(Debug)]
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

            fov: 90.0f32.to_radians(),

            eye_pos: Point3::new(0.0, 0.0, 0.0),

            up: Vector3::y(),
            dir: Vector3::new(0.0, 0.0, -1.0),

            clipping_near: 0.001,
            clipping_far: 1000.0,

            projection: Perspective3::<f32>::new(1.0f32, 0.0f32, 0.001, 1000.0),
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