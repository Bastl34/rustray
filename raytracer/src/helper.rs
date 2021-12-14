use rand::Rng;

pub fn rand<T: std::cmp::PartialOrd + rand::distributions::uniform::SampleUniform>(from: T, to: T) -> T
{
    rand::thread_rng().gen_range(from..to) as T
}