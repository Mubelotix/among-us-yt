use std::{cell::Cell, ops::AddAssign, rc::Rc};

use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::*;
#[macro_use]
mod util;
mod ytimg;
mod yt_format;
mod rendering;
use yt_format::*;
use rendering::*;

pub async fn get_images(loaded: bool) -> Option<(Vec<(std::ops::Range<usize>, bool)>, usize)> {
    let (yt_initial_player_response, yt_initial_data) = if loaded {
        let document = window()
            .unwrap()
            .document()
            .unwrap()
            .document_element()
            .unwrap();
        let html = js_sys::Reflect::get(&document, &"innerHTML".into())
            .unwrap()
            .as_string()
            .unwrap();

        let idx_start = html.find("var ytInitialPlayerResponse = {").unwrap() + 30;
        let idx_end = html[idx_start..]
            .find(";var meta = document.createElement('meta');")
            .unwrap();
        let yt_initial_player_response = html[idx_start..idx_start + idx_end].to_string();
        let yt_initial_player_response = parse_json(yt_initial_player_response).unwrap();

        let idx_start = html.find("var ytInitialData = {").unwrap() + 20;
        let idx_end = html[idx_start..]
            .find(";</script><script nonce=\"")
            .unwrap();
        let yt_initial_data = html[idx_start..idx_start + idx_end].to_string();
        let yt_initial_data = parse_json(yt_initial_data).unwrap();

        (yt_initial_player_response, yt_initial_data)
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

        let object = JsFuture::from(response.json().unwrap()).await.unwrap();
        let mut yt_initial_player_response = js_sys::Reflect::get(&object, &2.into()).unwrap();
        yt_initial_player_response = js_sys::Reflect::get(&yt_initial_player_response, &"playerResponse".into()).unwrap();

        let mut yt_initial_data: JsValue = js_sys::Reflect::get(&object, &3.into()).ok()?;
        yt_initial_data = js_sys::Reflect::get(&yt_initial_data, &"response".into()).ok()?;

        (yt_initial_player_response, yt_initial_data)
    };

    if get_game_name(&yt_initial_player_response) != Some("Among Us".to_string()) {
        return None;
    }
    let endpoints = ytimg::parse_value(get_storyboard(yt_initial_data).unwrap()).unwrap();

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
    display_debugging_data(&images, &games);

    Some((games, images.len()))
}

pub async fn run(loaded: bool) {
    log!("running...");
    let (games, lenght) = match get_images(loaded).await {
        Some(v) => v,
        None => return,
    };

    display_bar(lenght, games).await;
}

#[wasm_bindgen(start)]
pub async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    log!("Hello World!");

    let window2 = window().unwrap();
    let mut last_url = window2.location().href().unwrap();
    if last_url.starts_with("https://www.youtube.com/watch?v=") {
        wasm_bindgen_futures::spawn_local(async move {
            run(true).await;
        });
    }
    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        if window2.location().href().unwrap() != last_url {
            last_url = window2.location().href().unwrap();
            log!("url changed! to {}", last_url);
            if last_url.starts_with("https://www.youtube.com/watch?v=")
            {
                wasm_bindgen_futures::spawn_local(async move {
                    run(false).await;
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
