pub fn run()
{
    let mut numbers: Vec<i32> = vec![1,2,3,4];

    println!("{:?}", numbers);
    println!("{:?}", numbers[0]);

    numbers[2] = 10;


    numbers.push(5);
    numbers.push(6);
    numbers.push(6);
    numbers.pop();

    for i in 0..10
    {
        numbers.push(i);
    }

    println!("{:?}", numbers[2]);

    //dbg!(numbers.len());
    println!("{}", numbers.len());

    dbg!(numbers[0]);

    println!("array occupies {} bytes", std::mem::size_of_val(&numbers));

    //slices
    let slice: &[i32] = &numbers;
    println!("{:?}", slice);

    let slice2: &[i32] = &numbers[0..2];
    println!("{:?}", slice2);

    for v in &numbers
    {
        println!("{}", v);
    }

    //muate stuff
    for v in &mut numbers
    {
        *v *= 2;
        println!("{}", v);
    }
}