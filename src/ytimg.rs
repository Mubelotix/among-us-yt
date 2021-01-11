#[derive(Debug)]
pub struct Endpoint {
    start: String,
    sqp: String,
    number: u8,
    sigh: String,
    width: u8,
    height: u8,
    image_width: u8,
    image_height: u8,
}

impl Endpoint {
    pub async fn get_image(&self, n: usize) -> Result<Vec<Image>, &'static str> {
        use js_sys::Promise;
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::*;

        let mut request: RequestInit = RequestInit::new();
        request.method("GET");

        let headers = Headers::new().unwrap();
        request.headers(&headers);

        let window = window().unwrap();
        let response = match JsFuture::from(window.fetch_with_str_and_init(
            &format!(
                "{}{}/M{}.jpg?sqp={}&sigh={}",
                self.start, self.number, n, self.sqp, self.sigh
            ),
            &request,
        ))
        .await
        {
            Ok(response) => Response::from(response),
            Err(e) => {
                elog!("error: {:?}", e);
                return Err("failed request");
            }
        };

        if response.status() != 200 {
            return Err("Unexpected response status");
        }

        let blob: Blob = JsFuture::from(response.blob().unwrap())
            .await
            .unwrap()
            .dyn_into()
            .unwrap();

        let url = Url::create_object_url_with_blob(&blob).unwrap(); // FIXME: revoke object url
        let document = window.document().unwrap();
        let img: HtmlImageElement = document.create_element("img").unwrap().dyn_into().unwrap();
        img.set_src(&url);
        JsFuture::from(Promise::new(&mut |yes, no| {
            img.add_event_listener_with_callback("load", &yes).unwrap();
            img.add_event_listener_with_callback("error", &no).unwrap();
        }))
        .await
        .unwrap();

        let canvas: HtmlCanvasElement = document
            .create_element("canvas")
            .unwrap()
            .dyn_into()
            .unwrap();
        canvas.set_width(self.width as u32 * self.image_width as u32);
        canvas.set_height(self.height as u32 * self.image_height as u32);
        document.body().unwrap().append_child(&canvas).unwrap();
        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        context
            .draw_image_with_html_image_element(&img, 0.0, 0.0)
            .unwrap();

        let mut images = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                let image_data: ImageData = context
                    .get_image_data(
                        x as f64 * self.image_width as f64,
                        y as f64 * self.image_height as f64,
                        self.image_width as f64,
                        self.image_height as f64,
                    )
                    .unwrap();
                let data: Vec<u8> = image_data.data().to_vec();
                images.push(Image::new(data));
            }
        }

        Ok(images)
    }
}

pub fn parse_value(data: String) -> Result<Vec<Endpoint>, &'static str> {
    // https://i.ytimg.com/sb/j370EOXd2RY/storyboard3_L$L/$N.jpg?sqp=-oaymwENSDfyq4qpAwVwAcABAaLzl_8DBgjth6TwBQ==|48#27#100#10#10#0#default#rs$AOn4CLBKGLzTjWWnKLMISMXxpHdX4BiGsQ|80#45#108#10#10#2000#M$M#rs$AOn4CLBA7OWuDEaK8Hah8Tv8jIcbpObXEg|160#90#108#5#5#2000#M$M#rs$AOn4CLC20ooDO3rFTwH0xW6NJwZnsJ8qhQ
    let mut parts: Vec<&str> = data.split('|').collect();
    if parts.is_empty() {
        return Err("empty value");
    }
    let start = parts.remove(0);
    let start_parts: Vec<&str> = start.split("$L/$N.jpg?sqp=").collect();
    if start_parts.len() != 2 {
        return Err("Invalid start value (2 parts expected)");
    }
    let start = start_parts[0];
    let sqp = start_parts[1];

    let mut endpoints = Vec::new();
    for (idx, part) in parts.iter().enumerate() {
        let parts: Vec<&str> = part.split('#').collect();
        endpoints.push(Endpoint {
            start: start.to_string(),
            sqp: sqp.to_string(),
            number: idx as u8,
            image_width: parts
                .get(0)
                .ok_or("Missing image width")?
                .parse()
                .map_err(|_| "image width is not a number")?,
            image_height: parts
                .get(1)
                .ok_or("Missing image height")?
                .parse()
                .map_err(|_| "image height is not a number")?,
            width: parts
                .get(3)
                .ok_or("Missing width")?
                .parse()
                .map_err(|_| "width is not a number")?,
            height: parts
                .get(4)
                .ok_or("Missing height")?
                .parse()
                .map_err(|_| "height is not a number")?,
            sigh: parts.get(7).ok_or("Missing sigh argument")?.to_string(),
        });
    }

    Ok(endpoints)
}

pub struct Image {
    data: Vec<u8>,
    pub council: bool,
    pub bright_map: bool,
    pub impostor_objective: bool,
    pub open_map: bool,
    pub game_settings: bool,
    pub victory_screen: bool,
    pub alert: bool,
    pub progress_bar: bool,
    base64: String,
}

impl Image {
    pub fn new(data: Vec<u8>) -> Self {
        let mut image = Image {
            data,
            council: false,
            bright_map: false,
            impostor_objective: false,
            open_map: false,
            game_settings: false,
            victory_screen: false,
            alert: false,
            progress_bar: false,
            base64: String::new(),
        };

        image.council = image.does_pixels_mean_match(28..74, 16..17, 0xadbfd4, 20);
        image.bright_map = image.does_pixels_mean_match(152..156, 14..15, 0xc8cbcc, 20) && image.does_pixels_mean_match(153..155, 17..19, 0x54595a, 20);
        image.impostor_objective = !image.council && image.does_pixels_mean_match(1..39, 12..13, 0x51252b, 20);
        image.open_map = image.bright_map && (image.does_pixels_mean_match(24..29, 9..14, 0xbdc0c4, 20) || image.does_pixels_mean_match(10..15, 9..15, 0xb9bfbe, 20));
        image.alert = {
            let (r,g,b) = image.get_pixels_mean(0..160, 0..30);
            let diff = r as i32 - (g as i32 + b as i32);
            if r >= 110 && diff > -40 {
                let (r,g,b) = image.get_pixels_mean(0..160, 30..60);
                let diff = r as i32 - (g as i32 + b as i32);
                if r >= 110 && diff > -40 {
                    let (r,g,b) = image.get_pixels_mean(0..160, 60..90);
                    let diff = r as i32 - (g as i32 + b as i32);
                    r >= 110 && diff > -40
                } else {
                    false
                }
            } else {
                false
            }
        };
        image.progress_bar = !image.council && (image.does_pixels_mean_match(2..12, 3..6, 0x72a072, 20) || (image.does_pixels_mean_match(64..71, 3..6, 0x353d38, 20) && image.does_pixels_mean_match(64..71, 2..3, 0x989ca5, 50)));
        image.game_settings = !image.is_game() && image.does_pixels_mean_match(1..17, 3..68, 0x484949, 15);
        image.victory_screen = !image.is_game() && image.does_pixels_mean_match(49..111, 12..21, 0x163150, 10) && image.does_pixels_mean_match(40..120, 25..41, 0x000000, 10);

        use image::{ImageBuffer, RgbImage};
        let mut img: RgbImage = ImageBuffer::new(160, 90);
        for x in 0..160 {
            for y in 0..90 {
                let (r, g, b) = image.get_pixel(x, y);
                img.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
            }
        }
        let mut output = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut output);
        encoder
            .encode(&img, 160, 90, image::ColorType::Rgb8)
            .unwrap();
        image.base64 = base64::encode(output);

        image
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> (u8, u8, u8) {
        (
            self.data[y as usize * 4 * 160 + x as usize * 4],
            self.data[y as usize * 4 * 160 + x as usize * 4 + 1],
            self.data[y as usize * 4 * 160 + x as usize * 4 + 2],
        )
    }

    pub fn get_pixels_mean(&self, x_range: std::ops::Range<u8>, y_range: std::ops::Range<u8>) -> (u8, u8, u8) {
        let mut r: u64 = 0;
        let mut g: u64 = 0;
        let mut b: u64 = 0;

        for x in x_range.clone() {
            for y in y_range.clone() {
                let (r2, g2, b2) = self.get_pixel(x, y);
                r += r2 as u64;
                g += g2 as u64;
                b += b2 as u64;
            }
        }

        let number = (x_range.end - x_range.start) as u64 * (y_range.end - y_range.start) as u64;
        r /= number;
        g /= number;
        b /= number;

        (r as u8, g as u8, b as u8)
    }

    pub fn does_pixels_mean_match(&self, x_range: std::ops::Range<u8>, y_range: std::ops::Range<u8>, expected: u32, tolerance: u8) -> bool {
        let [_, expected_r, expected_g, expected_b] = expected.to_be_bytes();
        let got = self.get_pixels_mean(x_range, y_range);
        std::cmp::max(got.0, expected_r) - std::cmp::min(got.0, expected_r) <= tolerance
            && std::cmp::max(got.1, expected_g) - std::cmp::min(got.1, expected_g) <= tolerance
            && std::cmp::max(got.2, expected_b) - std::cmp::min(got.2, expected_b) <= tolerance
    }

    pub fn does_pixel_match(&self, x: u8, y: u8, expected: u32, tolerance: u8) -> bool {
        let [_, expected_r, expected_g, expected_b] = expected.to_be_bytes();
        let got = self.get_pixel(x, y);
        std::cmp::max(got.0, expected_r) - std::cmp::min(got.0, expected_r) <= tolerance
            && std::cmp::max(got.1, expected_g) - std::cmp::min(got.1, expected_g) <= tolerance
            && std::cmp::max(got.2, expected_b) - std::cmp::min(got.2, expected_b) <= tolerance
    }

    pub fn base64(&self) -> &str {
        &self.base64
    }

    pub fn is_game(&self) -> bool {
        self.bright_map || self.council || self.alert || self.progress_bar
    }
}
