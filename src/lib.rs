use std::{cell::Cell, ops::AddAssign, rc::Rc};

use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::*;
#[macro_use]
mod util;
mod ytimg;

pub async fn get_images(loaded: bool) -> (Vec<(std::ops::Range<usize>, bool)>, usize) {
    let endpoints = if loaded {
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
        ytimg::parse_value(html).unwrap()
    } else {
        let mut url = window().unwrap().location().href().unwrap();
        let start = url.find("watch?v=").unwrap();
        url.replace_range(..start + 8, "");
        let end = url.find('&').unwrap_or_else(|| url.len());
        url.truncate(end);
        let id = url;

        let open_db_request = window()
            .unwrap()
            .indexed_db()
            .unwrap()
            .unwrap()
            .open("swpushnotificationsdb")
            .unwrap();
        let open_db_request2 = open_db_request.clone();
        let state = Rc::new(Cell::new(None));
        let state2 = Rc::clone(&state);
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let database: IdbDatabase = open_db_request2.result().unwrap().dyn_into().unwrap();
            state2.set(Some(database));
        }) as Box<dyn FnMut(_)>);
        open_db_request
            .add_event_listener_with_callback(
                "success", // TODO HANDLE ERROR
                closure.as_ref().unchecked_ref(),
            )
            .unwrap();

        let db = loop {
            match state.take() {
                Some(db) => break db,
                None => util::sleep(std::time::Duration::from_millis(200)).await,
            }
        };
        let transaction = db.transaction_with_str("swpushnotificationsstore").unwrap();
        let store = transaction
            .object_store("swpushnotificationsstore")
            .unwrap();
        let value_request = store.get(&"IDToken".into()).unwrap();
        let value_request2 = value_request.clone();
        let state = Rc::new(Cell::new(None));
        let state2 = Rc::clone(&state);
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let value = js_sys::Reflect::get(&value_request2.result().unwrap(), &"value".into())
                .unwrap()
                .as_string()
                .unwrap();
            state2.set(Some(value));
        }) as Box<dyn FnMut(_)>);
        value_request
            .add_event_listener_with_callback(
                "success", // TODO HANDLE ERROR
                closure.as_ref().unchecked_ref(),
            )
            .unwrap();

        let user_id = loop {
            match state.take() {
                Some(user_id) => break user_id,
                None => util::sleep(std::time::Duration::from_millis(200)).await,
            }
        };

        let mut request: RequestInit = RequestInit::new();
        request.method("GET");

        let headers = Headers::new().unwrap();
        headers
            .append("X-SPF-Previous", "https://www.youtube.com/")
            .unwrap();
        headers
            .append("X-SPF-Referer", "https://www.youtube.com/")
            .unwrap();
        headers.append("X-YouTube-Client-Name", "1").unwrap();
        headers
            .append("X-YouTube-Client-Version", "2.20210110.08.00")
            .unwrap();
        headers.append("x-youtube-csoc", "1").unwrap();
        headers
            .append(
                "X-YouTube-Device",
                "cbr=Firefox&cbrver=84.0&ceng=Gecko&cengver=84.0&cos=X11&cplatform=DESKTOP",
            )
            .unwrap();
        headers
            .append("X-Youtube-Identity-Token", &user_id)
            .unwrap();
        headers
            .append("X-YouTube-Time-Zone", "Europe/Paris")
            .unwrap();
        headers.append("X-YouTube-Utc-Offset", "60").unwrap();
        request.headers(&headers);

        let window = window().unwrap();
        let response = Response::from(
            JsFuture::from(window.fetch_with_str_and_init(
                &format!("https://www.youtube.com/watch?v={}&pbj=1", id),
                &request,
            ))
            .await
            .unwrap(),
        );

        if response.status() != 200 {
            panic!("Unexpected response status");
        }

        let mut object = JsFuture::from(response.json().unwrap()).await.unwrap();
        object = js_sys::Reflect::get(&object, &2.into()).unwrap();
        object = js_sys::Reflect::get(&object, &"playerResponse".into()).unwrap();
        object = js_sys::Reflect::get(&object, &"storyboards".into()).unwrap();
        object = js_sys::Reflect::get(&object, &"playerStoryboardSpecRenderer".into()).unwrap();
        object = js_sys::Reflect::get(&object, &"spec".into()).unwrap();
        let value = object.as_string().unwrap();

        ytimg::parse_value(value).unwrap()
    };

    log!("Status confirmed: {:?}", endpoints);

    let endpoint = &endpoints[2];

    let mut images = Vec::new();
    let mut n = 0;
    while let Ok(mut new_images) = endpoint.get_image(n).await {
        util::sleep(std::time::Duration::from_millis(100)).await;
        images.append(&mut new_images);
        n += 1;
    }
    'images: for i in (0..images.len()).into_iter().rev() {
        for x in 0..160 {
            for y in 0..90 {
                if images[i].get_pixel(x, y) != (0, 0, 0) {
                    break 'images;
                }
            }
        }
        images.remove(i);
        log!("removed an image");
    }

    let selection = [
        112, 113, 114, 115, 116, 117, 247, 248, 249, 250, 335, 336, 337,
    ]; // https://www.youtube.com/watch?v=kofC4k2tm68&ab_channel=DomingoReplay
    let mut r: u64 = 0;
    let mut g: u64 = 0;
    let mut b: u64 = 0;
    for (idx, image) in images.iter().enumerate() {
        if selection.contains(&idx) {
            let (r2, g2, b2) = image.get_pixels_mean(40..120, 25..41);
            r += r2 as u64;
            g += g2 as u64;
            b += b2 as u64;
        }
    }
    r /= selection.len() as u64;
    g /= selection.len() as u64;
    b /= selection.len() as u64;
    log!("mean = {} {} {}", r, g, b);

    let mut games = Vec::new();
    let mut current_game: Option<(usize, usize, usize)> = None;
    for (idx, image) in images.iter().enumerate() {
        if image.is_game() && current_game.is_none() {
            current_game = Some((idx, 0, 1));
        } else if image.victory_screen || image.game_settings || image.defeat_screen {
            if let Some((start, impostor_objectives_count, ingame_frames_count)) = current_game {
                let ratio = impostor_objectives_count as f64 / ingame_frames_count as f64;
                games.push((start..idx, ratio > 0.6));
                current_game = None;
            }
        }

        if let Some((_, impostor_objectives_count, _)) = &mut current_game {
            if image.impostor_objective && !image.alert {
                impostor_objectives_count.add_assign(1);
            }
        }

        if let Some((_, _, ingame_frames_count)) = &mut current_game {
            if image.is_game() && !image.open_map && !image.council && !image.alert {
                ingame_frames_count.add_assign(1);
            }
        }
    }
    if let Some((start, impostor_objectives_count, ingame_frames_count)) = current_game {
        let ratio = impostor_objectives_count as f64 / ingame_frames_count as f64;
        games.push((start..images.len(), ratio > 0.6));
    }

    #[cfg(feature = "debugging")]
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

                #games {
                    position: fixed;
                    left: 5px;
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

    #[cfg(feature = "debugging")]
    window()
        .unwrap()
        .open_with_url("about:blank")
        .unwrap()
        .unwrap()
        .document()
        .unwrap()
        .dyn_into::<HtmlDocument>()
        .unwrap()
        .write_1(&html.into_string())
        .unwrap();

    (games, images.len())
}

pub async fn run(loaded: bool) {
    log!("running...");
    let (games, lenght) = get_images(loaded).await;
    let window = window().unwrap();
    let container = loop {
        match window
            .document()
            .unwrap()
            .get_elements_by_class_name("ytp-progress-bar-padding")
            .item(0)
        {
            Some(container) => break container,
            None => util::sleep(std::time::Duration::from_millis(200)).await,
        }
    };
    let factor: f64 = 100.0 / lenght as f64;

    let html = maud::html! {
        style { (include_str!("integrated.css")) }
        #among_us_addon_chapters {
            @for (game, is_impostor) in games.iter() {
                @if *is_impostor {
                    div.impostor_game style=(format!("left: {}%; width: calc({}% - 4px);", game.start as f64 * factor, (game.end - game.start) as f64 * factor)) {
                        "Impostor"
                    }
                } @else {
                    div.crewmate_game style=(format!("left: {}%; width: calc({}% - 4px);", game.start as f64 * factor, (game.end - game.start) as f64 * factor)) {
                        "Crewmate"
                    }
                }

            }
        }
    };
    container.set_inner_html(&html.into_string());
}

#[wasm_bindgen(start)]
pub async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    log!("Hello World!");

    let mut launched = false;
    let window2 = window().unwrap();
    let mut last_url = window2.location().href().unwrap();
    if last_url.starts_with("https://www.youtube.com/watch?v=") {
        wasm_bindgen_futures::spawn_local(async move {
            if !launched {
                launched = true;
                run(true).await;
            }
        });
    }
    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        if window2.location().href().unwrap() != last_url {
            last_url = window2.location().href().unwrap();
            log!("url changed! to {}", last_url);
            if last_url.starts_with("https://www.youtube.com/watch?v=")
                && last_url.contains("ab_channel")
            {
                wasm_bindgen_futures::spawn_local(async move {
                    if !launched {
                        launched = true;
                        run(false).await;
                    }
                });
            }
        }
    }) as Box<dyn FnMut(_)>);
    window()
        .unwrap()
        .set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            1000,
        )
        .unwrap();
    closure.forget();
}
