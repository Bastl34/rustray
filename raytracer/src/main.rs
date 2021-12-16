extern crate sdl2;
extern crate rand;

use std::io::Write;
use std::time::Instant;

use sdl2::rect::Point;
use sdl2::pixels::Color;

use sdl2::keyboard::Keycode;
use sdl2::event::WindowEvent;

use sdl2::video::WindowPos::Positioned;

use std::fs::File;

pub mod helper;
pub mod pixel_color;
pub mod shape;

pub mod renderer;
pub mod raytracing;

//mod pixel_color;

use renderer::RendererManager;
use raytracing::Raytracing;


/*SDL stuff:

https://crates.io/crates/sdl2
https://docs.rs/sdl2/0.30.0/sdl2/render/struct.Canvas.html
http://nercury.github.io/rust/opengl/tutorial/2018/02/09/opengl-in-rust-from-scratch-02-opengl-context.html

*/


fn main()
{
    let sdl = sdl2::init().unwrap();

    let video_subsystem = sdl.video().unwrap();

    let mut width: i32 = 900;
    let mut height: i32 = 700;

    let mut window = video_subsystem.window("Raytracer", width as u32, height as u32).resizable().build().unwrap();

    //try to load window position
    let data = std::fs::read_to_string("pos.data");
    if data.is_ok()
    {
        let res = data.unwrap();
        let splits: Vec<&str> = res.split("x").collect();
        let splits_arr = splits.as_slice();

        let x: i32 = splits_arr[0].parse().unwrap();
        let y: i32 = splits_arr[1].parse().unwrap();

        println!("x: {} y: {}", x, y);

        window.set_position(Positioned(x), Positioned(y));
    }

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    //let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let mut event_pump = sdl.event_pump().unwrap();

    let mut last_time: u128 = 0;
    let mut current_time: u128;

    let timer = Instant::now();

    //let mut raytracing = std::sync::Arc::new(std::sync::Mutex::new(Raytracing::new()));
    //let mut raytracing: Box<Raytracing + Send> = Box::new(Raytracing::new());
    let mut raytracing = std::sync::Arc::new(Raytracing::new());

    let mut rendering = RendererManager::new(width, height, raytracing);
    rendering.start();

    'main: loop
    {
        for event in event_pump.poll_iter()
        {
            match event
            {
                sdl2::event::Event::Quit {..} =>
                    break 'main,
                sdl2::event::Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>
                    break 'main,
                sdl2::event::Event::Window { win_event: WindowEvent::Resized(w, h), ..} =>
                {
                    width = w;
                    height = h;

                    canvas.set_draw_color(Color::RGB(0, 0, 0));
                    canvas.clear();

                    rendering.restart(width, height);
                },
                //save the window position
                sdl2::event::Event::Window { win_event: WindowEvent::Moved(x, y), ..} =>
                {
                    let mut file = File::create("pos.data").unwrap();
                    let _ = file.write(format!("{}x{}", x, y).as_bytes());
                },
                _ => {},
            }
        }

        let receiver = rendering.get_message_receiver();

        loop
        {
            let res = receiver.try_recv();

            if res.is_err() { break }

            if res.is_ok()
            {
                let item = res.unwrap();
                canvas.set_draw_color(Color::RGB(item.r, item.g, item.b));
                canvas.draw_point(Point::new(item.x, item.y)).unwrap();
            }
        }

        //draw
        canvas.present();

        //calc fps
        current_time = timer.elapsed().as_millis();
        let fps = 1000.0 / (current_time - last_time) as f64;
        last_time = current_time;

        //update window title
        let window = canvas.window_mut();
        let title = format!("Raytracer (FPS: {:.2})",fps);
        window.set_title(&title).unwrap();
    }

    rendering.stop();
}
