use parry3d::shape::Ball;

pub struct Spere
{
    basic: ShapeBasics,
    name: String,

    ball: Ball
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

impl Sphere
{
    pub fn new(r: i32) -> Sphere
    {
        Spere
        {
            basic: ShapeBasics::new(),
            name: String::from("Sphere"),
            ball: Ball::new(1.0f32)
        }
    }
}