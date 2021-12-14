pub fn run()
{
    let mut numbers: [i32; 4] = [1,2,3,4];

    println!("{:?}", numbers);
    println!("{:?}", numbers[0]);

    numbers[2] = 10;

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
}