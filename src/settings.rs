use crate::util::sleep;
use maud::{html, Markup, Render};
use std::{cell::Cell, fmt::Display, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;

pub struct Settings {
    settings: Vec<&'static dyn Setting>,
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            settings: Vec::new(),
        }
    }

    pub fn add_setting(&mut self, setting: &'static dyn Setting) {
        self.settings.push(setting)
    }

    pub fn enable(self_rc: Rc<Settings>) {
        for setting in &self_rc.settings {
            setting.enable(Rc::clone(&self_rc))
        }
    }
}

impl Render for Settings {
    fn render(&self) -> Markup {
        html! {
            .ytp-panel {
                .ytp-panel-menu role="menu" {
                    @for setting in self.settings.iter() {
                        (**setting)
                    }
                }
            }
        }
    }
}

pub trait Setting: Render {
    fn enable(&self, settings: Rc<Settings>);
}

pub struct CheckBox<T: Display> {
    id: &'static str,
    label: T,
    checked: Rc<Cell<bool>>,
}

impl<T: Display> CheckBox<T> {
    pub fn new(id: &'static str, label: T, checked: bool) -> CheckBox<T> {
        CheckBox {
            id,
            label,
            checked: Rc::new(Cell::new(checked)),
        }
    }
}

impl<T: Display> Render for CheckBox<T> {
    fn render(&self) -> Markup {
        html! {
            .ytp-menuitem role="menuitemcheckbox" id=(self.id) aria-checked=(self.checked.get()) tabindex="0" {
                .ytp-menuitem-icon {}
                .ytp-menuitem-label {(self.label)}
                .ytp-menuitem-content {
                    .ytp-menuitem-toggle-checkbox {}
                }
            }
        }
    }
}

impl<T: Display> Setting for CheckBox<T> {
    fn enable(&self, _settings: Rc<Settings>) {
        let window = window().unwrap();
        let document = window.document().unwrap();
        let self_element = document
            .query_selector(&format!(
                "#among_us_settings_menu>.ytp-panel>.ytp-panel-menu>#{}",
                self.id
            ))
            .unwrap()
            .unwrap();

        let state = Rc::clone(&self.checked);
        let self_element2 = self_element.clone();
        let closure = Closure::wrap(Box::new(move |_: Event| {
            state.set(!state.get());
            self_element2
                .set_attribute("aria-checked", &state.get().to_string())
                .unwrap();
        }) as Box<dyn FnMut(_)>);
        self_element
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
}

pub trait Choice: Display + Copy + PartialEq {
    fn enumerate_values() -> Vec<Self>;
    fn select_value(s: &str) -> Self;
}

pub struct Selection<C: Choice> {
    id: &'static str,
    label: String,
    selected: Rc<Cell<C>>,
}

impl<C: Choice> Selection<C> {
    pub fn new<L: Display>(id: &'static str, label: L, selected: C) -> Selection<C> {
        Selection {
            id,
            label: label.to_string(),
            selected: Rc::new(Cell::new(selected)),
        }
    }
}

impl<C: Choice> Render for Selection<C> {
    fn render(&self) -> Markup {
        html! {
            .ytp-menuitem id=(self.id) aria-haspopup="true" role="menuitem" tabindex="0" {
                .ytp-menuitem-icon {}
                .ytp-menuitem-label {(self.label)}
                .ytp-menuitem-content { (self.selected.get()) }
            }
        }
    }
}

impl<C: Choice + 'static> Setting for Selection<C> {
    fn enable(&self, settings: Rc<Settings>) {
        let window = window().unwrap();
        let document = window.document().unwrap();
        let self_element = document.query_selector(&format!(
                "#among_us_settings_menu>.ytp-panel>.ytp-panel-menu>#{}",
                self.id
            )).unwrap().unwrap();
        let selected = Rc::clone(&self.selected);
        let label = self.label.clone();

        fn handle_click_to_update_value<C: Choice + 'static>(
            selected: Rc<Cell<C>>,
            node: Node,
            value: C,
            settings: Rc<Settings>,
        ) {
            let closure = Closure::wrap(Box::new(move |_: Event| {
                selected.set(value);
                let settings = Rc::clone(&settings);
                wasm_bindgen_futures::spawn_local(async move { animate_back(settings).await });
            }) as Box<dyn FnMut(_)>);
            node.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        };

        async fn animate_back(settings: Rc<Settings>) {
            let settings_menu = web_sys::window().unwrap().document().unwrap().query_selector("#among_us_settings_menu").unwrap().unwrap();
            let new_child = web_sys::window().unwrap().document().unwrap().create_element("div").unwrap();
            settings_menu.append_child(&new_child).unwrap();
            let mut html = settings.render().into_string();
            html = html.replace("\"ytp-panel\"", "\"ytp-panel ytp-panel-animate-back\"");
            new_child.set_outer_html(&html);
            //new_child.class_list().add_1("ytp-panel-animate-back").unwrap();
            sleep(std::time::Duration::from_millis(20)).await;

            let scroll_height = settings_menu.last_element_child().unwrap().scroll_height();
            let height = std::cmp::min(scroll_height, 700);
            settings_menu.set_attribute("style", &format!("height: {}px;", height)).unwrap();
            settings_menu.class_list().add_1("ytp-popup-animating").unwrap();
            settings_menu.first_element_child().unwrap().class_list().add_1("ytp-panel-animate-forward").unwrap();
            settings_menu.last_element_child().unwrap().class_list().remove_1("ytp-panel-animate-back").unwrap();
            sleep(std::time::Duration::from_millis(250)).await;

            settings_menu.class_list().remove_1("ytp-popup-animating").unwrap();
            settings_menu.first_element_child().unwrap().remove();
            Settings::enable(settings);
        }

        async fn animate_forward<C: Choice + 'static>(
            selected: Rc<Cell<C>>,
            label: String,
            settings: Rc<Settings>,
        ) {
            let settings_menu = web_sys::window().unwrap().document().unwrap().query_selector("#among_us_settings_menu").unwrap().unwrap();

            let new_child = web_sys::window().unwrap().document().unwrap().create_element("div").unwrap();
            settings_menu.append_child(&new_child).unwrap();
            let values = C::enumerate_values();
            new_child.set_outer_html(&html! {
                .ytp-panel.ytp-panel-animate-forward {
                    .ytp-panel-header {
                        button .ytp-button.ytp-panel-options {} // Here you can but an option link
                        button .ytp-button.ytp-panel-title {(label)}
                    }
                    .ytp-panel-menu role="menu" {
                        @for item in &values {
                            .ytp-menuitem tabindex="0" role="menuitemradio" aria-checked=((item==&selected.get()).to_string()) {
                                .ytp-menuitem-label {(item)}
                            }
                        }
                    }
                }
            }.into_string());
            let selectable_items = web_sys::window().unwrap().document().unwrap().query_selector_all("#among_us_settings_menu > .ytp-panel-animate-forward > .ytp-panel-menu > .ytp-menuitem").unwrap();
            let mut i = 0;
            while let Some(item) = selectable_items.item(i) {
                handle_click_to_update_value(
                    Rc::clone(&selected),
                    item,
                    values[i as usize],
                    Rc::clone(&settings),
                );
                i += 1;
            }
            sleep(std::time::Duration::from_millis(20)).await;


            let scroll_height = settings_menu.last_element_child().unwrap().scroll_height();
            let height = std::cmp::min(scroll_height, 700);
            settings_menu.set_attribute("style", &format!("height: {}px;", height)).unwrap();
            settings_menu.class_list().add_1("ytp-popup-animating").unwrap();
            settings_menu.first_element_child().unwrap().class_list().add_1("ytp-panel-animate-back").unwrap();
            settings_menu.last_element_child().unwrap().class_list().remove_1("ytp-panel-animate-forward").unwrap();
            sleep(std::time::Duration::from_millis(250)).await;

            settings_menu.class_list().remove_1("ytp-popup-animating").unwrap();
            settings_menu.first_element_child().unwrap().remove();
            settings_menu.first_element_child().unwrap().class_list().remove_1("ytp-panel-animate-forward").unwrap();
        }

        fn spawn_animation_task<C: Choice + 'static>(
            selected: Rc<Cell<C>>,
            label: String,
            settings: Rc<Settings>,
        ) {
            wasm_bindgen_futures::spawn_local(async move {
                animate_forward(selected, label, settings).await
            });
        }

        let closure = Closure::wrap(Box::new(move |_: Event| {
            let selected = Rc::clone(&selected);
            let label = label.clone();
            let settings = Rc::clone(&settings);

            spawn_animation_task(selected, label, settings);
        }) as Box<dyn FnMut(_)>);
        self_element
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
}
