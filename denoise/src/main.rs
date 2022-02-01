use image::GenericImageView;

extern crate image;
extern crate oidn;

fn main()
{
    let img = image::open("data/test.png").unwrap();

    let mut input_img = vec![0.0f32; (3 * img.width() * img.height()) as usize];
    for y in 0..img.height()
    {
        for x in 0..img.width()
        {
            let p = img.get_pixel(x, y);
            for c in 0..3
            {
                input_img[3 * ((y * img.width() + x) as usize) + c] = p[c] as f32 / 255.0;
            }
        }
    }

    println!("Image dims {}x{}", img.width(), img.height());

    let mut filter_output = vec![0.0f32; input_img.len()];

    let device = oidn::Device::new();
    let mut filter = oidn::RayTracing::new(&device);
    filter.srgb(true).image_dimensions(img.width() as usize, img.height() as usize);
    filter.filter(&input_img[..], &mut filter_output[..]).expect("Invalid input image dimensions?");

    if let Err(e) = device.get_error()
    {
        println!("Error denosing image: {}", e.1);
    }

    let mut output_img = vec![0u8; filter_output.len()];
    for i in 0..filter_output.len()
    {
        let p = filter_output[i] * 255.0;
        if p < 0.0
        {
            output_img[i] = 0;
        }
        else if p > 255.0
        {
            output_img[i] = 255;
        }
        else
        {
            output_img[i] = p as u8;
        }
    }

    image::save_buffer("data/out.png", &output_img[..], img.width(), img.height(), image::ColorType::Rgb8).unwrap();
}