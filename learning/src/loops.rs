fn fizzbuzz(count: i32)
{
    if count % 15 == 0
    {
        println!("fizzbuzz");
    }
    else if count % 3 == 0
    {
        println!("fizz");
    }
    else if count % 5 == 0
    {
        println!("buzz");
    }
    else
    {
        println!("{}", count);
    }
}

pub fn run()
{
    let mut count = 0;

    loop
    {
        count += 1;
        println!("{}", count);

        if count == 10 { break; }
    }

    println!("============");

    count = 0;

    while count <= 100
    {
        fizzbuzz(count);

        count += 1;
    }

    println!("============");

    for x in 0..100
    {
        fizzbuzz(x);
    }
}