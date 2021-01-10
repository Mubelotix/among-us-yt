use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;
#[macro_use]
mod util;
mod ytimg;

pub async fn get_images() -> Vec<ytimg::Image> {
    let document = window()
        .unwrap()
        .document()
        .unwrap()
        .document_element()
        .unwrap();
    let mut html = js_sys::Reflect::get(&document, &"innerHTML".into())
        .unwrap()
        .as_string()
        .unwrap();
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
        util::sleep(std::time::Duration::from_millis(100)).await;
        images.append(&mut new_images);
        n += 1;
    }

    use maud::Render;

    let html = maud::html! {
        head {
            title { "Video Analys Report" }
            style {
                (maud::PreEscaped(r#"
                main {
                    display: flex;
                    flex-direction: row;
                }

                html, body {
                    padding: 0;
                    margin: 0;
                }

                main>div {
                    min-width: 160px;
                    margin: 5px;
                }

                main>div:hover {
                    min-width: 480px;
                }

                .preview_image {
                    width: 100%;
                    margin-bottom: 1rem;
                }
                
                table, th, td {
                    border: 1px solid black;
                    border-collapse: collapse;
                }

                td {
                    padding: 5px;
                }

                td[boolean_value="true"] {
                    background-color: #bfb;
                }

                td[boolean_value="false"] {
                    background-color: #fbb;
                }
                "#))
            }
        }
        body {
            main {
                @for (idx, image) in images.iter().enumerate() {
                    div {
                        img.preview_image src=(format!("data:image/png;base64, {}", image.base64())) {}
                        table {
                            tr { td {"index"} td {(idx)} }
                            tr { td {"council"} td boolean_value=(image.is_council()) {(image.is_council())} }
                        }
                    }
                }
            }
        }
    };

    log!("{} ", html.into_string());

    images
}

#[wasm_bindgen(start)]
pub async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    log!("Hello World!");

    let images = get_images().await;
    log!("{}", images.len());
}
