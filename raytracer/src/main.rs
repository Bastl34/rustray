//#![feature(get_mut_unchecked)]

extern crate sdl2;
extern crate rand;
extern crate image;

use chrono::{Datelike, Timelike, Utc};

use std::{io::Write, thread};
use std::time::{Instant, Duration};

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
pub mod camera;

use renderer::RendererManager;
use raytracing::Raytracing;
use scene::Scene;

/*SDL stuff:

https://crates.io/crates/sdl2
https://docs.rs/sdl2/0.30.0/sdl2/render/struct.Canvas.html
http://nercury.github.io/rust/opengl/tutorial/2018/02/09/opengl-in-rust-from-scratch-02-opengl-context.html

*/

//const DATA_PATH: &str = "data";
const IMAGE_PATH: &str = "data/output";
const POS_PATH: &str = "data/pos.data";


fn main()
{
    let args: Vec<String> = std::env::args().collect();

    let mut run_as_window = true;

    if args.len() > 1
    {
        let command = args[1].clone();

        if command != "win"
        {
            run_as_window = false;
        }
    }

    if run_as_window
    {
        main_sdl();
    }
    else
    {
        main_cmd();
    }
}

fn main_cmd()
{
    let width: i32 = 800;
    let height: i32 = 600;

    let mut image: RgbImage = ImageBuffer::new(width as u32, height as u32);

    let mut current_time: u128;

    let timer = Instant::now();

    let mut scene = Scene::new();
    scene.cam.init(width as u32, height as u32);
    scene.load_json("scene/room.json");
    scene.print();

    let scene = std::sync::Arc::new(std::sync::RwLock::new(scene));

    let raytracing = Raytracing::new(scene);

    let raytracing_arc = std::sync::Arc::new(std::sync::RwLock::new(raytracing));

    let mut rendering = RendererManager::new(width, height, raytracing_arc.clone());
    rendering.start();

    let mut fps_display_update: u128 = 0;
    let mut pps = 0;

    let mut completed = false;

    while !completed
    {
        let receiver = rendering.get_message_receiver();

        loop
        {
            let res = receiver.try_recv();

            if res.is_err() { break }

            if res.is_ok()
            {
                let item = res.unwrap();

                //check range to prevent draing something outside while resizing
                if item.x < image.width() as i32 && item.y < image.height() as i32
                {
                    image.put_pixel(item.x as u32, item.y as u32, Rgb([item.r, item.g, item.b]));
                    pps += 1;
                }
            }
        }

        current_time = timer.elapsed().as_millis();

        //update window title
        if current_time - fps_display_update >= 1000
        {
            let pixels = rendering.get_rendered_pixels();
            let is_done = rendering.is_done();
            let elapsed = rendering.check_and_get_elapsed_time() as f64 / 1000.0;
            let percentage = (pixels as f32 / (width * height) as f32) * 100.0;

            println!("{:0>6.2}%, PPS: {} Pixels: {}, Time: {:.2}s",percentage, pps, pixels, elapsed);

            fps_display_update = current_time;

            if is_done && !completed
            {
                println!("rendering time: {}", elapsed);
                completed = true;

                //save
                let now = Utc::now();

                let filename = format!("{}/output_{}-{}-{} {}-{}-{}.png", IMAGE_PATH, now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
                image.save(filename).unwrap();
            }

            if !completed
            {
                pps = 0;
            }
        }

        thread::sleep(Duration::from_millis(100));
    }

    println!("done");
}

fn main_sdl()
{
    let sdl = sdl2::init().unwrap();

    sdl2::hint::set("SDL_HINT_VIDEO_ALLOW_SCREENSAVER", "1");

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
    scene.clear();
    //scene.load_json("scene/room.json");
    //scene.load_json("scene/spheres.json");
    //scene.load_json("scene/monkey.json");
    //scene.load_json("scene/kbert.json");
    //scene.load_json("scene/earth.json");
    //scene.load_gltf("scene/models/monkey/monkey.glb");
    //scene.get_by_name("unknown").unwrap().get_basic_mut().material.smooth_shading = false;
    //scene.load_gltf("scene/bmw27_cpu.glb");
    scene.load_gltf("scene/models/Sponza/glTF/Sponza_fixed.gltf");
    for item in scene.get_vec_by_name("unknown")
    {
        item.get_basic_mut().material.texture_ambient = item.get_basic_mut().material.texture_base.clone();
        item.get_basic_mut().material.ambient_color = item.get_basic_mut().material.base_color * 0.5;

        item.get_basic_mut().material.smooth_shading = true;
    }

    let scene = std::sync::Arc::new(std::sync::RwLock::new(scene));

    let mut raytracing = Raytracing::new(scene.clone());
    raytracing.load_settings("scene/default_render_settings.json");
    raytracing.print_settings();

    {
        scene.write().unwrap().cam.init(width as u32, height as u32);
        scene.read().unwrap().print();
    }

    let raytracing_arc = std::sync::Arc::new(std::sync::RwLock::new(raytracing));

    let mut rendering = RendererManager::new(width, height, raytracing_arc.clone());
    rendering.start();

    let mut fps_display_update: u128 = 0;
    let mut pps = 0;

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
                    thread::sleep(Duration::from_millis(100));

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
                    //reset rendering canvas and buffer canvas
                    rendering.stop();
                    thread::sleep(Duration::from_millis(100));

                    //apply
                    width = w;
                    height = h;

                    canvas.set_draw_color(Color::RGB(255, 255, 255));
                    canvas.clear();
                    canvas.present();

                    render_canvas = sdl2::surface::Surface::new(width as u32, height as u32, PixelFormatEnum::RGBA32).unwrap().into_canvas().unwrap();
                    render_canvas.set_draw_color(Color::RGB(255, 255, 255));
                    render_canvas.clear();

                    {
                        let mut scene = scene.write().unwrap();
                        scene.cam.init(width as u32, height as u32);
                    }

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

                //check range to prevent draing something outside while resizing
                if item.x < image.width() as i32 && item.y < image.height() as i32
                {
                    image.put_pixel(item.x as u32, item.y as u32, Rgb([item.r, item.g, item.b]));
                    pps += 1;
                }
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

            let title = format!("Raytracer (FPS: {:.2}, PPS: {} Res: {}x{}, Complete: {:.2}%, Pixels: {}, Time: {:.2}s, Done: {})",fps, pps, width, height, percentage, pixels, elapsed, is_done);

            window.set_title(&title).unwrap();

            fps_display_update = current_time;

            if is_done && !completed
            {
                println!("rendering time: {}", elapsed);
                completed = true;

                //save
                let now = Utc::now();

                let filename = format!("{}/output_{}-{}-{} {}-{}-{}.png", IMAGE_PATH, now.year(), now.month(), now.day(), now.hour(), now.minute(), now.second());
                image.save(filename).unwrap();
            }

            if !completed
            {
                pps = 0;
            }
        }

    }

    rendering.stop();
}
