pub fn run()
{
    println!("hello bla bla");
    println!("Test: {} {}", 1, 2);

    println!("{0} {0} {1} {2}", 1, 2, 3);

    println!("{test} {test2}", test = "lol", test2 = "lol2");

    println!("Bin: {:b}, Hex: {:x}, Oct: {:0}", 10, 10, 10);

    println!("{:?}", (12, true, 1, "test"));

    println!("{}", 10 + 10);
}