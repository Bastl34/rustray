fn divide(a: f32, b: f32) -> Option<f32>
{
    if b == 0.0
    {
        None
    }
    else
    {
        Some(a/b)
    }
}

pub fn run()
{
    let res = divide(1.0, 1.0);
    if let Some(res) = res
    {
        println!("{}", res);
    }
    else
    {
        println!("division not possible");
    }

    match res
    {
        Some(res) => println!("bla bla: {}", res),
        _ => (),
    }
}