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

pub trait Choice: Display + Copy {
    fn enumerate_values() -> Vec<String>;
    fn select_value(s: &str) -> Self;
}

pub struct Selection<L: Display, C: Choice> {
    id: &'static str,
    label: L,
    selected: Rc<Cell<C>>,
}

impl<L: Display, C: Choice> Selection<L, C> {
    pub fn new(id: &'static str, label: L, selected: C) -> Selection<L, C> {
        Selection {
            id,
            label,
            selected: Rc::new(Cell::new(selected)),
        }
    }
}

impl<L: Display, C: Choice> Render for Selection<L, C> {
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

impl<L: Display, C: Choice> Setting for Selection<L, C> {
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
                settings_menu
                    .set_attribute("style", "width: 349px; height: 177px;")
                    .unwrap();
                sleep(std::time::Duration::from_millis(20)).await;
                settings_menu
                    .set_attribute("style", "width: 258px; height: 154px;")
                    .unwrap();
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
        self_element
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
}
