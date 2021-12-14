
fn thread_func(origin: &str)
{
    for i in 1..10
    {
        println!("{}: i={}", origin, i);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

pub fn run()
{
    let t = std::thread::spawn(|| { thread_func("spawned thread"); });

    thread_func("main thread");

    t.join().unwrap();


    let v = vec![1,2,3];
    let handle = std::thread::spawn(move ||
    {
        println!("Here's a vector: {:?}", v);
    });

    handle.join().unwrap();


    // ********** mutex **********

    let m = std::sync::Mutex::new(5);

    {
        let mut num = m.lock().unwrap();
        *num = 6;
    }

    println!("{:?}", m);

    // ********** mutex **********

    let mut threads = vec![];
    let counter = std::sync::Arc::new(std::sync::Mutex::new(0));

    for _ in 0..10
    {
        let c_copy = std::sync::Arc::clone(&counter);
        let thread = std::thread::spawn( move ||
        {
            let mut number = c_copy.lock().unwrap();
            *number += 10;
        });

        threads.push(thread);
    }

    for thread in threads
    {
        thread.join().unwrap();
    }

    println!("{}", *(counter.lock().unwrap()));
}