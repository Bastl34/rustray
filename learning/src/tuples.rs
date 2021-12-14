pub fn run()
{
    let person: (&str, &str, i8) = ("lol", "lol2", 10);
    let person2 = ("test", "test", 10, 1034234);

    println!("0: {}, 1: {}, 2: {}", person.0, person.1, person.2);
    println!("{:?}", person);
    println!("{:?}", person2);
}