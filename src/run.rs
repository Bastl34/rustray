extern crate rand;
extern crate image;

use chrono::{Datelike, Timelike, Utc, DateTime};

use std::sync::{RwLock, Arc};
use std::{io::Write, thread};
use std::time::{Instant, Duration};

use eframe::egui;
use egui_extras::RetainedImage;

use image::{ImageBuffer, RgbaImage, Rgba};

use std::fs::File;

use crate::renderer::RendererManager;
use crate::raytracing::Raytracing;
use crate::scene::Scene;

const IMAGE_PATH: &str = "data/output";
const ANIMATION_PATH: &str = "data/output/animation";
const POS_PATH: &str = "data/pos.data";

const DEFAULT_RES: (i32, i32) = (800, 600);

// ******************** Stats ********************
pub struct Stats
{
    last_time: u128,
    current_time: u128,

    frame: u64,

    timer: Instant,
    output_time: DateTime<Utc>,

    screen_update_time: u128,
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

            screen_update_time: 0,
            pps: 0,

            completed: false
        }
    }

    pub fn reset(&mut self)
    {
        self.last_time = 0;
        self.current_time = 0;
        self.frame = 0;

        self.timer = Instant::now();
        self.screen_update_time = 0;
        self.pps = 0;

        self.completed = false;
    }
}

// ******************** Run ********************
pub struct Run//<'a>
{
    width: i32,
    height: i32,

    window_x: i32,
    window_y: i32,

    window: bool,
    animate: bool,

    scenes_list: Vec<String>,

    image: RgbaImage,
    scene: Arc<RwLock<Scene>>,
    pub raytracing: Arc<RwLock<Raytracing>>,
    rendering: RendererManager,

    stats: Stats,

    help_printed: bool
}

impl Run
{
    pub fn new(width: i32, height: i32, window: bool, scenes_list: Vec<String>, animate: bool) -> Run
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

        {
            for scene_item in &self.scenes_list
            {
                scene.load(&scene_item);
            }

            scene.cam.init(self.width as u32, self.height as u32);
            scene.apply_frame(self.stats.frame);
            scene.print();
        }

        let rt_config = scene.raytracing_config;

        let scene = std::sync::Arc::new(std::sync::RwLock::new(scene));

        let mut raytracing = Raytracing::new(scene.clone());
        raytracing.config.apply(rt_config);

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
            //self.start_sdl();
            //self.sdl_loop();
            //self.start_egui();
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

    pub fn render_next_frame_if_possible(&mut self) -> bool
    {
        let mut render_next_frame = false;

        let has_animation;
        {
            let scene = self.scene.read().unwrap();
            has_animation = scene.animation.has_animation();
        }

        if !self.animate || !has_animation
        {
            return render_next_frame;
        }

        //check for next frame
        let has_next_frame;
        {
            let scene = self.scene.read().unwrap();
            has_next_frame = scene.frame_exists(self.stats.frame + 1);
        }

        //stop end get next frame;
        if self.stats.completed && has_next_frame
        {
            self.rendering.stop();
            thread::sleep(Duration::from_millis(100));

            {
                self.stats.frame += 1;
                self.scene.write().unwrap().apply_frame(self.stats.frame);
                render_next_frame = true
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
            let filename = format!("{}/output_{}-{}-{}_{}-{}-{}_%08d.png", ANIMATION_PATH, self.stats.output_time.year(), self.stats.output_time.month(), self.stats.output_time.day(), self.stats.output_time.hour(), self.stats.output_time.minute(), self.stats.output_time.second());
            let filename_animation = format!("{}/output_{}-{}-{}_{}-{}-{}", ANIMATION_PATH, self.stats.output_time.year(), self.stats.output_time.month(), self.stats.output_time.day(), self.stats.output_time.hour(), self.stats.output_time.minute(), self.stats.output_time.second());
            let fps = self.scene.read().unwrap().animation.fps;

            println!("");
            println!("use ffmpeg to combine frames:");
            println!(" - for mp4:  ffmpeg -i \"{}\" -c:v libx264 -vf fps={} \"{}.mp4\"", filename, fps, filename_animation);
            println!(" - for gif:  ffmpeg -i \"{}\" -vf fps={} \"{}.gif\"", filename, fps, filename_animation);
            println!(" - for webp: ffmpeg -i \"{}\" -vcodec libwebp -lossless 0 -loop 0 -an -vf fps={} \"{}.webp\"", filename, fps, filename_animation);
            println!("");

            self.help_printed = true;
        }

        render_next_frame
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
        let print_str;
        { print_str = format!("frame: {}/{}", self.stats.frame + 1, frames); }

        println!("{}", print_str);
        for _ in 0..print_str.len() { print!("="); }
        println!("");
    }

    pub fn apply_pixels(&mut self) -> bool
    {
        let receiver = self.rendering.get_message_receiver();
        let mut change = false;

        loop
        {
            let res = receiver.try_recv();

            if res.is_err() { break }

            if res.is_ok()
            {
                let item = res.unwrap();

                //check range to prevent draing something outside while resizing
                if item.x < self.image.width() as i32 && item.y < self.image.height() as i32
                {
                    self.image.put_pixel(item.x as u32, item.y as u32, Rgba([item.r, item.g, item.b, 255]));
                    self.stats.pps += 1;
                    change = true;
                }
            }
        }

        change
    }

    pub fn save_image(&mut self)
    {
        let mut out_dir = IMAGE_PATH;

        let has_animation;
        {
            let scene = self.scene.read().unwrap();
            has_animation = scene.animation.has_animation();
        }

        {
            if self.animate && has_animation
            {
                out_dir = ANIMATION_PATH;
            }
        }

        let filename = format!("{}/output_{}-{}-{}_{}-{}-{}_{:0>8}.png", out_dir, self.stats.output_time.year(), self.stats.output_time.month(), self.stats.output_time.day(), self.stats.output_time.hour(), self.stats.output_time.minute(), self.stats.output_time.second(), self.stats.frame);
        self.image.save(filename).unwrap();
    }

    pub fn loop_update(&mut self) -> bool
    {
        //apply pixels from raytracing to the buffer
        let mut change = self.apply_pixels();

        //check if complete
        if self.stats.current_time - self.stats.screen_update_time >= 1000
        {
            let is_done = self.rendering.is_done();
            let elapsed = self.rendering.check_and_get_elapsed_time() as f64 / 1000.0;

            self.stats.screen_update_time = self.stats.current_time;

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
        change = self.render_next_frame_if_possible() || change;

        change
    }

    pub fn cmd_loop(&mut self)
    {
        while !self.stats.completed
        {
            self.loop_update();
        }

        println!("done");
    }

    pub fn get_egui_options(&self) -> eframe::NativeOptions
    {
        eframe::NativeOptions
        {
            initial_window_size: Some(egui::vec2(self.width as f32, self.height as f32)),
            initial_window_pos: Some(egui::pos2(self.window_x as f32, self.window_y as f32)),
            ..Default::default()
        }
    }
}

// ******************** GUI ********************

fn image_to_retained_image(image: RgbaImage) -> RetainedImage
{
    let pixels = image.as_flat_samples();

    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        pixels.as_slice(),
    );

    RetainedImage::from_color_image("test", color_image)
}

impl eframe::App for Run
{
    fn clear_color(&self, _visuals: &egui::Visuals) -> egui::Rgba
    {
        egui::Rgba::WHITE
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame)
    {
        let some_change = self.loop_update();

        if some_change
        {
            ctx.request_repaint();
        }

        self.update_gui(ctx, frame);
        self.update_states(ctx, frame);
    }
}

impl Run
{
    fn update_gui(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        //main
        let main_frame = egui::containers::Frame
        {
            ..egui::containers::Frame::default()
        };

        egui::CentralPanel::default().frame(main_frame).show(ctx, |ui|
        {
            let image = self.image.clone();
            let image = image_to_retained_image(image);

            image.show(ui);
        });

        let bottom_frame = egui::containers::Frame {
            inner_margin: egui::style::Margin { left: 4., right: 4., top: 4., bottom: 2. },
            fill: egui::Color32::from_rgba_premultiplied(215, 215, 215, 215),
            ..egui::containers::Frame::default()
        };

        egui::TopBottomPanel::bottom("bottom_panel").frame(bottom_frame).show(ctx, |ui|
        {
            ui.vertical(|ui|
            {
                let is_done = self.rendering.is_done();
                let elapsed = self.rendering.check_and_get_elapsed_time() as f64 / 1000.0;

                let pixels = self.rendering.get_rendered_pixels();
                let progress = pixels as f32 / (self.width * self.height) as f32;

                let status = format!("PPS: {}, Frame: {}, Res: {}x{}, Pixels: {}, Time: {:.2}s, Done: {}", self.stats.pps, self.stats.frame, self.width, self.height, pixels, elapsed, is_done);
                ui.label(status);

                let progress_bar = egui::ProgressBar::new(progress)
                    .show_percentage();
                    //.animate(self.rendering);
                ui.add(progress_bar);
            });
        });
    }

    fn update_states(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame)
    {
        let window_info = frame.info.window_info.clone();

        if ctx.input_mut().pointer.primary_clicked()
        {
            let x = ctx.input_mut().pointer.interact_pos().unwrap().x as i32;
            let y = ctx.input_mut().pointer.interact_pos().unwrap().y as i32;

            let rt = self.raytracing.read().unwrap();
            let pick_res = rt.pick(x, y);

            if let Some(pick_res) = pick_res
            {
                dbg!(pick_res);
            }
        }

        if let Some(window_info) = window_info
        {
            let x = window_info.position.x as i32;
            let y = window_info.position.y as i32;

            let w = window_info.size.x as i32;
            let h = window_info.size.y as i32;

            //save the window position
            if x != self.window_x || y != self.window_y
            {
                //apply changes
                self.window_x = x;
                self.window_y = y;

                //save to file
                let mut file = File::create(POS_PATH).unwrap();
                let _ = file.write(format!("{}x{}x{}x{}", self.window_x, self.window_y, self.width, self.height).as_bytes());
            }

            //restart rendering on resize
            if w != self.width || h != self.height
            {
                //apply
                self.width = w;
                self.height = h;

                self.restart_rendering();

                //save resolution to file
                let mut file = File::create(POS_PATH).unwrap();
                let _ = file.write(format!("{}x{}x{}x{}", self.window_x, self.window_y, self.width, self.height).as_bytes());
            }
        }
    }
}