pub fn run()
{
    let name = "Test";
    let mut age = 10;


    println!("name: {}, age: {}", name, age);

    age = 20;

    println!("name: {}, age: {}", name, age);

    const ID: i32 = 1;

    println!("ID: {}", ID);

    let (test1, test2) = ("lol", "lol2");

    println!("{} {}", test1, test2);
}