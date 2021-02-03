use crate::{settings::*, util::sleep, ytimg::Image};
use maud::{PreEscaped, Render};
use std::ops::Range;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;

#[derive(Clone, Copy)]
enum Theme {
    Default,
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Default => write!(f, "Default"),
        }
    }
}

impl Choice for Theme {
    fn enumerate_values() -> Vec<String> {
        vec!["Default".to_string()]
    }

    fn select_value(s: &str) -> Self {
        Theme::Default
    }
}

pub async fn display_bar(lenght: usize, games: Vec<(Range<usize>, bool)>) {
    // select the target node
    let target = window()
        .unwrap()
        .document()
        .unwrap()
        .query_selector(".ytp-settings-menu")
        .unwrap()
        .unwrap();

    let closure = Closure::wrap(Box::new(move |events: js_sys::Array, _init| {
        for event in events
            .iter()
            .filter_map(|e| e.dyn_into::<MutationRecord>().ok())
        {
            let target: HtmlElement = event.target().unwrap().dyn_into().unwrap();

            if let Some(name) = event.attribute_name() {
                let new_value = target.get_attribute(&name);
                log!(
                    "{} in {:?}: {}\n{:?}\n{:?}",
                    event.type_().to_uppercase(),
                    target.class_name(),
                    name,
                    match event.old_value() {
                        Some(ref value) => value.as_str(),
                        None => "{None}",
                    },
                    match new_value {
                        Some(ref value) => value.as_str(),
                        None => "{None}",
                    }
                );
            } else {
                log!(
                    "{} in {:?}:\n{}",
                    event.type_().to_uppercase(),
                    target.class_name(),
                    target.inner_html()
                );
            }
        }
    })
        as Box<dyn FnMut(js_sys::Array, MutationObserverInit)>);

    // create an observer instance
    let observer = MutationObserver::new(closure.as_ref().unchecked_ref()).unwrap();
    closure.forget();

    // configuration of the observer:
    let mut config = MutationObserverInit::new();
    config
        .animations(true)
        .attribute_old_value(true)
        .attributes(true)
        .character_data(true)
        .character_data_old_value(true)
        .child_list(true)
        .subtree(true);

    // pass in the target node, as well as the observer options
    observer.observe_with_options(&target, &config).unwrap();

    let window = window().unwrap();
    let document = window.document().unwrap();
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

    if document
        .query_selector("#among_us_settings_menu")
        .unwrap()
        .is_some()
    {
        return;
    }

    // Create the settings menu
    let movie_player = document.get_element_by_id("movie_player").unwrap();
    let among_us_settings_menu = document.create_element("div").unwrap();
    among_us_settings_menu
        .set_attribute("class", "ytp-popup ytp-settings-menu")
        .unwrap();
    among_us_settings_menu
        .set_attribute("id", "among_us_settings_menu")
        .unwrap();
    among_us_settings_menu
        .set_attribute("style", "display: none;")
        .unwrap();
    let generate_comments_setting = CheckBox::new("amgus_ext_comments", "Generate comments", true);
    let theme_setting = Selection::new("amgus_ext_theme", "Theme", Theme::Default);
    let mut settings = Settings::new();
    settings.add_setting(&generate_comments_setting);
    settings.add_setting(&theme_setting);
    among_us_settings_menu.set_inner_html(&settings.render().into_string());
    movie_player.append_child(&among_us_settings_menu).unwrap();
    let settings_rc = std::rc::Rc::new(settings);
    generate_comments_setting.enable(std::rc::Rc::clone(&settings_rc));
    theme_setting.enable(std::rc::Rc::clone(&settings_rc));

    // Create the button in the bottom bar
    let ytp_right_controls = document
        .query_selector(".ytp-right-controls")
        .unwrap()
        .unwrap();
    let among_us_settings_button = document.create_element("button").unwrap();
    among_us_settings_button
        .set_attribute("class", "ytp-button among_us_settings_button")
        .unwrap();
    among_us_settings_button
        .set_attribute("title", "Among Us Settings")
        .unwrap();
    among_us_settings_button.set_inner_html(include_str!("icon.svg"));
    ytp_right_controls
        .insert_before(
            &among_us_settings_button,
            ytp_right_controls.child_nodes().item(4).as_ref(),
        )
        .unwrap();

    // Handle button clicks
    let state = std::rc::Rc::new(std::cell::Cell::new(false));
    let body = document.body().unwrap();
    let closure = Closure::wrap(Box::new(move |event: Event| {
        //let settings_rc = std::rc::Rc::clone(&settings_rc);
        let target = event.target().unwrap().dyn_into().unwrap();
        if among_us_settings_button.contains(Some(&target)) {
            state.set(!state.get());
        } else if among_us_settings_menu.contains(Some(&target)) || !body.contains(Some(&target)) {
            return;
        } else {
            state.set(false);
        }

        let state2 = std::rc::Rc::clone(&state);
        wasm_bindgen_futures::spawn_local(async move {
            let element = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .query_selector("#among_us_settings_menu")
                .unwrap()
                .unwrap();

            if state2.get() {
                element.set_attribute("aria-hidden", "true").unwrap();
                //among_us_settings_menu.set_inner_html(&(settings_rc.render().into_string()));

                element
                    .set_attribute("style", "width: 349px; height: 177px;")
                    .unwrap();
                sleep(std::time::Duration::from_millis(10)).await;
                element.remove_attribute("aria-hidden").unwrap();
            } else {
                element.set_attribute("aria-hidden", "true").unwrap();
                sleep(std::time::Duration::from_millis(90)).await;
                element.set_attribute("style", "display: none;").unwrap();
                element.remove_attribute("aria-hidden").unwrap();
            }
        });
    }) as Box<dyn FnMut(_)>);
    window
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();

    // Handle submenu clicks
    /*let speed_selector = document
        .query_selector(
            "#among_us_settings_menu>.ytp-panel>.ytp-panel-menu>.ytp-menuitem:nth-child(2)",
        )
        .unwrap()
        .unwrap();
    let closure = Closure::wrap(Box::new(move |event: Event| {
        wasm_bindgen_futures::spawn_local(async move {
            let settings_menu = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .query_selector("#among_us_settings_menu")
                .unwrap()
                .unwrap();

            settings_menu.set_inner_html(r#"<div class="ytp-panel" style="min-width: 250px; width: 349px; height: 177px;"><div class="ytp-panel-menu" role="menu" style="height: 177px;"><div class="ytp-menuitem" role="menuitemcheckbox" aria-checked="true" tabindex="0"><div class="ytp-menuitem-icon"></div><div class="ytp-menuitem-label">Anmerkungen</div><div class="ytp-menuitem-content"><div class="ytp-menuitem-toggle-checkbox"></div></div></div><div class="ytp-menuitem" aria-haspopup="true" role="menuitem" tabindex="0"><div class="ytp-menuitem-icon"></div><div class="ytp-menuitem-label">Wiedergabegeschwindigkeit</div><div class="ytp-menuitem-content">Standard</div></div><div class="ytp-menuitem" aria-haspopup="true" role="menuitem" tabindex="0"><div class="ytp-menuitem-icon"></div><div class="ytp-menuitem-label"><div><span>Untertitel</span><span class="ytp-menuitem-label-count"> (1)</span></div></div><div class="ytp-menuitem-content">Aus</div></div><div class="ytp-menuitem" aria-haspopup="true" role="menuitem" tabindex="0"><div class="ytp-menuitem-icon"></div><div class="ytp-menuitem-label">Qualität</div><div class="ytp-menuitem-content"><div><span>Automatisch</span> <span class="ytp-menu-label-secondary">480p</span></div></div></div></div></div><div class="ytp-panel ytp-panel-animate-forward" style="min-width: 250px; width: 318px; height: 301px;"><div class="ytp-panel-header"><button class="ytp-button ytp-panel-options">Optionen</button><button class="ytp-button ytp-panel-title">Untertitel</button></div><div class="ytp-panel-menu" role="menu" style="height: 97px;"><div class="ytp-menuitem" tabindex="0" role="menuitemradio" aria-checked="true"><div class="ytp-menuitem-label">Aus</div></div><div class="ytp-menuitem" tabindex="0" role="menuitemradio"><div class="ytp-menuitem-label">Französisch (automatisch erzeugt)</div></div></div></div>"#);
            settings_menu.set_attribute("style", "width: 349px; height: 177px;").unwrap();
            sleep(std::time::Duration::from_millis(20)).await;
            settings_menu.set_attribute("style", "width: 258px; height: 154px;").unwrap();
            settings_menu
                .class_list()
                .add_1("ytp-popup-animating")
                .unwrap();
            settings_menu
                .first_element_child()
                .unwrap()
                .class_list()
                .add_1("ytp-panel-animate-back")
                .unwrap();
            settings_menu
                .last_element_child()
                .unwrap()
                .class_list()
                .remove_1("ytp-panel-animate-forward")
                .unwrap();
            sleep(std::time::Duration::from_millis(250)).await;
            settings_menu
                .class_list()
                .remove_1("ytp-popup-animating")
                .unwrap();
            settings_menu.set_inner_html(
                r#"<div class="ytp-panel" style="min-width: 250px; width: 258px; height: 154px;">
                <div class="ytp-panel-header">
                    <button class="ytp-button ytp-panel-options">Optionen</button>
                    <button class="ytp-button ytp-panel-title">Untertitel</button>
                </div>
                <div class="ytp-panel-menu" role="menu" style="height: 97px;">
                    <div class="ytp-menuitem" tabindex="0" role="menuitemradio" aria-checked="true">
                        <div class="ytp-menuitem-label">Aus</div>
                    </div>
                    <div class="ytp-menuitem" tabindex="0" role="menuitemradio">
                        <div class="ytp-menuitem-label">Französisch (automatisch erzeugt)</div>
                    </div>
                </div>
            </div>"#,
            );
        });
    }) as Box<dyn FnMut(_)>);
    speed_selector
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();*/
}

pub fn update_flex_font() {
    let divs = window()
        .unwrap()
        .document()
        .unwrap()
        .get_elements_by_class_name("flex_font");
    for div in 0..divs.length() {
        let div = divs.item(div).unwrap();
        let width = div.client_width() - 10;
        let font_size: f64 = width as f64 * 0.19;
        let html_element: HtmlElement = div.dyn_into().unwrap();
        let style = html_element.style();
        if font_size >= 7.0 {
            style
                .set_property("font-size", &format!("min({}px, 2rem)", font_size))
                .unwrap();
        } else {
            style.set_property("font-size", "0").unwrap();
        }
    }
}

pub async fn remove_previous_display() {
    let window = window().unwrap();
    let container = loop {
        match window
            .document()
            .unwrap()
            .get_elements_by_class_name("ytp-progress-bar-padding")
            .item(0)
        {
            Some(container) => break container,
            None => sleep(std::time::Duration::from_millis(100)).await,
        }
    };
    container.set_inner_html("");
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
            None => sleep(std::time::Duration::from_millis(100)).await,
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
