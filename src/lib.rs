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

    let selection = [109,110,111,112,113,114,130,131,132,133,134,135,136,137,138,154,155,171,185,186,187,188,189,190,213,214,215,232,233,234,235,236,237,238]; // https://www.youtube.com/watch?v=BTQcPQ03n3I&ab_channel=DomingoReplay
    let mut r: u64 = 0;
    let mut g: u64 = 0;
    let mut b: u64 = 0;
    for (idx, image) in images.iter().enumerate() {
        if selection.contains(&idx) {
            let (r2, g2, b2) = image.get_pixels_mean(64..71, 3..6);
            r += r2 as u64;
            g += g2 as u64;
            b += b2 as u64;
        }
    }
    r /= selection.len() as u64;
    g /= selection.len() as u64;
    b /= selection.len() as u64;
    log!("mean = {} {} {}", r, g, b);

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
                            tr {
                                td {"game"}
                                td boolean_value=(image.is_game())
                                    title="See below"
                                    {(image.is_game())} }
                            tr {
                                td {"council"}
                                td boolean_value=(image.council)
                                    title=(format!("Mean of 28..74,16..17 = {:?}", image.get_pixels_mean(28..74, 16..17)))
                                    {(image.council)} }
                            tr {
                                td {"bright map"}
                                td
                                    boolean_value=(image.bright_map)
                                    title=(format!("Mean of 152..156,14..15 = {:?}\nMean of 153..155,17..19 = {:?}", image.get_pixels_mean(152..156, 14..15), image.get_pixels_mean(153..155, 17..19)))
                                    {(image.bright_map)}
                            }
                            tr {
                                td {"open map"}
                                td
                                    boolean_value=(image.open_map)
                                    title=(format!("Mean of 24..29,9..14 = {:?}\nMean of 10..15,9..15 = {:?}", image.get_pixels_mean(24..29, 9..14), image.get_pixels_mean(10..15, 9..15)))
                                    {(image.open_map)}
                            }
                            tr {
                                td {"impostor objective"}
                                td
                                    boolean_value=(image.impostor_objective)
                                    title=(format!("Mean of 1..39,12..13 = {:?}", image.get_pixels_mean(1..39, 12..13)))
                                    {(image.impostor_objective)}
                            }
                            tr {
                                td {"game settings"}
                                td
                                    boolean_value=(image.game_settings)
                                    title=(format!("Mean of 1..17,3..68 = {:?}", image.get_pixels_mean(1..17, 3..68)))
                                    {(image.game_settings)}
                            }
                            tr {
                                td {"victory screen"}
                                td
                                    boolean_value=(image.victory_screen)
                                    title=(format!("Mean of 49..111,12..21 = {:?}\nMean of 40..120,25..41 = {:?}", image.get_pixels_mean(49..111, 12..21), image.get_pixels_mean(40..120, 25..41)))
                                    {(image.victory_screen)}
                            }
                            tr {
                                td {"alert"}
                                td
                                    boolean_value=(image.alert)
                                    title=(format!("Mean of 0..160,0..30 = {:?}\nMean of 0..160,30..60 = {:?}\nMean of 0..160,60..90 = {:?}", image.get_pixels_mean(0..160, 0..30), image.get_pixels_mean(0..160, 30..60), image.get_pixels_mean(0..160, 60..90)))
                                    {(image.alert)}
                            }
                            tr {
                                td {"progress bar"}
                                td
                                    boolean_value=(image.progress_bar)
                                    title=(format!("Mean of 2..12,3..6 = {:?}\nMean of 64..71,3..6 = {:?}", image.get_pixels_mean(2..12, 3..6), image.get_pixels_mean(64..71, 3..6)))
                                    {(image.progress_bar)}
                            }
                        }
                    }
                }
            }
        }
    };

    let window = window().unwrap().open_with_url("about:blank").unwrap().unwrap();
    window.document().unwrap().dyn_into::<HtmlDocument>().unwrap().write_1(&html.into_string()).unwrap();

    images
}

#[wasm_bindgen(start)]
pub async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    log!("Hello World!");

    let images = get_images().await;
    log!("{}", images.len());
}
