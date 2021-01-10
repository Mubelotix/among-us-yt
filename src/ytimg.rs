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
    is_council: bool,
    base64: String,
}

impl Image {
    pub fn new(data: Vec<u8>) -> Self {
        let mut image = Image {
            data,
            is_council: false,
            base64: String::new(),
        };

        image.is_council = image.does_pixel_match(75, 16, 0xbbd2e6, 30);

        use image::{ImageBuffer, RgbImage};
        let mut img: RgbImage = ImageBuffer::new(160, 90);
        for x in 0..160 {
            for y in 0..90 {
                let (r, g, b) = image.get_pixel(x, y);
                img.put_pixel(x as u32, y as u32, image::Rgb([r, g, b]));
            }
        }
        img.put_pixel(75, 16, image::Rgb([255, 0, 0]));
        let mut output = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut output);
        encoder
            .encode(&img, 160, 90, image::ColorType::Rgb8)
            .unwrap();
        image.base64 = base64::encode(output);

        image
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
        (
            self.data[y * 4 * 160 + x * 4],
            self.data[y * 4 * 160 + x * 4 + 1],
            self.data[y * 4 * 160 + x * 4 + 2],
        )
    }

    pub fn does_pixel_match(&self, x: usize, y: usize, expected: u32, tolerance: u8) -> bool {
        let [_, expected_r, expected_g, expected_b] = expected.to_be_bytes();
        let got = self.get_pixel(x, y);
        std::cmp::max(got.0, expected_r) - std::cmp::min(got.0, expected_r) <= tolerance
            && std::cmp::max(got.1, expected_g) - std::cmp::min(got.1, expected_g) <= tolerance
            && std::cmp::max(got.2, expected_b) - std::cmp::min(got.2, expected_b) <= tolerance
    }

    pub fn is_council(&self) -> bool {
        self.is_council
    }

    pub fn base64(&self) -> &str {
        &self.base64
    }
}
