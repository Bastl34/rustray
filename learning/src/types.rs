pub fn run()
{
    //detault i32
    let x = 1;

    // default f64
    let y = 0.5;

    let bla: i64 =  1022;

    let i_max: i32 = std::i32::MAX;
    let i_64_max: i64 = std::i64::MAX;

    //bool
    let test_bool: bool = true;

    println!("{:?}", (x, y, bla, i_max, i_64_max, test_bool));

    //get bool from expression
    let is_greater = 10 > 5;

    println!("{:?}", (is_greater));

    let char_test = 'a';
    let char_test2 = '\u{1F600}';
    let char_test3 = '✅';

    println!("{:?}", (char_test, char_test2, char_test3));
}