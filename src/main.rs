use egui::{Style, Visuals};
use regex::Regex;
use run::Run;

pub mod helper;
pub mod shape;

pub mod renderer;
pub mod raytracing;
pub mod scene;
pub mod camera;
pub mod animation;
pub mod run;
pub mod post_processing;

fn main()
{
    let args: Vec<String> = std::env::args().collect();

    let mut window = true;
    let mut scenes = vec![];
    let mut animation = true;
    let mut width = 0;
    let mut height = 0;
    let mut monte_carlo = None;
    let mut samples = None;
    let mut start = false;

    let res_regex = Regex::new(r"^\d+x\d+$").unwrap(); // example: 800x600

    for arg in args
    {
        if arg == "cmd"
        {
            window = false;
        }
        else if arg == "no-animation"
        {
            animation = false;
        }
        else if arg.starts_with("monte_carlo=")
        {
            let splits: Vec<&str> = arg.split("=").collect();
            let splits_arr = splits.as_slice();

            monte_carlo = Some(splits_arr[1] == "1" || splits_arr[1] == "true");
        }
        else if arg.ends_with(".json") || arg.ends_with(".gltf") || arg.ends_with(".glb") || arg.ends_with(".obj")
        {
            scenes.push(arg);
        }
        else if res_regex.is_match(arg.as_str())
        {
            let splits: Vec<&str> = arg.split("x").collect();
            let splits_arr = splits.as_slice();

            width = splits_arr[0].parse().unwrap();
            height = splits_arr[1].parse().unwrap();
        }
        else if arg.starts_with("samples=")
        {
            let splits: Vec<&str> = arg.split("=").collect();
            let splits_arr = splits.as_slice();

            samples = Some(splits_arr[1].parse().unwrap());
        }
        else if arg.starts_with("start=")
        {
            let splits: Vec<&str> = arg.split("=").collect();
            let splits_arr = splits.as_slice();

            start = splits_arr[1] == "1" || splits_arr[1] == "true";
        }
    }

    let mut runner = Run::new(width, height, window, scenes, animation, start);

    //apply cmd settings
    {
        let rt = runner.raytracing.write().unwrap();
        if let Some(monte_carlo) = monte_carlo { rt.scene.write().unwrap().raytracing_config.monte_carlo = monte_carlo; }
        if let Some(samples) = samples { rt.scene.write().unwrap().raytracing_config.samples = samples; }
    }

    runner.init();
    runner.start();

    //create window if needed
    if window
    {
        let egui_options = runner.get_egui_options();

        let res = eframe::run_native
        (
            "Rustray",
            egui_options,
            Box::new(|creation_context|
            {
                let style = Style
                {
                    visuals: Visuals::dark(),
                    ..Style::default()
                };
                creation_context.egui_ctx.set_style(style);
                Box::new(runner)
            }),
        );

        if res.is_err()
        {
            println!("error occured");
        }
    }
}