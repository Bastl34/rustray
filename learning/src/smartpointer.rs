#![allow(unused_variables)]

enum List
{
    Cons(i32, Box<List>),
    Nil,
}

//use crate::List::{Cons, Nil};
use crate::smartpointer::List::{Cons, Nil};

pub fn run()
{
    let b = Box::new(5);

    println!("{}", b);

    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
}