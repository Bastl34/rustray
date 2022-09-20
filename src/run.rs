extern crate rand;
extern crate image;

use chrono::{Datelike, Timelike, Utc, DateTime};
use egui::{Color32, ScrollArea};
use nalgebra::Vector3;

use std::{fs};
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
const SCENE_PATH: &str = "scene/";

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
    pps_current: u64,
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
            pps_current: 0,
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
        self.pps_current = 0;
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

    dir_scenes_list: Vec<String>,
    rendering_scenes_list: Vec<String>,

    image: RgbaImage,
    scene: Arc<RwLock<Scene>>,
    pub raytracing: Arc<RwLock<Raytracing>>,
    rendering: RendererManager,

    stats: Stats,

    stopped: bool,

    help_printed: bool,
    selected_scene_id: usize
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

            dir_scenes_list: vec![],
            rendering_scenes_list: scenes_list,

            scene: scene,
            raytracing: rt,
            rendering: rendering,

            image: ImageBuffer::new(0, 0),

            stats: Stats::new(),

            stopped: false,

            help_printed: false,
            selected_scene_id: 0
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
            for scene_item in &self.rendering_scenes_list
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

            if splits_arr.len() == 4
            {
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

        self.read_scenes_from_dir();
    }

    pub fn read_scenes_from_dir(&mut self)
    {
        self.dir_scenes_list.clear();

        let paths = fs::read_dir(SCENE_PATH).unwrap();

        for path in paths
        {
            if path.is_ok()
            {
                let path = path.unwrap();

                if path.file_type().is_ok() && path.file_type().unwrap().is_file()
                {
                    self.dir_scenes_list.push(path.file_name().to_str().unwrap().to_string());
                }
            }
        }
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
        if !self.window
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
                    self.stats.pps_current += 1;
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

        //stats
        self.stats.current_time = self.stats.timer.elapsed().as_millis();
        self.stats.last_time = self.stats.current_time;

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
                self.stats.pps = self.stats.pps_current;
                self.stats.pps_current = 0;
            }
        }

        //animation
        if !self.stopped
        {
            let has_next = self.render_next_frame_if_possible();

            if self.stats.completed && !has_next
            {
                self.stopped = true;
            }

            change = has_next || change;
        }

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
        self.loop_update();

        //force update (otherwise animation is somehow not working)
        ctx.request_repaint();

        self.update_gui(ctx, frame);
        self.update_states(ctx, frame);
    }
}

impl Run
{
    fn update_gui(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        // ********** main **********
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

        // ********** settings **********
        egui::Window::new("Settings").show(ctx, |ui|
        {
            let running = self.rendering.is_running();
            let is_done = self.rendering.is_done();

            let settings_updates_allowed = !(running && !is_done);

            let samples;
            let mut samples_new;

            let monte_carlo;
            let mut monte_carlo_new;

            let threads;
            let mut threads_new;

            let focal_length;
            let mut focal_length_new;

            let aperture_size;
            let mut aperture_size_new;

            let fog_density;
            let mut fog_density_new;

            let fog_color;
            let mut fog_color_new;

            let max_recursion;
            let mut max_recursion_new;

            let gamma_correction;
            let mut gamma_correction_new;

            {
                let rt = self.raytracing.read().unwrap();
                monte_carlo = rt.config.monte_carlo;
                monte_carlo_new = rt.config.monte_carlo;

                samples = rt.config.samples;
                samples_new = rt.config.samples;

                threads = self.rendering.thread_amount;
                threads_new = self.rendering.thread_amount;

                focal_length = rt.config.focal_length;
                focal_length_new = rt.config.focal_length;

                aperture_size = rt.config.aperture_size;
                aperture_size_new = rt.config.aperture_size;

                fog_density = rt.config.fog_density;
                fog_density_new = rt.config.fog_density;

                let r = (rt.config.fog_color.x * 255.0) as u8;
                let g = (rt.config.fog_color.y * 255.0) as u8;
                let b = (rt.config.fog_color.z * 255.0) as u8;
                fog_color = Color32::from_rgb(r, g, b);
                fog_color_new = fog_color;

                max_recursion = rt.config.max_recursion;
                max_recursion_new = rt.config.max_recursion;

                gamma_correction = rt.config.gamma_correction;
                gamma_correction_new = rt.config.gamma_correction;
            }

            ui.add_enabled_ui(settings_updates_allowed, |ui|
            {
                // ********** Scene **********
                ui.heading("Scene");

                let mut selected_scene_id_new = self.selected_scene_id;

                let mut list_copy = self.dir_scenes_list.clone();
                list_copy.insert(0, " ~~~ select scene ~~~ ".to_string());
                let items = list_copy.as_slice();

                egui::ComboBox::from_label("").width(200.0).show_index(
                    ui,
                    &mut selected_scene_id_new,
                    items.len(),
                    |i| items[i].to_owned()
                );
                if selected_scene_id_new != self.selected_scene_id
                {
                    self.selected_scene_id = selected_scene_id_new;

                    let new_scene = SCENE_PATH.to_string() + self.dir_scenes_list[self.selected_scene_id - 1].clone().as_str();

                    self.rendering_scenes_list.clear();
                    self.rendering_scenes_list.push(new_scene);
                    self.init_raytracing();
                }
                ui.separator();

                // ********** rendering settings **********
                ui.heading("Rendering Settings");

                ui.add(egui::Slider::new(&mut samples_new, 1..=1024).text("samples"));
                ui.checkbox(&mut self.animate, "Animation");
                ui.checkbox(&mut monte_carlo_new, "Monte Carlo");

                let max_threads = num_cpus::get() as u32;
                ui.add(egui::Slider::new(&mut threads_new, 1..=max_threads).text("CPU threads"));
                ui.separator();

                // ********** scene settings **********
                ui.heading("Scene Settings");

                ui.add(egui::Slider::new(&mut focal_length_new, 1.0..=128.0).text("focal length (unit)"));
                ui.add(egui::Slider::new(&mut aperture_size_new, 1.0..=128.0).text("aperture size (in px)"));

                ui.add(egui::Slider::new(&mut fog_density_new, 0.0..=1.0).text("fog density (amount)"));

                ui.horizontal(|ui| {
                    ui.label("fog color:");
                    ui.color_edit_button_srgba(&mut fog_color_new);
                });

                ui.add(egui::Slider::new(&mut max_recursion_new, 1..=64).text("max recursion"));
                ui.checkbox(&mut gamma_correction_new, "gamma correction");

                ui.separator();

                // ********** scene items **********
                ui.heading("Scene Items");

                let scene_items;
                {
                    scene_items = self.scene.read().unwrap().items.len();
                }

                let mut height = 10.0;
                if scene_items > 0
                {
                    height = 200.0;
                }

                let scroll_area = ScrollArea::vertical().max_height(height).auto_shrink([false; 2]);

                scroll_area.show(ui, |ui|
                {
                    ui.vertical(|ui|
                    {
                        let mut scene_items = vec![];
                        {
                            let scene = self.scene.read().unwrap();
                            for item in & scene.items
                            {
                                scene_items.push((item.get_basic().id, item.get_basic().name.clone()));
                            }
                        }

                        for item in & scene_items
                        {
                            ui.collapsing(format!("{}: {}", item.0, item.1), |ui|
                            {
                                ui.horizontal_wrapped(|ui|
                                {
                                    let mut visible;
                                    {
                                        let scene = self.scene.read().unwrap();
                                        let item = scene.get_by_id(item.0).unwrap();
                                        visible = item.get_basic().visible;
                                    }

                                    if ui.checkbox(&mut visible, "Visible").changed()
                                    {
                                        let mut scene = self.scene.write().unwrap();
                                        let item = scene.get_by_id_mut(item.0).unwrap();
                                        item.get_basic_mut().visible = visible;
                                    };
                                });
                            });
                            ui.end_row();
                        }
                    });
                })
                .inner;

                ui.separator();
            });

            if samples != samples_new { self.raytracing.write().unwrap().config.samples = samples_new; }
            if monte_carlo != monte_carlo_new { self.raytracing.write().unwrap().config.monte_carlo = monte_carlo_new; }
            if threads != threads_new { self.rendering.thread_amount = threads_new; }

            if focal_length != focal_length_new { self.raytracing.write().unwrap().config.focal_length = focal_length_new; }
            if aperture_size != aperture_size_new { self.raytracing.write().unwrap().config.aperture_size = aperture_size_new; }
            if fog_density != fog_density_new { self.raytracing.write().unwrap().config.fog_density = fog_density_new; }
            if fog_color != fog_color_new
            {
                let r = ((fog_color_new.r() as f32) / 255.0).clamp(0.0, 1.0);
                let g = ((fog_color_new.g() as f32) / 255.0).clamp(0.0, 1.0);
                let b = ((fog_color_new.b() as f32) / 255.0).clamp(0.0, 1.0);
                self.raytracing.write().unwrap().config.fog_color = Vector3::<f32>::new(r, g, b);
            }
            if max_recursion != max_recursion_new { self.raytracing.write().unwrap().config.max_recursion = max_recursion_new; }
            if gamma_correction != gamma_correction_new { self.raytracing.write().unwrap().config.gamma_correction = gamma_correction_new; }

            if running && !is_done
            {
                if ui.button("Stop Rendering").clicked()
                {
                    self.rendering.stop();
                    self.stopped = true;
                }
            }
            else
            {
                if ui.button("Start Rendering").clicked()
                {
                    self.restart_rendering();
                    self.stopped = false;
                }
            }
        });

        // ********** status **********
        let bottom_frame = egui::containers::Frame
        {
            inner_margin: egui::style::Margin { left: 4., right: 4., top: 4., bottom: 2. },
            fill: egui::Color32::from_rgba_premultiplied(215, 215, 215, 215),
            ..egui::containers::Frame::default()
        };

        egui::TopBottomPanel::bottom("bottom_panel").frame(bottom_frame).show(ctx, |ui|
        {
            ui.vertical(|ui|
            {
                let is_done = self.rendering.is_done();

                let elapsed;
                if !self.stopped || is_done
                {
                    elapsed = self.rendering.check_and_get_elapsed_time() as f64 / 1000.0;
                }
                else
                {
                    elapsed = 0.0;
                }

                let pixels = self.rendering.get_rendered_pixels();
                let progress = pixels as f32 / (self.width * self.height) as f32;

                let mut frames = 1;
                {
                    let scene = self.scene.read().unwrap();
                    if scene.animation.has_animation()
                    {
                        frames = scene.animation.get_frames_amount_to_render();
                    }
                }

                let status = format!("PPS: {}, Frame: {}/{}, Res: {}x{}, Pixels: {}, Time: {:.2}s, Done: {}", self.stats.pps, self.stats.frame + 1, frames, self.width, self.height, pixels, elapsed, is_done);
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
        let window_info = frame.info().window_info.clone();

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


        if let Some(position) = window_info.position
        {
            let x = position.x as i32;
            let y = position.y as i32;

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
        }

        let w = window_info.size.x as i32;
        let h = window_info.size.y as i32;

        //restart rendering on resize
        if w != self.width || h != self.height
        {
            //apply
            self.width = w;
            self.height = h;

            let running = self.rendering.is_running();
            let is_done = self.rendering.is_done();
            let is_running = running && !is_done;

            if is_running
            {
                self.restart_rendering();
            }

            //save resolution to file
            let mut file = File::create(POS_PATH).unwrap();
            let _ = file.write(format!("{}x{}x{}x{}", self.window_x, self.window_y, self.width, self.height).as_bytes());
        }

    }
}