extern crate sdl2;
extern crate rand;
extern crate image;

use chrono::{Datelike, Timelike, Utc, DateTime};
use sdl2::{Sdl, EventPump};
use sdl2::mouse::MouseButton;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

use std::sync::{RwLock, Arc};
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

// ******************** SDLData ********************
pub struct SDLData<'a>
{
    pub context: Sdl,
    pub canvas: Canvas<Window>,
    pub render_canvas: Canvas<Surface<'a>>,

    pub texture_creator: TextureCreator<WindowContext>,

    pub event_pump: EventPump
}

// ******************** Stats ********************
pub struct Stats
{
    last_time: u128,
    current_time: u128,

    frame: u64,

    timer: Instant,
    output_time: DateTime<Utc>,

    fps_display_update: u128,
    fps: f64,
    pps: u64,

    completed: bool
}

impl Stats
{
    pub fn new() -> Stats
    {
        Stats
        {
            last_time: 0,
            current_time: 0,
            frame: 0,
            timer: Instant::now(),
            output_time: Utc::now(),

            fps_display_update: 0,
            fps: 0.0,
            pps: 0,

            completed: false
        }
    }

    pub fn reset(&mut self)
    {
        self.last_time = 0;
        self.current_time = 0;
        self.frame = 0;

        //self.output_time = Utc::now();

        self.timer = Instant::now();
        self.fps_display_update = 0;
        self.pps = 0;

        self.completed = false;
    }
}

// ******************** Run ********************
pub struct Run<'a>
{
    width: i32,
    height: i32,

    window_x: i32,
    window_y: i32,

    window: bool,
    animate: bool,

    scenes_list: Vec<String>,

    image: RgbImage,
    scene: Arc<RwLock<Scene>>,
    pub raytracing: Arc<RwLock<Raytracing>>,
    rendering: RendererManager,

    stats: Stats,
    sdl: Option<SDLData<'a>>,

    help_printed: bool
}

impl<'a> Run<'a>
{
    pub fn new(width: i32, height: i32, window: bool, scenes_list: Vec<String>, animate: bool) -> Run<'a>
    {
        let scene = Arc::new(RwLock::new(Scene::new()));
        let rt = Arc::new(RwLock::new(Raytracing::new(scene.clone())));
        let rendering = RendererManager::new(width, height, rt.clone());

        Run
        {
            width: width,
            height: height,

            window_x: 0,
            window_y: 0,

            window: window,
            animate: animate,

            scenes_list: scenes_list,

            scene: scene,
            raytracing: rt,
            rendering: rendering,

            image: ImageBuffer::new(0, 0),

            stats: Stats::new(),
            sdl: None,

            help_printed: false,
        }
    }

    pub fn init_image(&mut self)
    {
        self.image = ImageBuffer::new(self.width as u32, self.height as u32);
    }

    pub fn init_stats(&mut self)
    {
        self.stats.reset();
    }

    pub fn init_raytracing(&mut self)
    {
        let mut scene = Scene::new();
        scene.clear();

        for scene_item in &self.scenes_list
        {
            scene.load(&scene_item);
        }

        scene.cam.init(self.width as u32, self.height as u32);
        scene.apply_frame(self.stats.frame);
        scene.print();

        let rt_config = scene.raytracing_config;

        let scene = std::sync::Arc::new(std::sync::RwLock::new(scene));

        let mut raytracing = Raytracing::new(scene.clone());
        raytracing.config.apply(rt_config);

        /*
        {
            scene.write().unwrap().cam.init(self.width as u32, self.height as u32);
            scene.read().unwrap().print();
        }

        {
            scene.write().unwrap().apply_frame(self.stats.frame);
        }
         */

         /*
        {
            raytracing.config.apply(scene.read().unwrap().raytracing_config);
        }
         */

        let raytracing_arc = std::sync::Arc::new(std::sync::RwLock::new(raytracing));

        let rendering = RendererManager::new(self.width, self.height, raytracing_arc.clone());

        self.scene = scene;
        self.raytracing = raytracing_arc;
        self.rendering = rendering;
    }

    pub fn load_window_pos_and_res(&mut self)
    {
        //only if window is enabled
        if !self.window
        {
            return;
        }

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

    pub fn init(&mut self)
    {
        self.load_window_pos_and_res();

        if self.width == 0 || self.height == 0
        {
            self.width = DEFAULT_RES.0;
            self.height = DEFAULT_RES.1;
        }

        self.init_image();
        self.init_stats();
        self.init_raytracing();
    }

    pub fn start(&mut self)
    {
        //print rt info
        { self.raytracing.read().unwrap().print_config(); }

        //some stats
        self.print_frame_info();

        //start
        self.rendering.start();

        //loop
        if self.window
        {
            self.start_sdl();
            self.sdl_loop();
        }
        else
        {
            self.cmd_loop();
        }
    }

    pub fn stop(&mut self)
    {
        self.rendering.stop();
    }

    pub fn restart_rendering(&mut self)
    {
        //stop
        self.rendering.stop();
        thread::sleep(Duration::from_millis(100));

        //clear rendering surfaces
        if let Some(sdl) = &mut self.sdl
        {
            sdl.canvas.set_draw_color(Color::RGB(255, 255, 255));
            sdl.canvas.clear();
            sdl.canvas.present();

            sdl.render_canvas = sdl2::surface::Surface::new(self.width as u32, self.height as u32, PixelFormatEnum::RGBA32).unwrap().into_canvas().unwrap();
            sdl.render_canvas.set_draw_color(Color::RGB(255, 255, 255));
            sdl.render_canvas.clear();
        }

        //reinit scene with resolution
        {
            let mut scene = self.scene.write().unwrap();
            scene.cam.init(self.width as u32, self.height as u32);
        }

        //reset stats
        self.stats.completed = false;
        self.stats.frame = 0;
        self.help_printed = false;

        //print some stats
        self.print_frame_info();

        //restart
        self.image = ImageBuffer::new(self.width as u32, self.height as u32);
        self.rendering.restart(self.width, self.height);
    }

    pub fn render_next_frame_if_possible(&mut self)
    {
        if !self.animate
        {
            return;
        }

        //check for next frame
        let has_next_frame;
        {
            let scene = self.scene.read().unwrap();
            has_next_frame = scene.frame_exists(self.stats.frame + 1);
        }

        //stop end get next frame
        if self.stats.completed && has_next_frame
        {
            self.rendering.stop();
            thread::sleep(Duration::from_millis(100));

            {
                let mut scene = self.scene.write().unwrap();

                self.stats.frame += 1;
                scene.apply_frame(self.stats.frame);
            }

            if let Some(sdl) = &mut self.sdl
            {
                //sdl.canvas.set_draw_color(Color::RGB(255, 255, 255));
                //sdl.canvas.clear();
                //sdl.canvas.present();

                sdl.render_canvas = sdl2::surface::Surface::new(self.width as u32, self.height as u32, PixelFormatEnum::RGBA32).unwrap().into_canvas().unwrap();
                sdl.render_canvas.set_draw_color(Color::RGB(255, 255, 255));
                sdl.render_canvas.clear();
            }

            //reset stats
            self.stats.completed = false;

            //print some stats
            self.print_frame_info();

            //restart
            self.image = ImageBuffer::new(self.width as u32, self.height as u32);
            self.rendering.restart(self.width, self.height);
        }

        //print out some stuff to create an animation file out of the rendered frames
        if self.stats.completed && !has_next_frame && !self.help_printed
        {
            let filename = format!("{}/output_{}-{}-{} {}-{}-{}_%08d.png", ANIMATION_PATH, self.stats.output_time.year(), self.stats.output_time.month(), self.stats.output_time.day(), self.stats.output_time.hour(), self.stats.output_time.minute(), self.stats.output_time.second());
            let filename_animation = format!("{}/output_{}-{}-{} {}-{}-{}", ANIMATION_PATH, self.stats.output_time.year(), self.stats.output_time.month(), self.stats.output_time.day(), self.stats.output_time.hour(), self.stats.output_time.minute(), self.stats.output_time.second());
            let fps = self.scene.read().unwrap().animation.fps;

            println!("");
            println!("use ffmpeg to combine frames:");
            println!(" - for mp4:  ffmpeg -i \"{}\" -c:v libx264 -vf fps={} \"{}.mp4\"", filename, fps, filename_animation);
            println!(" - for gif:  ffmpeg -i \"{}\" -vf fps={} \"{}.gif\"", filename, fps, filename_animation);
            println!(" - for webp: ffmpeg -i \"{}\" -vcodec libwebp -lossless 0 -loop 0 -an -vf fps={} \"{}.webp\"", filename, fps, filename_animation);
            println!("");

            self.help_printed = true;
        }
    }

    pub fn print_frame_info(&self)
    {
        let mut frames = 1;
        {
            let scene = self.scene.read().unwrap();
            if scene.animation.has_animation()
            {
                frames = scene.animation.get_frames_amount_to_render();
            }
        }
        println!("");
        let print_str = format!("frame: {}/{}", self.stats.frame + 1, frames);

        println!("{}", print_str);
        for _ in 0..print_str.len() { print!("="); }
        println!("");
    }

    pub fn apply_pixels(&mut self)
    {
        let receiver = self.rendering.get_message_receiver();

        loop
        {
            let res = receiver.try_recv();

            if res.is_err() { break }

            if res.is_ok()
            {
                let item = res.unwrap();

                if let Some(sdl) = &mut self.sdl
                {
                    sdl.render_canvas.set_draw_color(Color::RGB(item.r, item.g, item.b));
                    sdl.render_canvas.draw_point(Point::new(item.x, item.y)).unwrap();
                }

                //check range to prevent draing something outside while resizing
                if item.x < self.image.width() as i32 && item.y < self.image.height() as i32
                {
                    self.image.put_pixel(item.x as u32, item.y as u32, Rgb([item.r, item.g, item.b]));
                    self.stats.pps += 1;
                }
            }
        }

        if let Some(sdl) = &mut self.sdl
        {
            let texture = sdl.texture_creator.create_texture_from_surface(sdl.render_canvas.surface()).unwrap();
            sdl.canvas.clear();
            sdl.canvas.copy(&texture, None, None).unwrap();
            sdl.canvas.present();
        }
    }

    pub fn save_image(&mut self)
    {
        let mut out_dir = IMAGE_PATH;
        if self.animate
        {
            out_dir = ANIMATION_PATH;
        }

        let filename = format!("{}/output_{}-{}-{} {}-{}-{}_{:0>8}.png", out_dir, self.stats.output_time.year(), self.stats.output_time.month(), self.stats.output_time.day(), self.stats.output_time.hour(), self.stats.output_time.minute(), self.stats.output_time.second(), self.stats.frame);
        self.image.save(filename).unwrap();
    }

    pub fn calc_fps(&mut self)
    {
        //calc fps
        self.stats.current_time = self.stats.timer.elapsed().as_millis();
        self.stats.fps = 1000.0f64 / (self.stats.current_time - self.stats.last_time) as f64;
        self.stats.last_time = self.stats.current_time;
    }

    pub fn update(&mut self)
    {
        //apply pixels from raytracing to the buffer
        self.apply_pixels();

        //calc stats
        self.calc_fps();

        //check if complete
        if self.stats.current_time - self.stats.fps_display_update >= 1000
        {
            let is_done = self.rendering.is_done();
            let elapsed = self.rendering.check_and_get_elapsed_time() as f64 / 1000.0;

            //update window title
            self.sdl_set_new_window_title(elapsed, is_done);

            self.stats.fps_display_update = self.stats.current_time;

            if is_done && !self.stats.completed
            {
                println!("frame rendered ✅ (rendering time: {})", elapsed);
                self.stats.completed = true;

                //save
                self.save_image()
            }

            if !self.stats.completed
            {
                self.stats.pps = 0;
            }
        }

        //animation
        self.render_next_frame_if_possible();
    }

    pub fn cmd_loop(&mut self)
    {
        while !self.stats.completed
        {
            self.update();
        }

        println!("done");
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
        'main: loop
        {
            let mut events = vec![];

            //dumplicate events to new array to prevent referenceing issues with "self"
            {
                let sdl = self.sdl.as_mut().unwrap();
                for event in sdl.event_pump.poll_iter()
                {
                    events.push(event);
                }
            }

            for event in events
            {
                match event
                {
                    sdl2::event::Event::Quit { .. } =>
                        break 'main,
                    sdl2::event::Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>
                        break 'main,
                    sdl2::event::Event::KeyDown { keycode: Some(Keycode::Space), .. } =>
                    {
                        self.restart_rendering();
                    },
                    //get object at mous position
                    sdl2::event::Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } =>
                    {
                        let rt = self.raytracing.read().unwrap();
                        let pick_res = rt.pick(x, y);

                        if let Some(pick_res) = pick_res
                        {
                            dbg!(pick_res);
                        }
                    },
                    //restart rendering on resize
                    sdl2::event::Event::Window { win_event: WindowEvent::Resized(w, h), ..} =>
                    {
                        //apply
                        self.width = w;
                        self.height = h;

                        self.restart_rendering();

                        //save resolution to file
                        let mut file = File::create(POS_PATH).unwrap();
                        let _ = file.write(format!("{}x{}x{}x{}", self.window_x, self.window_y, self.width, self.height).as_bytes());
                    },
                    //save the window position
                    sdl2::event::Event::Window { win_event: WindowEvent::Moved(x, y), ..} =>
                    {
                        //apply changes
                        self.window_x = x;
                        self.window_y = y;

                        //save to file
                        let mut file = File::create(POS_PATH).unwrap();
                        let _ = file.write(format!("{}x{}x{}x{}", self.window_x, self.window_y, self.width, self.height).as_bytes());
                    },
                    _ => {},
                }
            }

            self.update();
        }

        println!("done");
    }

    pub fn sdl_set_new_window_title(&mut self, elapsed: f64, is_done: bool)
    {
        if let Some(sdl) = &mut self.sdl
        {
            let pixels = self.rendering.get_rendered_pixels();
            let percentage = (pixels as f32 / (self.width * self.height) as f32) * 100.0;

            let window = sdl.canvas.window_mut();

            let title = format!("Raytracer (FPS: {:.2}, PPS: {}, Frame: {}, Res: {}x{}, Complete: {:.2}%, Pixels: {}, Time: {:.2}s, Done: {})", self.stats.fps, self.stats.pps, self.stats.frame, self.width, self.height, percentage, pixels, elapsed, is_done);

            window.set_title(&title).unwrap();
        }
    }
}