use wasm_bindgen::{JsValue, JsCast};
use js_sys::Reflect::{get, apply};

pub fn get_game_name(object: &JsValue) -> Option<String> {
    let mut game_name = get(&object, &"contents".into()).ok()?;
    game_name = get(&game_name, &"twoColumnWatchNextResults".into()).ok()?;
    game_name = get(&game_name, &"results".into()).ok()?;
    game_name = get(&game_name, &"results".into()).ok()?;
    game_name = get(&game_name, &"contents".into()).ok()?;
    game_name = get(&game_name, &1.into()).ok()?;
    game_name = get(&game_name, &"videoSecondaryInfoRenderer".into()).ok()?;
    game_name = get(&game_name, &"metadataRowContainer".into()).ok()?;
    game_name = get(&game_name, &"metadataRowContainerRenderer".into()).ok()?;
    game_name = get(&game_name, &"rows".into()).ok()?;
    game_name = get(&game_name, &0.into()).ok()?;
    game_name = get(&game_name, &"richMetadataRowRenderer".into()).ok()?;
    game_name = get(&game_name, &"contents".into()).ok()?;
    game_name = get(&game_name, &0.into()).ok()?;
    game_name = get(&game_name, &"richMetadataRenderer".into()).ok()?;
    game_name = get(&game_name, &"title".into()).ok()?;
    game_name = get(&game_name, &"simpleText".into()).ok()?;
    game_name.as_string()
}

pub fn get_storyboard(mut object: JsValue) -> Option<String> {
    object = get(&object, &"storyboards".into()).ok()?;
    object = get(&object, &"playerStoryboardSpecRenderer".into()).ok()?;
    object = get(&object, &"spec".into()).ok()?;
    object.as_string()
}

pub fn parse_json(text: String) -> Option<JsValue> {
    let json = get(&web_sys::window()?.into(), &"JSON".into()).ok()?;
    let json_parse = get(&json, &"parse".into()).ok()?;
    let params = js_sys::Array::new();
    params.push(&text.into());
    apply(&json_parse.dyn_into().ok()?, &JsValue::NULL, &params).ok()
}