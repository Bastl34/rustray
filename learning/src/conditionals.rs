pub fn run()
{
    let age = 20;
    let check_id = false;

    if age > 10 && check_id
    {
        println!("lol");
    }
    else if age < 19
    {
        println!("lol2");
    }
    else
    {
        println!("lol3");
    }

    let bla = if check_id { true } else { false };

    println!("{}", bla);
}