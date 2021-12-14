pub fn run()
{
    let hello = "test";
    let mut hello2 = String::from("lalala");

    hello2 = hello2 + "lol" + "sad";

    hello2.push(' ');
    hello2.push_str(" lo lo l o l ll");


    println!("{:?}", (hello));

    println!("String Length: {}", hello2.len());
    println!("String capacity: {}", hello2.capacity());
    println!("String empty: {}", hello2.is_empty());
    println!("String contains lol: {}", hello2.contains("lol"));
    println!("String contains test: {}", hello2.contains("test"));
    println!("String replace: {}", hello2.replace("lol", "what"));

    println!("{:?}", hello2);

    println!("========");
    //for word in hello2.split(" ")
    for word in hello2.split_whitespace()
    {
        println!("{}", word);
    }

    println!("==========");

    let mut s = String::with_capacity(10);
    s.push('a');
    s.push('b');

    assert_eq!(s.len(), 2);
    assert_eq!(s.capacity(), 10);

    println!("{}", s);


    // slices
    let s = String::from("hello world");

    let hello = &s[0..5];
    let world = &s[6..11];

    println!("{} {}", hello, world);


    //find first word
    let test_string = String::from("hallo bla bla bla");
    let forst_word = find_first_word(&test_string);

    println!("first word is: {}", forst_word);


    //utf8
    let mut utf8_test = "✅🦄".to_string();
    utf8_test.push('🤖');
    println!("{}", utf8_test);


    //more tests
    let s1 = String::from("Hello, ");
    let s2 = String::from("world!");
    let s3 = s1.clone() + &s2; // note s1 has been moved here and can no longer be used

    println!("{} {} {}", s1, s2, s3);


    for c in "नमस्ते".chars() {
        println!("{}", c);
    }
}

fn find_first_word(str: &str) -> &str
{
    let bytes = str.as_bytes();

    for (i, &item) in bytes.iter().enumerate()
    {
        if item == b' '
        {
            return &str[..i];
        }
    }

    &str[..]
}