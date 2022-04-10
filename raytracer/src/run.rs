extern crate sdl2;
extern crate rand;
extern crate image;

use chrono::{Datelike, Timelike, Utc};
use sdl2::{Sdl, EventPump};
use sdl2::mouse::MouseButton;
use sdl2::render::{Canvas, TextureCreator, Texture};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

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


use crate::renderer::RendererManager;
use crate::raytracing::Raytracing;
use crate::scene::Scene;

const WINDOW_TITLE: &str = "Raytracer";

const IMAGE_PATH: &str = "data/output";
const ANIMATION_PATH: &str = "data/output/animation";
const POS_PATH: &str = "data/pos.data";

const DEFAULT_RES: (i32, i32) = (800, 600);

pub struct SDLData<'a>
{
    pub context: Sdl,
    pub canvas: Canvas<Window>,
    pub render_canvas: Canvas<Surface<'a>>,

    pub texture_creator: TextureCreator<WindowContext>,

    pub event_pump: EventPump
}

pub struct Run<'a>
{
    width: i32,
    height: i32,

    window_x: i32,
    window_y: i32,

    window: bool,
    animate: bool,

    scenes: Vec<String>,

    image: RgbImage,

    sdl: Option<SDLData<'a>>
}

impl<'a> Run<'a>
{
    pub fn new(width: i32, height: i32, window: bool, scenes: Vec<String>, animate: bool) -> Run<'a>
    {
        Run
        {
            width: width,
            height: height,

            window_x: 0,
            window_y: 0,

            window: window,
            animate: animate,

            scenes: scenes,

            image: ImageBuffer::new(0, 0),

            sdl: None
        }
    }

    pub fn start(&mut self)
    {
        self.load_window_pos_and_res();

        if self.width == 0 || self.height == 0
        {
            self.width = DEFAULT_RES.0;
            self.height = DEFAULT_RES.1;
        }

        self.init_image();

        self.start_sdl();
        self.sdl_loop();
    }

    pub fn init_image(&mut self)
    {
        self.image = ImageBuffer::new(self.width as u32, self.height as u32);
    }

    pub fn load_window_pos_and_res(&mut self)
    {
        //try to load window position
        let data = std::fs::read_to_string(POS_PATH);
        if data.is_ok()
        {
            let res = data.unwrap();
            let splits: Vec<&str> = res.split("x").collect();
            let splits_arr = splits.as_slice();

            self.window_x = splits_arr[0].parse().unwrap();
            self.window_y = splits_arr[1].parse().unwrap();

            //do only load resolution when there was no set explicitly
            if self.width == 0 && self.height == 0
            {
                self.width = splits_arr[2].parse().unwrap();
                self.height = splits_arr[3].parse().unwrap();
            }
        }
    }

    pub fn start_sdl(&mut self)
    {
        let sdl = sdl2::init().unwrap();

        sdl2::hint::set("SDL_HINT_VIDEO_ALLOW_SCREENSAVER", "1");

        let video_subsystem = sdl.video().unwrap();

        let mut window = video_subsystem.window(WINDOW_TITLE, self.width as u32, self.height as u32).resizable().allow_highdpi().build().unwrap();

        if self.window_x != 0 && self.window_y != 0
        {
            window.set_position(Positioned(self.window_x), Positioned(self.window_y));
        }

        let mut canvas = window.into_canvas().present_vsync().build().unwrap();

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();
        canvas.present();

        let mut render_canvas = sdl2::surface::Surface::new(self.width as u32, self.height as u32, PixelFormatEnum::RGBA32).unwrap().into_canvas().unwrap();
        render_canvas.set_draw_color(Color::RGB(255, 255, 255));
        render_canvas.clear();

        let texture_creator = canvas.texture_creator();

        let event_pump = sdl.event_pump().unwrap();

        self.sdl = Some(SDLData
        {
            context: sdl,
            canvas: canvas,
            render_canvas: render_canvas,
            texture_creator: texture_creator,
            event_pump: event_pump
        });
    }

    pub fn sdl_loop(&mut self)
    {
        let mut sdl = self.sdl.as_mut().unwrap();

        'main: loop
        {
            for event in sdl.event_pump.poll_iter()
            {
                match event
                {
                    sdl2::event::Event::Quit { .. } =>
                        break 'main,
                    sdl2::event::Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>
                        break 'main,
                    _ => {},
                }
            }
        }
    }
}