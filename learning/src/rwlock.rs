use std::{thread, time::Duration};
use std::sync::{Arc, RwLock};

struct Test
{
    some_val: i32,
}

impl Test
{
    pub fn new() -> Test
    {
        Test { some_val: 10 }
    }

    pub fn print(&self)
    {
        println!("{}",self.some_val);
    }

    pub fn update(&mut self, val: i32)
    {
        self.some_val = val;
    }
}

pub fn run()
{
    let test = Arc::new(RwLock::new(Test::new()));

    let mut threads = vec![];

    {
        let mut node = test.write().unwrap();
        node.update(11);
    }

    for i in 0..10
    {
        let clone1 = test.clone();
        let clone2 = test.clone();

        threads.push(thread::spawn(move ||
        {
            let t = clone1.read().unwrap();
            t.print();
            thread::sleep(Duration::from_millis(1000));
        }));

        let i = i;
        threads.push(thread::spawn(move ||
        {
            let mut t = clone2.write().unwrap();
            t.update(i);
            thread::sleep(Duration::from_millis(1000));
        }));
    }

    for thread in threads
    {
        thread.join().unwrap();
    }
}