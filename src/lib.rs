use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;
#[macro_use]
mod util;
mod ytimg;

pub async fn get_images() -> Vec<ytimg::Image> {
    let document = window().unwrap().document().unwrap().document_element().unwrap();
    let mut html = js_sys::Reflect::get(&document, &"innerHTML".into()).unwrap().as_string().unwrap();
    let idx = html.find("https://i.ytimg.com/sb/").unwrap();
    html.replace_range(..idx, "");
    let idx = html.find("\"}}").unwrap();
    html.truncate(idx);
    log!("value: {}", html);
    let endpoints = ytimg::parse_value(html).unwrap();

    log!("Status confirmed: {:?}", endpoints);

    let endpoint = &endpoints[2];

    let mut images = Vec::new();
    let mut n = 0;
    while let Ok(mut new_images) = endpoint.get_image(n).await {
        util::sleep(std::time::Duration::from_millis(200)).await;
        images.append(&mut new_images);
        n+=1;
    }

    let mut rv: usize = 0;
    let mut gv: usize = 0;
    let mut bv: usize = 0;

    let mut t = 0;
    for image in &images {
        #[cfg(feature="visualization")]
        if t == 6*60 {
            use image::{GenericImage, GenericImageView, ImageBuffer, RgbImage};
            let mut img: RgbImage = ImageBuffer::new(160, 90);

            for x in 0..160 {
                for y in 0..90 {
                    let (r, g, b) = image.get_pixel(x, y);
                    img.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
                }
            }
            img.put_pixel(75, 16, image::Rgb([255,0,0]));
            let mut output = Vec::new();
            let encoder = image::codecs::png::PngEncoder::new(&mut output);
            encoder.encode(&img, 160, 90, image::ColorType::Rgb8).unwrap();
            log!("{}", base64::encode(output));
        }

        if image.does_pixel_match(16, 75, 0xbbd2e6, 40) {
            log!("concil!");
        } else {
            log!("game");
        }

        if t>=5*60+35 && t <=8*60+20 {
            image.get_pixel(16, 75);
        }

        t += 5;
    }

    log!("{} {} {}", rv/33, gv/33, bv/33);

    images
}

#[wasm_bindgen(start)]
pub async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    log!("Hello World!");

    let images = get_images().await;
    log!("{}", images.len());
}
