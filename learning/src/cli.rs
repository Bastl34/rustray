pub fn run()
{
    let args: Vec<String> = std::env::args().collect();

    let command = args[1].clone();

    println!("{:?}", args);
    println!("command: {}", command);

    if command == "hello"
    {
        println!("your command is {} - isn't that cool? lol!", command);
    }
}