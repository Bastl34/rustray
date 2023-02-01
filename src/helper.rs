use nalgebra::Vector3;
use rand::Rng;

use std::fs::File;

pub fn rand<T: std::cmp::PartialOrd + rand::distributions::uniform::SampleUniform>(from: T, to: T) -> T
{
    rand::thread_rng().gen_range(from..to) as T
}

pub fn approx_equal(a: f32, b: f32) -> bool
{
    let decimal_places = 6;

    let factor = 10.0f32.powi(decimal_places as i32);
    let a = (a * factor).trunc();
    let b = (b * factor).trunc();

    a == b
}

pub fn download(url: &str, local_path: &str) -> attohttpc::Result
{
    let resp = attohttpc::get(url).send()?;

    if resp.is_success()
    {
        let file = File::create(local_path)?;
        resp.write_to(file)?;
    }

    Ok(())
}

pub fn interpolate(a: f32, b: f32, f: f32) -> f32
{
    return a + f * (b - a);
}

// https://stackoverflow.com/a/16544330
pub fn plane_based_vector_angle(a1: Vector3<f32>, a2: Vector3<f32>) -> f32
{
    // get the normal of the plane
    let n = a1.cross(&a2);

    let dot = a1.x*a2.x + a1.y*a2.y + a1.z*a2.z;
    let det = a1.x*a2.y*n.z + a2.x*n.y*a1.z + n.x*a1.y*a2.z - a1.z*a2.y*n.x - a2.z*n.y*a1.x - n.z*a1.y*a2.x;

    det.atan2(dot)
}