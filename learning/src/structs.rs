#[derive(Debug)]
struct Color
{
    red: u8,
    green: u8,
    blue: u8
}

struct ColorStruct(u8, u8, u8);

struct Person
{
    first_name: String,
    last_name: String
}

impl Person
{
    fn new(first: &str, last: &str) -> Person
    {
        Person { first_name: first.to_string(), last_name: last.to_string() }
    }

    fn full_name(&self) -> String
    {
        format!("{} {}", self.first_name, self.last_name)
    }

    fn set_last_name(&mut self, last: &str)
    {
        self.last_name = last.to_string();
    }

    fn to_tuple(self) -> (String, String)
    {
        (self.first_name, self.last_name)
    }
}

pub fn run()
{
    let mut c = Color {red: 255, green: 0, blue: 0};
    c.red = 200;
    println!("red: {} green: {} blue: {}", c.red, c.green, c.blue);

    let mut c2 = ColorStruct(255,0,0);
    c2.0 = 200;
    println!("red: {} green: {} blue: {}", c2.0, c2.1, c2.2);

    let mut p = Person::new("John", "Wick");
    println!("Firstname: {} Lastname {}", p.first_name, p.last_name);
    println!("{}", p.full_name());

    p.set_last_name("WickyWick");

    println!("{}", p.full_name());

    println!("{:?}", p.to_tuple());


    //some more tests

    let my_color = Color
    {
        red: 1,
        green: 2,
        blue: 3
    };

    let my_new_color = Color
    {
        blue: 1,
        ..my_color
    };

    println!("{:?}", my_color);
    println!("{:?}", my_new_color);

    dbg!(&my_new_color);
    dbg!(&my_new_color);
}