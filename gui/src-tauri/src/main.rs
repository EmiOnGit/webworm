// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use state::Storage;
use tauri::State;

use crate::js_bookmark::JsBookmark;
mod js_bookmark;
mod state;
#[tauri::command]
async fn fetch_bookmarks(storage: State<'_, Storage>) -> Result<Vec<String>, ()> {
    let res: Vec<String> = storage
        .bookmarks
        .lock()
        .unwrap()
        .0
        .iter_mut()
        .map(|b| JsBookmark::from_bookmark(b))
        .map(|b| serde_json::to_string(&b).unwrap())
        .collect();
    for l in &res {
        println!("{l}");
    }
    Ok(res)
}
#[tauri::command]
fn previous(storage: State<'_, Storage>, name: &str) -> Result<String, ()> {
    let js_bookmark = {
        let mut bookmarks = storage.bookmarks.lock().unwrap();
        let bookmark = bookmarks.previous(name).unwrap();
        bookmark.can_advance();
        JsBookmark::from_bookmark(bookmark)
    };
    storage.save();
    Ok(serde_json::to_string(&js_bookmark).unwrap())
}
#[tauri::command]
fn advance(storage: State<'_, Storage>, name: &str) -> Result<String, ()> {
    let js_bookmark = {
        let mut bookmarks = storage.bookmarks.lock().unwrap();
        let bookmark = bookmarks.advance(name).unwrap();
        bookmark.can_advance();
        JsBookmark::from_bookmark(bookmark)
    };
    storage.save();
    Ok(serde_json::to_string(&js_bookmark).unwrap())
}
#[tauri::command]
fn insert(storage: State<Storage>, entry: &str) -> Result<String, String> {
    let js_bookmark = {
        let Ok(js_bookmark): Result<JsBookmark, _> = serde_json::from_str(entry) else {
            let e = format!("could not load bookmark from {}", entry);

        return Err(e);
    };
        let bookmark = js_bookmark.into_bookmark()?;
        let mut store = storage.bookmarks.lock().unwrap();
        if store.push_entry(bookmark.clone()).is_err() {
            let e = format!("the name {} already exists", bookmark.name);
            return Err(e);
        }
        store.update(&js_bookmark.name);
        JsBookmark::from_bookmark(store.get(js_bookmark.name.as_str()).unwrap())
    };
    storage.save();
    Ok(serde_json::to_string(&js_bookmark).unwrap())
}
#[tauri::command]
fn remove(storage: State<Storage>, name: &str) {
    {
        storage.bookmarks.lock().unwrap().remove(name);
    }
    storage.save();
}
fn main() {
    tauri::Builder::default()
        .manage(Storage::default())
        .invoke_handler(tauri::generate_handler![
            fetch_bookmarks,
            previous,
            advance,
            insert,
            remove,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
