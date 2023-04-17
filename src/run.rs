extern crate rand;
extern crate image;

use chrono::{Datelike, Timelike, Utc, DateTime};
use egui::{Color32, ScrollArea, RichText, Modifiers, Ui};
use nalgebra::{Vector3};
use rfd::FileDialog;


use std::f32::consts::PI;
use std::{fs};
use std::sync::{RwLock, Arc, Mutex};
use std::{io::Write, thread};
use std::time::{Instant, Duration};

use eframe::egui;
use egui_extras::RetainedImage;

use image::{ImageBuffer, RgbaImage, Rgba};

use std::fs::File;

use crate::camera::Camera;
use crate::post_processing::run_post_processing;
use crate::renderer::RendererManager;
use crate::raytracing::Raytracing;
use crate::scene::{Scene, LightType, ScemeItem};
use crate::shape::{TextureType, Material};

const IMAGE_PATH: &str = "data/output";
const ANIMATION_PATH: &str = "data/output/animation";
const POS_PATH: &str = "data/pos.data";
const SCENE_PATH: &str = "scene/";

const DEFAULT_RES: (i32, i32) = (800, 600);

#[derive(PartialEq)]
pub enum SceneLoadType
{
    Loading,
    RaytracingInitNeeded,
    Complete,
}

// ******************** Stats ********************
pub struct Stats
{
    last_time: u128,
    current_time: u128,

    frame: u64,

    timer: Instant,
    pub output_time: DateTime<Utc>,

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

    ui_visible: bool,

    loading_scene: Arc<Mutex<SceneLoadType>>,
    start_after_init: bool,

    dir_scenes_list: Vec<String>,
    rendering_scenes_list: Vec<String>,

    image: RgbaImage,
    normals: Vec<Vector3<f32>>, // access data via: y * w + x
    depth: Vec<f32>,
    objects: Vec<u32>,

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
    pub fn new(width: i32, height: i32, window: bool, scenes_list: Vec<String>, animate: bool, start_after_init: bool) -> Run
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

            ui_visible: true,

            loading_scene: std::sync::Arc::new(std::sync::Mutex::new(SceneLoadType::RaytracingInitNeeded)),

            dir_scenes_list: vec![],
            rendering_scenes_list: scenes_list,
            start_after_init: start_after_init,

            scene: scene,
            raytracing: rt,
            rendering: rendering,

            image: ImageBuffer::new(0, 0),
            normals: vec![],
            depth: vec![],
            objects: vec![],

            stats: Stats::new(),

            stopped: false,

            help_printed: false,
            selected_scene_id: 0
        }
    }

    pub fn init_image(&mut self)
    {
        let w = self.width as usize;
        let h = self.height as usize;

        self.image = ImageBuffer::new(w as u32, h as u32);
        self.normals = vec![Vector3::<f32>::zeros(); w * h];
        self.depth = vec![0.0; w * h];
        self.objects = vec![0; w * h];
    }

    pub fn init_stats(&mut self)
    {
        self.stats.reset();
    }

    pub fn init_scene(&mut self)
    {
        { *(self.loading_scene.lock().unwrap()) = SceneLoadType::Loading; }

        let width = self.width;
        let height = self.height;
        let frame = self.stats.frame;

        let rendering_scenes_list = self.rendering_scenes_list.clone();

        let loading_scene_mutex = Arc::clone(&self.loading_scene);

        let scene_arc = self.scene.clone();

        let thread_func = move ||
        {
            let mut scene = Scene::new();
            scene.clear();

            {
                scene.raytracing_config.apply(scene_arc.read().unwrap().raytracing_config);
                scene.post_processing = scene_arc.read().unwrap().post_processing.clone();
            }

            {
                for scene_item in &rendering_scenes_list
                {
                    scene.load(&scene_item);
                }

                scene.cam.init(width as u32, height as u32);
                scene.find_and_set_default_env_if_needed();
                scene.apply_frame(frame);
                scene.print();
            }

            *scene_arc.write().unwrap() = scene;

            { *(loading_scene_mutex.lock().unwrap()) = SceneLoadType::RaytracingInitNeeded; }
        };

        if self.window
        {
            std::thread::spawn(thread_func);
        }
        else
        {
            thread_func();
        }
    }

    pub fn init_raytracing_if_needed(&mut self)
    {
        let rt_init_needed;
        { rt_init_needed = *(self.loading_scene.lock().unwrap()) == SceneLoadType::RaytracingInitNeeded; }

        if rt_init_needed
        {
            let scene = self.scene.clone();

            let rt_config = scene.read().unwrap().raytracing_config;
            let post_processing = scene.read().unwrap().post_processing.clone();

            //let scene = std::sync::Arc::new(std::sync::RwLock::new(scene));

            let mut raytracing = Raytracing::new(scene.clone());
            raytracing.config.apply(rt_config);
            raytracing.post_processing = post_processing;

            let raytracing_arc = std::sync::Arc::new(std::sync::RwLock::new(raytracing));

            let rendering = RendererManager::new(self.width, self.height, raytracing_arc.clone());

            self.scene = scene;
            self.raytracing = raytracing_arc;
            self.rendering = rendering;

            { *(self.loading_scene.lock().unwrap()) = SceneLoadType::Complete; }

            let scene_len;
            {
                scene_len = self.scene.read().unwrap().items.len();
            }

            if self.start_after_init == true && scene_len > 0
            {
                self.start_after_init = false;
                self.restart_rendering();
            }
        }
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
        self.init_scene();
        self.init_raytracing_if_needed();

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

        self.dir_scenes_list.sort();
    }

    pub fn start(&mut self)
    {
        //print rt info
        { self.raytracing.read().unwrap().print_config(); }

        //some stats
        self.print_frame_info();

        //start
        let scene_items;
        {
            scene_items = self.scene.read().unwrap().items.len();
        }
        if scene_items > 0
        {
            { self.scene.write().unwrap().update(); }
            self.rendering.start();
        }
        else
        {
            println!("no items to render");
            self.stopped = true;
        }

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
        self.stats.output_time = Utc::now();

        //print some stats
        self.print_frame_info();

        //restart
        self.init_image();
        { self.scene.write().unwrap().update(); }
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
            self.init_image();
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
                    let x = item.x as usize;
                    let y = item.y as usize;
                    let w = self.image.width() as usize;
                    let color = Rgba([item.r, item.g, item.b, 255]);
                    let normal = item.normal;
                    let depth = item.depth;
                    let object = item.object_id;

                    //let color = Rgba([(item.normal.x * 255.0) as u8, (item.normal.y * 255.0) as u8, (item.normal.z * 255.0) as u8, 255]);
                    self.image.put_pixel(x as u32, y as u32, color);
                    self.normals[y * w + x] = normal;
                    self.depth[y * w + x] = depth;
                    self.objects[y * w + x] = object;

                    self.stats.pps_current += 1;
                    change = true;
                }
            }
        }

        change
    }

    pub fn save_image(&mut self, postfix: Option<&str>)
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

        //let filename = format!("{}/output_{}-{}-{}_{}-{}-{}_{:0>8}.png", out_dir, self.stats.output_time.year(), self.stats.output_time.month(), self.stats.output_time.day(), self.stats.output_time.hour(), self.stats.output_time.minute(), self.stats.output_time.second(), self.stats.frame);
        let mut filename = format!("{}/output_{}-{}-{}_{}-{}-{}_{:0>8}", out_dir, self.stats.output_time.year(), self.stats.output_time.month(), self.stats.output_time.day(), self.stats.output_time.hour(), self.stats.output_time.minute(), self.stats.output_time.second(), self.stats.frame);
        if postfix.is_some()
        {
            filename = format!("{}_{}.png", filename, postfix.unwrap());
        }
        else
        {
            filename = format!("{}.png", filename);
        }


        let res = self.image.save(&filename);

        if res.is_err()
        {
            println!("error on saving image to {}", &filename);
        }
        else
        {
            println!("image saved to {}", &filename);
        }
    }

    pub fn post_processing(&mut self)
    {
        let config;
        let cam: Camera;
        {
            config = self.raytracing.read().unwrap().post_processing.clone();
            cam = self.raytracing.read().unwrap().scene.read().unwrap().cam.clone();
        }

        let processed_image = run_post_processing(config, &self.image, &self.normals, &self.depth, &self.objects, &cam);
        self.image = processed_image.clone();
        self.save_image(Some("post"));
    }

    pub fn loop_update(&mut self) -> bool
    {
        //init raytracing if needed
        self.init_raytracing_if_needed();

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
                self.save_image(None)
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
            drag_and_drop_support: true,
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
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4]
    {
        egui::Rgba::WHITE.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame)
    {
        self.loop_update();

        //force update (otherwise animation is somehow not working)
        ctx.request_repaint();

        self.handle_file_drop(ctx);
        self.update_gui(ctx, frame);
        self.update_states(ctx, frame);
    }
}

impl Run
{
    fn handle_file_drop(&mut self, ctx: &egui::Context)
    {
        let dropped_files  = ctx.input(|i| i.raw.dropped_files.clone());

        if !dropped_files.is_empty()
        {
            self.rendering_scenes_list.clear();

            for file in dropped_files
            {
                self.rendering_scenes_list.push(file.path.unwrap().to_str().unwrap().to_string());
            }

            self.init_scene();
        }
    }

    fn show_material_setting(&mut self, ui: &mut Ui, material_id: u32)
    {
        ui.vertical(|ui|
        {
            // material settings
            let material_name;
            let mut alpha;
            let mut shininess;
            let mut reflectivity;
            let mut refraction_index;
            let mut normal_map_strength;
            let mut cast_shadow;
            let mut receive_shadow;
            let mut shadow_softness;
            let mut roughness;
            let mut monte_carlo;
            let mut smooth_shading;
            let mut reflection_only;
            let mut backface_cullig;

            let mut ambient_color;
            let mut base_color;
            let mut specular_color;

            {
                let scene = self.scene.read().unwrap();
                let mat_arc = scene.get_material_by_id(material_id).unwrap();
                let mat = mat_arc.read().unwrap();

                material_name = mat.name.clone();
                alpha = mat.alpha;
                shininess = mat.shininess;
                reflectivity = mat.reflectivity;
                refraction_index = mat.refraction_index;
                normal_map_strength = mat.normal_map_strength;
                cast_shadow = mat.cast_shadow;
                receive_shadow = mat.receive_shadow;
                shadow_softness = mat.shadow_softness;
                roughness = mat.roughness;
                monte_carlo = mat.monte_carlo;
                smooth_shading = mat.smooth_shading;
                reflection_only = mat.reflection_only;
                backface_cullig = mat.backface_cullig;

                let r = (mat.ambient_color.x * 255.0) as u8;
                let g = (mat.ambient_color.y * 255.0) as u8;
                let b = (mat.ambient_color.z * 255.0) as u8;
                ambient_color = Color32::from_rgb(r, g, b);

                let r = (mat.base_color.x * 255.0) as u8;
                let g = (mat.base_color.y * 255.0) as u8;
                let b = (mat.base_color.z * 255.0) as u8;
                base_color = Color32::from_rgb(r, g, b);

                let r = (mat.specular_color.x * 255.0) as u8;
                let g = (mat.specular_color.y * 255.0) as u8;
                let b = (mat.specular_color.z * 255.0) as u8;
                specular_color = Color32::from_rgb(r, g, b);
            }

            let mut apply_settings = false;

            apply_settings = ui.add(egui::Slider::new(&mut alpha, 0.0..=1.0).text("alpha")).changed() || apply_settings;
            apply_settings = ui.add(egui::Slider::new(&mut shininess, 0.0..=1.0).text("shininess")).changed() || apply_settings;
            apply_settings = ui.add(egui::Slider::new(&mut reflectivity, 0.0..=1.0).text("reflectivity")).changed() || apply_settings;
            apply_settings = ui.add(egui::Slider::new(&mut refraction_index, 1.0..=5.0).text("refraction index")).changed() || apply_settings;
            apply_settings = ui.add(egui::Slider::new(&mut normal_map_strength, 0.0..=100.0).text("normal map strength")).changed() || apply_settings;
            apply_settings = ui.checkbox(&mut cast_shadow, "cast shadow").changed() || apply_settings;
            apply_settings = ui.checkbox(&mut receive_shadow, "receive shadow").changed() || apply_settings;
            apply_settings = ui.add(egui::Slider::new(&mut shadow_softness, 0.0..=100.0).text("shadow softness")).changed() || apply_settings;
            apply_settings = ui.add(egui::Slider::new(&mut roughness, 0.0..=PI/2.0).text("roughness")).changed() || apply_settings;
            apply_settings = ui.checkbox(&mut monte_carlo, "monte carlo").changed() || apply_settings;
            apply_settings = ui.checkbox(&mut smooth_shading, "smooth shading").changed() || apply_settings;
            apply_settings = ui.checkbox(&mut reflection_only, "reflection only").changed() || apply_settings;
            apply_settings = ui.checkbox(&mut backface_cullig, "backface cullig").changed() || apply_settings;

            ui.horizontal(|ui|
            {
                ui.label("ambient color:");
                apply_settings = ui.color_edit_button_srgba(&mut ambient_color).changed() || apply_settings;
            });

            ui.horizontal(|ui|
            {
                ui.label("base color:");
                apply_settings = ui.color_edit_button_srgba(&mut base_color).changed() || apply_settings;
            });

            ui.horizontal(|ui|
            {
                ui.label("specular color:");
                apply_settings = ui.color_edit_button_srgba(&mut specular_color).changed() || apply_settings;
            });


            if apply_settings
            {
                let mut scene = self.scene.write().unwrap();
                let mat_arc = scene.get_material_by_id_mut(material_id).unwrap();
                let mut mat = mat_arc.write().unwrap();

                mat.alpha = alpha;
                mat.shininess = shininess;
                mat.reflectivity = reflectivity;
                mat.refraction_index = refraction_index;
                mat.normal_map_strength = normal_map_strength;
                mat.cast_shadow = cast_shadow;
                mat.receive_shadow = receive_shadow;
                mat.shadow_softness = shadow_softness;
                mat.roughness = roughness;
                mat.monte_carlo = monte_carlo;
                mat.smooth_shading = smooth_shading;
                mat.reflection_only = reflection_only;
                mat.backface_cullig = backface_cullig;

                let r = ((ambient_color.r() as f32) / 255.0).clamp(0.0, 1.0);
                let g = ((ambient_color.g() as f32) / 255.0).clamp(0.0, 1.0);
                let b = ((ambient_color.b() as f32) / 255.0).clamp(0.0, 1.0);
                mat.ambient_color = Vector3::<f32>::new(r, g, b);

                let r = ((base_color.r() as f32) / 255.0).clamp(0.0, 1.0);
                let g = ((base_color.g() as f32) / 255.0).clamp(0.0, 1.0);
                let b = ((base_color.b() as f32) / 255.0).clamp(0.0, 1.0);
                mat.base_color = Vector3::<f32>::new(r, g, b);

                let r = ((specular_color.r() as f32) / 255.0).clamp(0.0, 1.0);
                let g = ((specular_color.g() as f32) / 255.0).clamp(0.0, 1.0);
                let b = ((specular_color.b() as f32) / 255.0).clamp(0.0, 1.0);
                mat.specular_color = Vector3::<f32>::new(r, g, b);
            }


            // ********** textures
            ui.collapsing("Textures", |ui|
            {
                // labels
                let mut ambient_texture_label: String = "unset".to_string();
                let mut base_texture_label: String = "unset".to_string();
                let mut specular_texture_label: String = "unset".to_string();
                let mut normal_texture_label: String = "unset".to_string();
                let mut alpha_texture_label: String = "unset".to_string();
                let mut roughness_texture_label: String = "unset".to_string();
                let mut ao_texture_label: String = "unset".to_string();
                let mut reflectivity_texture_label: String = "unset".to_string();

                let has_ambient;
                let has_base;
                let has_specular;
                let has_normal;
                let has_alpha;
                let has_roughness;
                let has_ao;
                let has_reflectivity;

                {
                    let scene = self.scene.read().unwrap();
                    let mat_arc = scene.get_material_by_id(material_id).unwrap();
                    let material = mat_arc.write().unwrap();

                    has_ambient = material.texture_ambient.width() > 0;
                    has_base = material.texture_base.width() > 0;
                    has_specular = material.texture_specular.width() > 0;
                    has_normal = material.texture_normal.width() > 0;
                    has_alpha = material.texture_alpha.width() > 0;
                    has_roughness = material.texture_roughness.width() > 0;
                    has_ao = material.texture_ambient_occlusion.width() > 0;
                    has_reflectivity = material.texture_reflectivity.width() > 0;

                    if has_ambient { ambient_texture_label = format!("{}x{}", material.texture_ambient.width(), material.texture_ambient.height()); }
                    if has_base { base_texture_label = format!("{}x{}", material.texture_base.width(), material.texture_base.height()); }
                    if has_specular { specular_texture_label = format!("{}x{}", material.texture_specular.width(), material.texture_specular.height()); }
                    if has_normal { normal_texture_label = format!("{}x{}", material.texture_normal.width(), material.texture_normal.height()); }
                    if has_alpha { alpha_texture_label = format!("{}x{}", material.texture_alpha.width(), material.texture_alpha.height()); }
                    if has_roughness { roughness_texture_label = format!("{}x{}", material.texture_roughness.width(), material.texture_roughness.height()); }
                    if has_ao { ao_texture_label = format!("{}x{}", material.texture_ambient_occlusion.width(), material.texture_ambient_occlusion.height()); }
                    if has_reflectivity { reflectivity_texture_label = format!("{}x{}", material.texture_reflectivity.width(), material.texture_reflectivity.height()); }
                }

                let mut tex_items = vec![];
                tex_items.push(("ambient texture", has_ambient, ambient_texture_label, TextureType::AmbientEmissive));
                tex_items.push(("base texture", has_base, base_texture_label, TextureType::Base));
                tex_items.push(("specular texture", has_specular, specular_texture_label, TextureType::Specular));
                tex_items.push(("normal texture", has_normal, normal_texture_label, TextureType::Normal));
                tex_items.push(("alpha texture", has_alpha, alpha_texture_label, TextureType::Alpha));
                tex_items.push(("roughness texture", has_roughness, roughness_texture_label, TextureType::Roughness));
                tex_items.push(("ambient occlusion texture", has_ao, ao_texture_label, TextureType::AmbientOcclusion));
                tex_items.push(("reflectivity texture", has_reflectivity, reflectivity_texture_label, TextureType::Reflectivity));

                for tex in tex_items
                {
                    ui.horizontal(|ui|
                    {
                        ui.label(format!("{}:", tex.0));

                        if tex.1
                        {
                            ui.label(RichText::new(tex.2).strong());
                        }
                        else
                        {
                            ui.label(RichText::new(tex.2));
                        }

                        if ui.button("+").clicked()
                        {
                            if let Some(path) = FileDialog::new().add_filter("Image", &["jpg", "png"]).set_directory("/").pick_file()
                            {
                                let mut scene = self.scene.write().unwrap();
                                let mat_arc = scene.get_material_by_id_mut(material_id).unwrap();
                                let mut material = mat_arc.write().unwrap();

                                material.load_texture(&path.display().to_string(), tex.3);
                            }
                        }

                        if tex.1 && ui.button("🗑").clicked()
                        {
                            let mut scene = self.scene.write().unwrap();
                            let mat_arc = scene.get_material_by_id_mut(material_id).unwrap();
                            let mut material = mat_arc.write().unwrap();

                            material.remove_texture(tex.3);
                        }
                    });
                }
            });
        });
    }

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

        if !self.ui_visible
        {
            return;
        }

        // ********** settings **********
        egui::Window::new("Settings").show(ctx, |ui|
        {
            let running = self.rendering.is_running();
            let is_done = self.rendering.is_done();

            let scene_items;
            let light_items;
            {
                scene_items = self.scene.read().unwrap().items.len();
                light_items = self.scene.read().unwrap().lights.len();
            }

            let is_loading_scene;
            { is_loading_scene = *(self.loading_scene.lock().unwrap()) != SceneLoadType::Complete; }

            let settings_updates_allowed = !(running && !is_done) && !is_loading_scene;

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

                let mut reload_scene = false;
                ui.horizontal(|ui|
                {
                    egui::ComboBox::from_label("").width(200.0).show_index
                    (
                        ui,
                        &mut selected_scene_id_new,
                        items.len(),
                        |i| items[i].to_owned()
                    );

                    reload_scene = ui.button("⟳").clicked();

                    if is_loading_scene {
                        ui.spinner();
                    }
                });
                if selected_scene_id_new != self.selected_scene_id || reload_scene
                {
                    self.selected_scene_id = selected_scene_id_new;

                    self.rendering_scenes_list.clear();

                    if self.selected_scene_id > 0
                    {
                        let new_scene = SCENE_PATH.to_string() + self.dir_scenes_list[self.selected_scene_id - 1].clone().as_str();
                        self.rendering_scenes_list.push(new_scene);
                    }

                    self.init_scene();
                }

                if ui.button("add ground plane").clicked()
                {
                    self.scene.write().unwrap().add_ground_plane();
                }

                if ui.button("add environment sphere").clicked()
                {
                    self.scene.write().unwrap().add_environment_sphere();
                }

                ui.separator();

                if scene_items > 0
                {
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

                    ui.add(egui::Slider::new(&mut focal_length_new, 1.0..=128.0).suffix(" unit").text("focal length"));
                    ui.add(egui::Slider::new(&mut aperture_size_new, 1.0..=128.0).suffix(" px").text("aperture size"));

                    ui.add(egui::Slider::new(&mut fog_density_new, 0.0..=1.0).text("fog density (distance based amount)"));

                    ui.horizontal(|ui|
                    {
                        ui.label("fog color:");
                        ui.color_edit_button_srgba(&mut fog_color_new);
                    });

                    ui.add(egui::Slider::new(&mut max_recursion_new, 1..=64).text("max recursion"));
                    ui.checkbox(&mut gamma_correction_new, "gamma correction");

                    ui.separator();

                    {
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
                    }

                    // ********** Post Processing **********
                    ui.heading("Post Processing");

                    let mut cavaty;
                    let mut outline;
                    {
                        let rt = self.raytracing.read().unwrap();
                        cavaty = rt.post_processing.cavity;
                        outline = rt.post_processing.outline;
                    }

                    {
                        let mut apply_settings = false;

                        apply_settings = ui.checkbox(&mut cavaty, "Cavity").changed() || apply_settings;
                        apply_settings = ui.checkbox(&mut outline, "Outline").changed() || apply_settings;

                        if apply_settings
                        {
                            let mut rt = self.raytracing.write().unwrap();
                            rt.post_processing.cavity = cavaty;
                            rt.post_processing.outline = outline;
                        }
                    }

                    ui.separator();

                    // ********** scene and light settings **********
                    let mut height = 10.0;
                    if scene_items > 0 || light_items > 0
                    {
                        height = 200.0;
                    }

                    let scroll_area = ScrollArea::vertical().max_height(height).auto_shrink([false; 2]);

                    scroll_area.show(ui, |ui|
                    {
                        // ********** camera **********
                        ui.heading("Camera");
                        ui.vertical(|ui|
                        {
                            ui.collapsing("Camera settings", |ui|
                            {
                                ui.horizontal_wrapped(|ui|
                                {
                                    let mut fov;

                                    let mut eye_pos;
                                    let mut up;
                                    let mut dir;

                                    let mut clipping_near;
                                    let mut clipping_far;

                                    {
                                        let scene = self.scene.read().unwrap();
                                        let cam = &scene.cam;

                                        fov = cam.fov.to_degrees();

                                        eye_pos = cam.eye_pos;
                                        up = cam.up;
                                        dir = cam.dir;

                                        clipping_near = cam.clipping_near;
                                        clipping_far = cam.clipping_far;
                                    }

                                    let mut apply_settings = false;

                                    ui.vertical(|ui|
                                    {
                                        apply_settings = ui.add(egui::Slider::new(&mut fov, 0.001..=360.0).suffix(" °").text("field of view (fov)")).changed() || apply_settings;

                                        ui.horizontal(|ui|
                                        {
                                            ui.label("eye pos:");
                                            apply_settings = ui.add(egui::DragValue::new(&mut eye_pos.x).speed(0.1).prefix("x: ")).changed() || apply_settings;
                                            apply_settings = ui.add(egui::DragValue::new(&mut eye_pos.y).speed(0.1).prefix("y: ")).changed() || apply_settings;
                                            apply_settings = ui.add(egui::DragValue::new(&mut eye_pos.z).speed(0.1).prefix("z: ")).changed() || apply_settings;
                                        });

                                        ui.horizontal(|ui|
                                        {
                                            ui.label("up vector:");
                                            apply_settings = ui.add(egui::DragValue::new(&mut up.x).speed(0.1).prefix("x: ")).changed() || apply_settings;
                                            apply_settings = ui.add(egui::DragValue::new(&mut up.y).speed(0.1).prefix("y: ")).changed() || apply_settings;
                                            apply_settings = ui.add(egui::DragValue::new(&mut up.z).speed(0.1).prefix("z: ")).changed() || apply_settings;
                                        });

                                        ui.horizontal(|ui|
                                        {
                                            ui.label("dir vector:");
                                            apply_settings = ui.add(egui::DragValue::new(&mut dir.x).speed(0.1).prefix("x: ")).changed() || apply_settings;
                                            apply_settings = ui.add(egui::DragValue::new(&mut dir.y).speed(0.1).prefix("y: ")).changed() || apply_settings;
                                            apply_settings = ui.add(egui::DragValue::new(&mut dir.z).speed(0.1).prefix("z: ")).changed() || apply_settings;
                                        });

                                        apply_settings = ui.add(egui::Slider::new(&mut clipping_near, 0.0..=10.0).text("near clipping plane")).changed() || apply_settings;
                                        apply_settings = ui.add(egui::Slider::new(&mut clipping_far, 1.0..=100000.0).text("far clipping plane")).changed() || apply_settings;
                                    });

                                    if apply_settings
                                    {
                                        let mut scene = self.scene.write().unwrap();
                                        let cam = & mut scene.cam;

                                        cam.fov = fov.to_radians();

                                        cam.eye_pos = eye_pos;
                                        cam.up = up;
                                        cam.dir = dir;

                                        cam.clipping_near = clipping_near;
                                        cam.clipping_far = clipping_far;
                                    }
                                });
                            });
                            ui.end_row();
                        });

                        // ********** light **********
                        ui.horizontal(|ui|
                        {
                            ui.heading("Lights");

                            if ui.button("+").clicked()
                            {
                                let mut scene = self.scene.write().unwrap();
                                scene.add_default_light();
                            }
                        });

                        ui.vertical(|ui|
                        {
                            let mut light_items = vec![];
                            {
                                let scene = self.scene.read().unwrap();
                                for item in & scene.lights
                                {
                                    light_items.push((item.id, item.name().clone()));
                                }
                            }

                            for item in & light_items
                            {
                                ui.collapsing(format!("{}: {}", item.0, item.1), |ui|
                                {
                                    ui.horizontal_wrapped(|ui|
                                    {
                                        let mut enabled;

                                        let mut pos;
                                        let mut dir;
                                        let mut color;
                                        let mut intensity;
                                        let mut max_angle;
                                        let mut light_type;

                                        {
                                            let scene = self.scene.read().unwrap();
                                            let item = scene.get_light_by_id(item.0).unwrap();

                                            enabled = item.enabled;

                                            pos = item.pos;
                                            dir = item.dir;

                                            let r = (item.color.x * 255.0) as u8;
                                            let g = (item.color.y * 255.0) as u8;
                                            let b = (item.color.z * 255.0) as u8;
                                            color = Color32::from_rgb(r, g, b);

                                            intensity = item.intensity;
                                            max_angle = item.max_angle;
                                            light_type = item.light_type;
                                        }

                                        let mut apply_settings = false;

                                        ui.vertical(|ui|
                                        {
                                            apply_settings = ui.checkbox(&mut enabled, "Enabled").changed() || apply_settings;

                                            ui.horizontal(|ui|
                                            {
                                                ui.label("pos:");
                                                apply_settings = ui.add(egui::DragValue::new(&mut pos.x).speed(0.1).prefix("x: ")).changed() || apply_settings;
                                                apply_settings = ui.add(egui::DragValue::new(&mut pos.y).speed(0.1).prefix("y: ")).changed() || apply_settings;
                                                apply_settings = ui.add(egui::DragValue::new(&mut pos.z).speed(0.1).prefix("z: ")).changed() || apply_settings;
                                            });

                                            ui.horizontal(|ui|
                                            {
                                                ui.label("dir:");
                                                apply_settings = ui.add(egui::DragValue::new(&mut dir.x).speed(0.1).prefix("x: ")).changed() || apply_settings;
                                                apply_settings = ui.add(egui::DragValue::new(&mut dir.y).speed(0.1).prefix("y: ")).changed() || apply_settings;
                                                apply_settings = ui.add(egui::DragValue::new(&mut dir.z).speed(0.1).prefix("z: ")).changed() || apply_settings;
                                            });

                                            ui.horizontal(|ui|
                                            {
                                                ui.label("color:");
                                                apply_settings = ui.color_edit_button_srgba(&mut color).changed() || apply_settings;
                                            });

                                            apply_settings = ui.add(egui::Slider::new(&mut intensity, 0.0..=10000.0).text("intensity")).changed() || apply_settings;
                                            apply_settings = ui.add(egui::Slider::new(&mut max_angle, 0.0..=PI).text("max_angle")).changed() || apply_settings;

                                            ui.horizontal(|ui|
                                            {
                                                apply_settings = ui.selectable_value(& mut light_type, LightType::Directional, "Directional").changed() || apply_settings;
                                                apply_settings = ui.selectable_value(& mut light_type, LightType::Point, "Point").changed() || apply_settings;
                                                apply_settings = ui.selectable_value(& mut light_type, LightType::Spot, "Spot").changed() || apply_settings;
                                            });
                                        });

                                        if apply_settings
                                        {
                                            let mut scene = self.scene.write().unwrap();
                                            let item = scene.get_light_by_id_mut(item.0).unwrap();

                                            item.enabled = enabled;

                                            item.pos = pos;
                                            item.dir = dir;

                                            let r = ((color.r() as f32) / 255.0).clamp(0.0, 1.0);
                                            let g = ((color.g() as f32) / 255.0).clamp(0.0, 1.0);
                                            let b = ((color.b() as f32) / 255.0).clamp(0.0, 1.0);
                                            item.color = Vector3::<f32>::new(r, g, b);

                                            item.intensity = intensity;
                                            item.max_angle = max_angle;
                                            item.light_type = light_type;
                                        }
                                    });

                                    if ui.button(RichText::new("delete").color(ui.visuals().error_fg_color)).clicked()
                                    {
                                        let mut scene = self.scene.write().unwrap();
                                        scene.delete_light_by_id(item.0);
                                    }
                                });
                                ui.end_row();
                            }
                        });

                        // ********** materials **********
                        ui.heading("Materials");

                        let mut material_items = vec![];
                        {
                            let scene = self.scene.read().unwrap();
                            for material in & scene.materials
                            {
                                let mat = material.read().unwrap();
                                material_items.push((mat.id, mat.name.clone()));
                            }
                        }

                        ui.vertical(|ui|
                        {
                            for item in & material_items
                            {
                                ui.collapsing(format!("{}: {}", item.0, item.1), |ui|
                                {
                                    ui.horizontal_wrapped(|ui|
                                    {
                                        self.show_material_setting(ui, item.0);
                                    });
                                });
                            }
                        });

                        // ********** scene items **********
                        ui.heading("Scene Items");
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
                                    // ********** basic settings **********
                                    ui.horizontal_wrapped(|ui|
                                    {
                                        // basic settings
                                        let mut visible;
                                        let mut flip_normals;

                                        {
                                            let scene = self.scene.read().unwrap();
                                            let item = scene.get_obj_by_id(item.0).unwrap();

                                            visible = item.get_basic().visible;
                                            flip_normals = item.get_basic().flip_normals;
                                        }

                                        let mut apply_settings = false;

                                        ui.vertical(|ui|
                                        {
                                            ui.separator();
                                            apply_settings = ui.checkbox(&mut visible, "Visible").changed() || apply_settings;
                                            apply_settings = ui.checkbox(&mut flip_normals, "flip normals").changed() || apply_settings;
                                        });

                                        if apply_settings
                                        {
                                            let mut scene = self.scene.write().unwrap();
                                            let item = scene.get_obj_by_id_mut(item.0).unwrap();

                                            item.get_basic_mut().visible = visible;
                                            item.get_basic_mut().flip_normals = flip_normals;
                                        }
                                    });

                                    // ********** material and textures **********
                                    let material_id;
                                    let material_name;
                                    {
                                        let scene = self.scene.read().unwrap();
                                        let item = scene.get_obj_by_id(item.0).unwrap();
                                        material_id = item.get_material().read().unwrap().id;
                                        material_name = item.get_material().read().unwrap().name.clone();
                                    }
                                    ui.collapsing(format!("Material ({})", material_name), |ui|
                                    {
                                        self.show_material_setting(ui, material_id);
                                    });

                                    if ui.button(RichText::new("delete").color(ui.visuals().error_fg_color)).clicked()
                                    {
                                        let mut scene = self.scene.write().unwrap();
                                        scene.delete_object_by_id(item.0);
                                    }
                                });
                                ui.end_row();
                            }
                        });
                    })
                    .inner;

                    ui.separator();
                }
            });

            // ********** start rendering **********
            if scene_items > 0
            {
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
                    ui.horizontal(|ui|
                    {
                        if ui.button("Start Rendering").clicked()
                        {
                            self.restart_rendering();
                            self.stopped = false;
                        }

                        if ui.button("Start Post Processing").clicked()
                        {
                            self.post_processing();
                        }
                    });
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
        let clicked  = ctx.input(|i| i.pointer.any_click());
        if clicked
        {
            let pos = ctx.input(|i| i.pointer.interact_pos());
            let x = pos.unwrap().x as i32;
            let y = pos.unwrap().y as i32;

            let rt = self.raytracing.read().unwrap();
            let pick_res = rt.pick(x, y);

            if let Some(pick_res) = pick_res
            {
                let w = self.image.width() as usize;
                let x = x as usize;
                let y = y as usize;

                let pixel = self.image.get_pixel(x as u32, y as u32);
                let normal = self.normals[y * w + x];
                let depth = self.depth[y * w + x];
                let object_id = self.objects[y * w + x];

                println!("==== pixel info ===");
                println!("pick_res: id={}, name={}, depth={}", pick_res.0, pick_res.1, pick_res.2);
                println!("color: {:?}", pixel);
                println!("normal: {:?}",normal);
                println!("depth: {}",depth);
                println!("object_id: {}",object_id);
                println!();
            }
        }

        // hide ui with H-key
        if ctx.input(|i| i.key_pressed(egui::Key::H))
        {
            self.ui_visible = !self.ui_visible;
        }

        // start rendering with CTRL/CMD + R
        let rendering_shortcut_ctrl = egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::R);
        let rendering_shortcut_cmd = egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::R);
        if ctx.input_mut(|i| i.consume_shortcut(&rendering_shortcut_cmd) || i.consume_shortcut(&rendering_shortcut_ctrl))
        {
            if !self.rendering.is_running() || self.rendering.is_done()
            {
                self.restart_rendering();
                self.stopped = false;
            }
        }

        // clear image
        let clear_shortcut_ctrl = egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::D);
        let clear_shortcut_cmd = egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::D);
        if ctx.input_mut(|i| i.consume_shortcut(&clear_shortcut_ctrl) || i.consume_shortcut(&clear_shortcut_cmd))
        {
            if !self.rendering.is_running() || self.rendering.is_done()
            {
                self.init_image();
            }
        }

        // fullscreen
        if ctx.input(|i| i.key_pressed(egui::Key::F))
        {
            frame.set_fullscreen(!frame.info().window_info.fullscreen);
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