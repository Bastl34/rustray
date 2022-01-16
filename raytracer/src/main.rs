extern crate sdl2;
extern crate rand;
extern crate image;

use chrono::{Datelike, Timelike, Utc};

use std::io::Write;
use std::time::Instant;

use sdl2::rect::Point;
use sdl2::pixels::Color;

use sdl2::keyboard::Keycode;
use sdl2::event::WindowEvent;
use sdl2::pixels::PixelFormatEnum;

use sdl2::video::WindowPos::Positioned;

use image::{ImageBuffer, RgbImage, Rgb};

use std::fs::File;

pub mod helper;
pub mod pixel_color;
pub mod shape;

pub mod renderer;
pub mod raytracing;
pub mod scene;

use renderer::RendererManager;
use raytracing::Raytracing;
use scene::Scene;

/*SDL stuff:

https://crates.io/crates/sdl2
https://docs.rs/sdl2/0.30.0/sdl2/render/struct.Canvas.html
http://nercury.github.io/rust/opengl/tutorial/2018/02/09/opengl-in-rust-from-scratch-02-opengl-context.html

*/

const DATA_PATH: &str = "data";
const POS_PATH: &str = "data/pos.data";

fn main()
{
    let sdl = sdl2::init().unwrap();

    let video_subsystem = sdl.video().unwrap();

    let mut width: i32 = 800;
    let mut height: i32 = 600;

    let mut window_x = 0;
    let mut window_y = 0;

    //try to load window position
    let data = std::fs::read_to_string(POS_PATH);
    if data.is_ok()
    {
        let res = data.unwrap();
        let splits: Vec<&str> = res.split("x").collect();
        let splits_arr = splits.as_slice();

        window_x = splits_arr[0].parse().unwrap();
        window_y = splits_arr[1].parse().unwrap();
        width = splits_arr[2].parse().unwrap();
        height = splits_arr[3].parse().unwrap();
    }

    let mut window = video_subsystem.window("Raytracer", width as u32, height as u32).resizable().allow_highdpi().build().unwrap();
    window.set_position(Positioned(window_x), Positioned(window_y));

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();
    canvas.present();

    let mut image: RgbImage = ImageBuffer::new(width as u32, height as u32);

    let mut render_canvas = sdl2::surface::Surface::new(width as u32, height as u32, PixelFormatEnum::RGBA32).unwrap().into_canvas().unwrap();
    render_canvas.set_draw_color(Color::RGB(255, 255, 255));
    render_canvas.clear();

    let texture_creator = canvas.texture_creator();
    let mut texture;

    let mut event_pump = sdl.event_pump().unwrap();

    let mut last_time: u128 = 0;
    let mut current_time: u128;

    let timer = Instant::now();

    let mut scene = Scene::new();
    scene.init_with_some_objects();

    let mut raytracing = Raytracing::new(scene);
    raytracing.init_camera(width as u32, height as u32);

    let raytracing_arc = std::sync::Arc::new(raytracing);

    let mut rendering = RendererManager::new(width, height, raytracing_arc);
    rendering.start();

    let mut fps_display_update: u128 = 0;

    let mut completed = false;

    'main: loop
    {
        for event in event_pump.poll_iter()
        {
            match event
            {
                sdl2::event::Event::Quit { .. } =>
                    break 'main,
                sdl2::event::Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>
                    break 'main,
                sdl2::event::Event::KeyDown { keycode: Some(Keycode::Space), .. } =>
                {
                    rendering.stop();

                    render_canvas = sdl2::surface::Surface::new(width as u32, height as u32, PixelFormatEnum::RGBA32).unwrap().into_canvas().unwrap();
                    render_canvas.set_draw_color(Color::RGB(255, 255, 255));
                    render_canvas.clear();

                    rendering.restart(width, height);

                    image = ImageBuffer::new(width as u32, height as u32);

                    completed = false;
                },
                //restart rendering on resize
                sdl2::event::Event::Window { win_event: WindowEvent::Resized(w, h), ..} =>
                {
                    //apply
                    width = w;
                    height = h;

                    //reset rendering canvas and buffer canvas
                    rendering.stop();

                    canvas.set_draw_color(Color::RGB(255, 255, 255));
                    canvas.clear();
                    canvas.present();

                    render_canvas = sdl2::surface::Surface::new(width as u32, height as u32, PixelFormatEnum::RGBA32).unwrap().into_canvas().unwrap();
                    render_canvas.set_draw_color(Color::RGB(255, 255, 255));
                    render_canvas.clear();

                    //TODO
                    //test.init_camera(width as u32, height as u32);

                    rendering.restart(width, height);

                    image = ImageBuffer::new(width as u32, height as u32);

                    completed = false;

                    //save to file
                    let mut file = File::create(POS_PATH).unwrap();
                    let _ = file.write(format!("{}x{}x{}x{}", window_x, window_y, width, height).as_bytes());
                },
                //save the window position
                sdl2::event::Event::Window { win_event: WindowEvent::Moved(x, y), ..} =>
                {
                    //apply changes
                    window_x = x;
                    window_y = y;

                    //save to file
                    let mut file = File::create(POS_PATH).unwrap();
                    let _ = file.write(format!("{}x{}x{}x{}", window_x, window_y, width, height).as_bytes());
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

                render_canvas.set_draw_color(Color::RGB(item.r, item.g, item.b));
                render_canvas.draw_point(Point::new(item.x, item.y)).unwrap();

                image.put_pixel(item.x as u32, item.y as u32, Rgb([item.r, item.g, item.b]));
            }
        }

        texture = texture_creator.create_texture_from_surface(render_canvas.surface()).unwrap();
        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        //calc fps
        current_time = timer.elapsed().as_millis();
        let fps = 1000.0f64 / (current_time - last_time) as f64;
        last_time = current_time;

        //update window title
        if current_time - fps_display_update >= 1000
        {
            let pixels = rendering.get_rendered_pixels();
            let is_done = rendering.is_done();
            let elapsed = rendering.check_and_get_elapsed_time() as f64 / 1000.0;
            let percentage = (pixels as f32 / (width * height) as f32) * 100.0;

            let window = canvas.window_mut();

            let title = format!("Raytracer (FPS: {:.2}, Res: {}x{}, Complete: {:.2}%, Pixels: {}, Time: {:.2}s, Done: {})",fps, width, height, percentage, pixels, elapsed, is_done);

            window.set_title(&title).unwrap();

            fps_display_update = current_time;

            if is_done && !completed
            {
                println!("rendering time: {}", elapsed);
                completed = true;

                //save
                let now = Utc::now();

                let filename = format!("{}/output_{}-{}-{} {}-{}-{}.png", DATA_PATH, now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
                image.save(filename).unwrap();
            }
        }

    }

    rendering.stop();
}
