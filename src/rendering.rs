use crate::{util::sleep, ytimg::Image};
use web_sys::*;
use std::ops::Range;
use maud::PreEscaped;
use wasm_bindgen::JsCast;

pub async fn display_bar(lenght: usize, games: Vec<(Range<usize>, bool)>) {
    let window = window().unwrap();
    let document = window
        .document()
        .unwrap();
    let container = loop {
        match document
            .get_elements_by_class_name("ytp-progress-bar-padding")
            .item(0)
        {
            Some(container) => break container,
            None => sleep(std::time::Duration::from_millis(200)).await,
        }
    };
    let factor: f64 = 100.0 / lenght as f64;

    let html = maud::html! {
        style { (PreEscaped(include_str!("integrated.css"))) }
        #among_us_addon_chapters {
            @for (game, is_impostor) in games.iter() {
                @if *is_impostor {
                    div.impostor_game.flex_font style=(format!("left: {}%; width: calc({}% - 4px);", game.start as f64 * factor, (game.end - game.start) as f64 * factor)) {
                        "Impostor"
                    }
                } @else {
                    div.crewmate_game.flex_font style=(format!("left: {}%; width: calc({}% - 4px);", game.start as f64 * factor, (game.end - game.start) as f64 * factor)) {
                        "Crewmate"
                    }
                }

            }
        }
    };
    container.set_inner_html(&html.into_string());
    update_flex_font();
}

pub fn update_flex_font() {
    let divs = window().unwrap().document().unwrap().get_elements_by_class_name("flex_font");
    for div in 0..divs.length() {
        let div = divs.item(div).unwrap();
        let width = div.client_width() - 10;
        let font_size: f64 = width as f64 * 0.19;
        let html_element: HtmlElement = div.dyn_into().unwrap();
        let style = html_element.style();
        if font_size >= 7.0 {
            style.set_property("font-size", &format!("min({}px, 2rem)", font_size)).unwrap();
        } else {
            style.set_property("font-size", "0").unwrap();
        }
    }
}

pub async fn display_loading_state() {
    let window = window().unwrap();
    let container = loop {
        match window
            .document()
            .unwrap()
            .get_elements_by_class_name("ytp-progress-bar-padding")
            .item(0)
        {
            Some(container) => break container,
            None => sleep(std::time::Duration::from_millis(200)).await,
        }
    };

    let html = maud::html! {
        style { (PreEscaped(include_str!("integrated.css"))) }
        #among_us_addon_loading {
            "Among Us Youtube Extension : Loading video..."
        }
    };
    container.set_inner_html(&html.into_string());
}

#[cfg(feature = "debugging")]
pub fn display_debugging_data(images: &[Image], games: &[(Range<usize>, bool)]) {
    let html = maud::html! {
        head {
            title { "Video Analys Report" }
            style { (PreEscaped(include_str!("debugging.css"))) }
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
                                td {"defeat screen"}
                                td
                                    boolean_value=(image.defeat_screen)
                                    title=(format!("Mean of 53..105,9..23 = {:?}\nMean of 40..120,25..41 = {:?}", image.get_pixels_mean(53..105, 9..23), image.get_pixels_mean(40..120, 25..41)))
                                    {(image.defeat_screen)}
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
                                    title=(format!("Mean of 2..12,3..6 = {:?}\nMean of 64..71,3..6 = {:?}\nMean of 64..71,2..3 = {:?}", image.get_pixels_mean(2..12, 3..6), image.get_pixels_mean(64..71, 3..6), image.get_pixels_mean(64..71, 2..3)))
                                    {(image.progress_bar)}
                            }
                        }
                    }
                }
            }
            div#games {
                (format!("{:?}", games))
            }
        }
    };

    window()
        .unwrap()
        .open_with_url("about:blank")
        .unwrap()
        .unwrap()
        .document()
        .unwrap()
        .document_element()
        .unwrap()
        .set_inner_html(&html.into_string());
}
