fn open()
{

}

mod lol
{
    pub fn bla()
    {

    }
}

mod house
{
    pub mod apartment
    {
        pub fn door()
        {
            super::super::open();
            super::super::lol::bla();
        }
    }
}

use self::house::apartment;
use self::house::apartment::door;

use std::collections::HashMap;

pub fn run()
{
    println!("{}", "lol");

    house::apartment::door();
    crate::crates_modules::house::apartment::door();

    apartment::door();
    door();

    let mut map = HashMap::new();
    map.insert(1, 1);
}