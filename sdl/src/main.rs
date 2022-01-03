extern crate sdl2;

use sdl2::rect::Rect;
use sdl2::rect::Point;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;

use sdl2::keyboard::Keycode;

use rand::Rng;

pub fn rand<T: std::cmp::PartialOrd + rand::distributions::uniform::SampleUniform>(from: T, to: T) -> T
{
    rand::thread_rng().gen_range(from..to) as T
}

#[derive(Clone, Copy)]
struct Pixel
{
    r: u8,
    g: u8,
    b: u8,
    x: i32,
    y: i32,
}

fn main()
{
    let sdl = sdl2::init().unwrap();

    let video_subsystem = sdl.video().unwrap();

    let width: usize = 800;
    let height: usize = 600;

    let mut pixels: Vec::<Pixel> = Vec::new();

    let window = video_subsystem.window("SDL", width as u32, height as u32).resizable().build().unwrap();

    let mut canvas = window.into_canvas().target_texture().present_vsync().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let mut event_pump = sdl.event_pump().unwrap();

    for x in 0..width
    {
        for y in 0..height
        {
            pixels.push(Pixel {r: 0, g: 0, b: 0, x: 0, y: 0});
        }
    }

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::ARGB8888, width as u32, height as u32).unwrap();

    // Create a red-green gradient
    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        for y in 0..height as usize {
            for x in 0..width as usize {
                let offset = y * pitch + x * 4;
                buffer[offset] = 0;
                buffer[offset + 1] = 0;
                buffer[offset + 2] = 0;
                buffer[offset + 3] = 0;
            }
        }
    }).unwrap();

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
                _ => {},
            }
        }

        //canvas.set_draw_color(Color::RGB(0, 0, 250));
        //canvas.fill_rect(Rect::new(0, 0, 100, 100)).unwrap();

        let r = rand(0, 255);
        let g = rand(0, 255);
        let b = rand(0, 255);

        let x = rand(0, width) as i32;
        let y = rand(0, height) as i32;

        let i = x + (width as i32 * y);
        pixels[i as usize] = Pixel {r: r, g: g, b: b, x: x, y: y};

        canvas.clear();
        for pixel in &pixels
        {
            canvas.set_draw_color(Color::RGB(pixel.r, pixel.g, pixel.b));
            canvas.draw_point(Point::new(pixel.x, pixel.y)).unwrap();
        }
        canvas.present();

        //canvas.set_draw_color(Color::RGB(0, 0, 0));
        //canvas.clear();


        /*
        let pixel_data: [u8; 4] = [255, 0, 255, 0];
        texture.update(Rect::new(x, y, 1, 1), &pixel_data, 1).unwrap();

        canvas.clear();
        canvas.copy(&texture, None, Some(Rect::new(0, 0, width, height))).unwrap();
        canvas.present();
         */

        /*
        canvas.set_draw_color(Color::RGB(r, g, b));

        canvas.draw_point(Point::new(x, y)).unwrap();
        canvas.fill_rect(Rect::new(x, x, 100, 100)).unwrap();


        canvas.present();
        */


        //std::thread::sleep(std::time::Duration::from_millis(16));
    }
}