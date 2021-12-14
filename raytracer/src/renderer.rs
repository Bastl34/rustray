use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use std::sync::Mutex;
use std::sync::Arc;
use core::time::Duration;

extern crate num_cpus;

use crate::pixel_color::PixelColor;
use crate::helper;

struct CellRange
{
    x: i32,
    y: i32,
    width: i32,
    height: i32
}

pub struct Renderer
{
    width: i32,
    height: i32,

    running: Arc<Mutex<bool>>,

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

        let rows = (cores as f64).sqrt() as i32;
        let cols = (cores as f64).sqrt() as i32;

        let mut x_start: i32 = 0;
        let mut y_start: i32 = 0;

        let x_steps = self.width / (cols as i32);
        let y_steps = self.height / (rows as i32);

        for _ in 0..cols
        {
            for _ in 0..rows
            {
                let range = CellRange { x: x_start, y: y_start, width: x_steps, height: y_steps };
                let handle = self.start_thread(range);
                self.threads.push(handle);

                // WARNING: it could be that the last cell may not complete fill the screen res
                // but it is ok for now 🤷 ¯\_(ツ)_/¯
                y_start += y_steps;
            }

            x_start += x_steps;
            y_start = 0;
        }

        println!("rows: {}, cols: {}", rows, cols);
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

    fn start_thread(&self, range: CellRange) -> JoinHandle<()>
    {
        let tx = self.message_sender.clone();
        let mutex = Arc::clone(&self.running);

        let handle = std::thread::spawn(move ||
        {
            for y in range.y .. range.y + range.height
            {
                for x in range.x .. range.x + range.width
                {
                    {
                        let running = mutex.lock().unwrap();
                        if !(*running)
                        {
                            break;
                        }
                    }

                    let r = helper::rand(0, 255);
                    let g = helper::rand(0, 255);
                    let b = helper::rand(0, 255);

                    let pixel_val = PixelColor { r: r, g: g, b: b, x: x, y: y };
                    tx.send(pixel_val).unwrap();

                    ::std::thread::sleep(Duration::from_nanos(10));
                }
            }
        });

        handle
    }
}