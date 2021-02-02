use crate::util::sleep;
use maud::{html, Markup, Render};
use std::{fmt::Display, rc::Rc, cell::Cell};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::*;

pub struct Settings<'a> {
    settings: Vec<&'a dyn Setting>,
}

impl<'a> Settings<'a> {
    pub fn new() -> Settings<'a> {
        Settings {
            settings: Vec::new(),
        }
    }

    pub fn add_setting(&mut self, setting: &'a dyn Setting) {
        self.settings.push(setting)
    }
}

impl<'a> Render for Settings<'a> {
    fn render(&self) -> Markup {
        html! {
            .ytp-panel style="width: 349px; height: 177px;" {
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
        CheckBox { id, label, checked: Rc::new(Cell::new(checked)) }
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
            .query_selector(
                &format!("#among_us_settings_menu>.ytp-panel>.ytp-panel-menu>#{}", self.id),
            )
            .unwrap()
            .unwrap();

        let state = Rc::clone(&self.checked);
        let self_element2 = self_element.clone();
        let closure = Closure::wrap(Box::new(move |_: Event| {
            state.set(!state.get());
            self_element2.set_attribute("aria-checked", &state.get().to_string()).unwrap();
        }) as Box<dyn FnMut(_)>);
        self_element
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
}
