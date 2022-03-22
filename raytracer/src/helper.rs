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