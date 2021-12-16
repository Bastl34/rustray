use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::Mutex;
use std::sync::Arc;
use core::time::Duration;

use std::collections::VecDeque;

extern crate num_cpus;

use crate::pixel_color::PixelColor;
use crate::helper;

const BLOCK_SIZE: i32 = 32;

struct CellRange
{
    x0: i32,
    x1: i32,
    y0: i32,
    y1: i32
}

impl CellRange
{
    pub fn new() -> CellRange
    {
        CellRange {x0: 0, x1: 0, y0: 0, y1: 0}
    }
}

pub struct Renderer
{
    width: i32,
    height: i32,

    running: Arc<Mutex<bool>>,

    cell_list: Arc<Mutex<VecDeque<CellRange>>>,

    threads: Vec<JoinHandle<()>>,
    message_sender: Sender<PixelColor>,
    message_receiver: Receiver<PixelColor>,
}

impl Renderer
{
    pub fn new(width: i32, height: i32) -> Renderer
    {
        let (tx, rx) = mpsc::channel();

        Renderer
        {
            width: width,
            height: height,

            running: std::sync::Arc::new(std::sync::Mutex::new(false)),

            cell_list: std::sync::Arc::new(std::sync::Mutex::new(VecDeque::new())),

            threads: vec![],
            message_sender: tx,
            message_receiver: rx
        }
    }

    pub fn update_resolution(&mut self, width: i32, height: i32)
    {
        self.width = width;
        self.height = height;
    }

    pub fn start(&mut self)
    {
        let cores = num_cpus::get();

        println!("cores found: {}", cores);

        { *(self.running.lock().unwrap()) = true; }

        let mut x = 0;
        while x < self.width
        {
            let mut y = 0;
            while y < self.width
            {
                let mut x1 = x + BLOCK_SIZE;
                let mut y1 = y + BLOCK_SIZE;

                if x1 >= self.width { x1 = self.width - 1; }
                if y1 >= self.height { y1 = self.height - 1; }

                let cell = CellRange { x0: x, x1: x1, y0: y, y1: y1 };

                (*(self.cell_list.lock().unwrap())).push_back(cell);

                y += BLOCK_SIZE;
            }

            x += BLOCK_SIZE;
        }

        for _ in 0..cores
        {
            let handle = self.start_thread();
            self.threads.push(handle);
        }
    }

    pub fn stop(&mut self)
    {
        { *(self.running.lock().unwrap()) = false; }

        println!("waiting for all threads to end...");

        //https://stackoverflow.com/questions/68966949/unable-to-join-threads-from-joinhandles-stored-in-a-vector-rust
        while let Some(cur_thread) = self.threads.pop()
        {
            cur_thread.join().unwrap();
        }

        println!("threads stopped");
    }

    pub fn restart(&mut self, width: i32, height: i32)
    {
        self.stop();
        self.update_resolution(width, height);
        self.start();
    }

    pub fn get_message_receiver(&self) -> &Receiver<PixelColor>
    {
        &self.message_receiver
    }

    pub fn check_running(&self) -> bool
    {
        let running_mutex = Arc::clone(&self.running);
        let running = running_mutex.lock().unwrap();
        return *running
    }

    fn start_thread(&self) -> JoinHandle<()>
    {
        let tx = self.message_sender.clone();
        let cell_list = Arc::clone(&self.cell_list);

        let running_mutex = Arc::clone(&self.running);
        let check_running = move || -> bool
        {
            let running = running_mutex.lock().unwrap();
            return *running;
        };

        let handle = std::thread::spawn(move ||
        {
            let mut running = true;
            'outer: while running
            {
                //check running
                if !check_running()
                {
                    break 'outer;
                }

                //get new cell from list
                let mut range = CellRange::new();
                {
                    let front = (*(cell_list.lock().unwrap())).pop_front();

                    if front.is_some()
                    {
                        range = front.unwrap();
                    }
                    else
                    {
                        running = false;
                    }
                }

                //render
                if running
                {
                    for y in range.y0 .. range.y1
                    {
                        for x in range.x0 .. range.x1
                        {
                            if !check_running()
                            {
                                break 'outer;
                            }

                            let r = helper::rand(0, 255);
                            let g = helper::rand(0, 255);
                            let b = helper::rand(0, 255);

                            let pixel_val = PixelColor { r: r, g: g, b: b, x: x, y: y };
                            tx.send(pixel_val).unwrap();

                            ::std::thread::sleep(Duration::from_nanos(10));
                        }
                    }
                }
            }
        });

        handle
    }
}