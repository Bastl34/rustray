use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use std::time::{Instant, Duration};

use std::collections::VecDeque;

extern crate num_cpus;

use rand::seq::SliceRandom;


use crate::raytracing::Raytracing;
use crate::pixel_color::PixelColor;

//const BLOCK_SIZE: i32 = 32;
const BLOCK_SIZE: i32 = 2;

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

pub struct RendererManager
{
    width: i32,
    height: i32,

    running: Arc<Mutex<bool>>,

    cell_list: Arc<Mutex<VecDeque<CellRange>>>,

    threads: Vec<JoinHandle<()>>,
    message_sender: Sender<PixelColor>,
    message_receiver: Receiver<PixelColor>,

    pixels_rendered: Arc<Mutex<u64>>,

    start_time: Instant,
    done_time: Duration,

    raytracing: Arc<RwLock<Raytracing>>
}

impl RendererManager
{
    pub fn new(width: i32, height: i32, raytracing: Arc<RwLock<Raytracing>>) -> RendererManager
    {
        let (tx, rx) = mpsc::channel();

        RendererManager
        {
            width: width,
            height: height,

            running: std::sync::Arc::new(std::sync::Mutex::new(false)),

            cell_list: std::sync::Arc::new(std::sync::Mutex::new(VecDeque::new())),

            threads: vec![],
            message_sender: tx,
            message_receiver: rx,

            pixels_rendered: std::sync::Arc::new(std::sync::Mutex::new(0)),

            start_time: Instant::now(),
            done_time: Duration::new(0, 0),

            raytracing: raytracing
        }
    }

    pub fn update_resolution(&mut self, width: i32, height: i32)
    {
        self.width = width;
        self.height = height;
    }

    pub fn start(&mut self)
    {
        println!("");
        println!("starting (w={}, h={})...", self.width, self.height);
        println!("block size: {}", BLOCK_SIZE);

        let cores = num_cpus::get();
        println!("cores found: {}", cores);

        //start time
        self.start_time = Instant::now();
        self.done_time = Duration::new(0, 0);

        //init pixel stat
        { *(self.pixels_rendered.lock().unwrap()) = 0; }

        //running
        { *(self.running.lock().unwrap()) = true; }

        //init cells
        { (*(self.cell_list.lock().unwrap())).clear(); }

        let mut x = 0;
        while x < self.width
        {
            let mut x1 = x + BLOCK_SIZE -1 ;
            if x1 >= self.width { x1 = self.width - 1; }

            let mut y = 0;
            while y < self.height
            {
                let mut y1 = y + BLOCK_SIZE -1 ;
                if y1 >= self.height { y1 = self.height - 1; }

                let cell = CellRange { x0: x, x1: x1, y0: y, y1: y1 };

                (*(self.cell_list.lock().unwrap())).push_back(cell);

                if y1 == self.height -1
                {
                    y = self.height
                }
                else
                {
                    y += BLOCK_SIZE;
                }
            }

            if x1 == self.width -1
            {
                x = self.width
            }
            else
            {
                x += BLOCK_SIZE;
            }
        }

        //randomize
        {
            let mut deque = self.cell_list.lock().unwrap();
            deque.make_contiguous().shuffle(&mut rand::thread_rng());
        }

        for _ in 0..cores
        {
            let handle = self.start_thread();
            self.threads.push(handle);
        }
    }

    pub fn stop(&mut self)
    {
        {
            let mut running_ref = self.running.lock().unwrap();

            if *(running_ref) == false
            {
                println!("not running");
                return;
            }

            *(running_ref) = false;
        }

        //let bt = backtrace::Backtrace::new();
        //println!("{:?}",bt);

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

    pub fn has_cells_left(&self) -> bool
    {
        (*(self.cell_list.lock().unwrap())).len() > 0
    }

    pub fn get_rendered_pixels(&self) -> u64
    {
        *(self.pixels_rendered.lock().unwrap())
    }

    pub fn is_done(&self) -> bool
    {
        (*(self.pixels_rendered.lock().unwrap())) == self.width as u64 * self.height as u64
    }

    pub fn check_and_get_elapsed_time(&mut self) -> u128
    {
        if self.done_time.as_millis() > 0
        {
            return self.done_time.as_millis()
        }

        if self.done_time.as_millis() == 0 && self.is_done()
        {
            self.done_time = self.start_time.elapsed();
        }

        self.start_time.elapsed().as_millis()
    }

    pub fn get_message_receiver(&self) -> &Receiver<PixelColor>
    {
        &self.message_receiver
    }

    fn start_thread(&self) -> JoinHandle<()>
    {
        let tx = self.message_sender.clone();
        let cell_list = Arc::clone(&self.cell_list);
        let raytracing = Arc::clone(&self.raytracing);

        let running_mutex = Arc::clone(&self.running);
        let pixels_rendered = Arc::clone(&self.pixels_rendered);
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
                    for y in range.y0 .. range.y1 + 1
                    {
                        for x in range.x0 .. range.x1 + 1
                        {
                            if !check_running()
                            {
                                break 'outer;
                            }

                            let rt = raytracing.read().unwrap();
                            let pixel_val = rt.render(x,y);

                            { *(pixels_rendered.lock().unwrap()) += 1; }

                            let _ = tx.send(pixel_val);
                        }
                    }
                }
            }
        });

        handle
    }
}