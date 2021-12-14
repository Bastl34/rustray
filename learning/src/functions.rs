pub fn run()
{
    greeting("lol", "lol2".to_string());

    let sum = add(1,2);
    println!("{}", sum);

    let lol = 10;
    let add_nums_lol = |n1: i32, n2: i32| n1 + n2 + lol;
    println!("{}", add_nums_lol(1,2));

    let mut string = String::from("tada");
    string = ownership_test(string);
    println!("{}", string);

    let mut string2 = String::from("lol");
    reference_value(&mut string2);
    println!("{}", string2);

    let mut string3 = String::from("bla bla bla");
    some_string_change2(&mut string3);
    let string4 = some_string_change3(&mut string3);

    println!("{}", string4);
}

fn greeting(greet: &str, name: String)
{
    println!("greetings to {}", greet);
    println!("greetings to {}", name);
}

fn add(n1: i32, n2: i32) -> i32
{
    n1 + n2
}

fn ownership_test(test: String) -> String
{
    return test;
}

fn reference_value(test: &mut String)
{
    //test = test + " lol";
    test.push_str("lol");

    *test += "öasdasd";
}

fn some_string_change2(str: &mut String)
{
    str.push_str("lol lol lol");
}

fn some_string_change3(str: &mut String) ->&mut String
{
    str.push_str("lol lol lol");

    return str;
}