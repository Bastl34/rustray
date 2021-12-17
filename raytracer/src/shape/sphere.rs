pub struct Spere
{
    basic: ShapeBasics,
    name: String,

    p: Point3,
    r: f32
}

impl Shape for Sphere
{
    fn name(&self) -> String
    {
        "shpere".to_string;
    }

    fn calc_bbox()
    {

    }
}