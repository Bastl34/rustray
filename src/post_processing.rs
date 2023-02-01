use image::{RgbaImage, ImageBuffer, Rgba};
use nalgebra::{Vector3, Vector2, Matrix4, Vector4};

use crate::camera::Camera;

#[derive(Debug, Copy, Clone)]
pub struct PostProcessingConfig
{
    pub cavity: bool,
    pub outline: bool,
}

impl PostProcessingConfig
{
    pub fn new() -> PostProcessingConfig
    {
        PostProcessingConfig
        {
            cavity: false,
            outline: false,
        }
    }
}

pub fn curvature_soft_clamp(curvature: f32, control: f32) -> f32
{
    if curvature < 0.5 / control
    {
        return curvature * (1.0 - curvature * control);
    }

    0.25 / control
}

pub fn texel_fetch_offset_vec3(sampler: &Vec<Vector3<f32>>, texel: Vector2<i32>, offset: Vector2<i32>, width: i32) -> Vector3<f32>
{
    let x = texel.x + offset.x;
    let y = texel.y + offset.y;

    let index = y * width + x;

    if index < 0 || index as usize >= sampler.len()
    {
        return Vector3::<f32>::zeros();
    }

    sampler[index as usize]
}

pub fn texel_fetch_offset_u32(sampler: &Vec<u32>, texel: Vector2<i32>, offset: Vector2<i32>, width: i32) -> u32
{
    let x = texel.x + offset.x;
    let y = texel.y + offset.y;

    let index = y * width + x;

    if index < 0 || index as usize >= sampler.len()
    {
        return 0;
    }

    sampler[index as usize]
}

pub fn get_view_space_from_depth(pos: Vector2<i32>, depth: f32, projection_inverse: &Matrix4<f32>) -> Vector3<f32>
{
    let depth = (depth * 2.0) - 1.0;
    let x = (pos.x as f32 * 2.0) - 1.0;
    let y = (pos.y as f32 * 2.0) - 1.0;

    let mut vec = Vector4::<f32>::new(x, y, depth, 1.0);

    vec = projection_inverse * vec;
    vec.xyz() / vec.w
}

pub fn calculate_curvature(texel: Vector2<i32>, normals: &Vec<Vector3<f32>>, width: i32, ridge: f32, valley: f32) -> f32
{
    // https://playground.babylonjs.com/#YF8D42#1
    // https://developer.blender.org/diffusion/B/browse/master/source/blender/draw/engines/workbench/shaders/workbench_curvature_lib.glsl

    let normal_up = texel_fetch_offset_vec3(normals, texel, Vector2::<i32>::new(0, 1), width).xz();
    let normal_down = texel_fetch_offset_vec3(normals, texel, Vector2::<i32>::new(0, -1), width).xz();
    let normal_left = texel_fetch_offset_vec3(normals, texel, Vector2::<i32>::new(-1, 0), width).xz();
    let normal_right = texel_fetch_offset_vec3(normals, texel, Vector2::<i32>::new(1, 0), width).xz();

    let normal_diff = (normal_up.y - normal_down.y) + (normal_right.x - normal_left.x);

    if normal_diff < 0.0
    {
        return -2.0 * curvature_soft_clamp(-normal_diff, valley);
    }

    2.0 * curvature_soft_clamp(normal_diff, ridge)
}

pub fn calculate_outline(texel: Vector2<i32>, object_ids: &Vec<u32>, width: i32) -> f32
{
    // https://developer.blender.org/diffusion/B/browse/master/source/blender/draw/engines/workbench/shaders/workbench_effect_outline_frag.glsl
    let center_id = texel_fetch_offset_u32(object_ids, texel, Vector2::<i32>::zeros(), width);

    let object_up = texel_fetch_offset_u32(object_ids, texel, Vector2::<i32>::new(0, 1), width);
    let object_down = texel_fetch_offset_u32(object_ids, texel, Vector2::<i32>::new(0, -1), width);
    let object_right = texel_fetch_offset_u32(object_ids, texel, Vector2::<i32>::new(-1, 0), width);
    let object_left= texel_fetch_offset_u32(object_ids, texel, Vector2::<i32>::new(1, 0), width);

    let adjacent_ids = Vector4::<u32>::new(object_up, object_down, object_right, object_left);

    let vec_025 = Vector4::<f32>::new(0.25, 0.25, 0.25, 0.25);

    let eq0 = if adjacent_ids.x == center_id { 1.0 } else { 0.0 };
    let eq1 = if adjacent_ids.y == center_id { 1.0 } else { 0.0 };
    let eq2 = if adjacent_ids.z == center_id { 1.0 } else { 0.0 };
    let eq3 = if adjacent_ids.w == center_id { 1.0 } else { 0.0 };

    let equal_vec = Vector4::<f32>::new(eq0, eq1, eq2, eq3);

    let outline_opacity = 1.0 - equal_vec.dot(&vec_025);

    outline_opacity
}

pub fn run_post_processing(config: PostProcessingConfig, image: &RgbaImage, normals: &Vec<Vector3<f32>>, _depth: &Vec<f32>, object_ids: &Vec<u32>, _cam: &Camera) -> RgbaImage
{
    let width = image.width();
    let height = image.height();
    let mut processed_image: RgbaImage = ImageBuffer::new(width, height);

    let ridge = 1.15;
    let valley = 1.0;
    let outline_color = Vector3::<f32>::new(255.0, 255.0, 255.0);

    for x in 0..width
    {
        for y in 0..height
        {
            let pixel = image.get_pixel(x, y);

            let mut r = pixel[0] as f32;
            let mut g = pixel[1] as f32;
            let mut b = pixel[2] as f32;

            let x = x as usize;
            let y = y as usize;

            //let normal = normals[y * w + x];
            //let depth = depth[y * w + x];

            let texel = Vector2::<i32>::new(x as i32, y as i32);

            if config.outline
            {
                let outline = calculate_outline(texel, object_ids, width as i32);

                if outline > 0.0
                {
                    r *= outline * outline_color.x;
                    g *= outline * outline_color.y;
                    b *= outline * outline_color.z;
                }
            }

            if config.cavity
            {
                let curvature = calculate_curvature(texel, normals, width as i32, ridge, valley);
                r *= curvature + 1.0;
                g *= curvature + 1.0;
                b *= curvature + 1.0;
            }

            r = r.clamp(0.0, 255.0);
            g = g.clamp(0.0, 255.0);
            b = b.clamp(0.0, 255.0);

            let color = Rgba([r as u8, g as u8, b as u8, 255]);
            processed_image.put_pixel(x as u32, y as u32, color);
        }
    }

    processed_image
}