#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use dotenv::dotenv;
use rocket::{http::Cookies, request::Form, response::Redirect, Config};
use rocket_contrib::{json::Json, serve::StaticFiles, templates::Template};

mod actions;
mod parser;
mod convertor;

#[derive(FromForm)]
pub struct JumpDetails {
    chat_id: Option<u64>,
    message_id: Option<u64>,
}

#[derive(FromForm)]
pub struct GetMessages {
    message_id: u64,
    position: String,
}

#[derive(FromForm)]
pub struct Query {
    string: String,
    filters: String,
}

#[get("/")]
fn get_index(cookies: Cookies) -> Template {
    let mut backup_path = "";
    let mut chat_id = "";
    if let Some(backup) = cookies.get("backup") {
        backup_path = backup.value();
        if let Some(chat) = cookies.get("chat") {
            chat_id = chat.value();
        }
    }
    Template::render("index", actions::selection_context(backup_path, chat_id))
}

#[get("/reader")]
fn get_reader(cookies: Cookies) -> Result<Template, Redirect> {
    if let Some(backup) = cookies.get("backup") {
        if let Some(chat) = cookies.get("chat") {
            return Ok(Template::render(
                "reader",
                actions::chat(backup.value(), chat.value()),
            ));
        }
    }
    Err(Redirect::to("/"))
}

// POST requests

#[post("/jump", data = "<info>")]
fn post_jump(cookies: Cookies, info: Form<JumpDetails>) -> Json<actions::ChatContext> {
    if let Some(backup) = cookies.get("backup") {
        // If the chat ID is not specified, take the current chat
        let chat_id = match info.chat_id {
            Some(chat_id) => chat_id,
            None => match cookies.get("chat") {
                Some(chat) => chat.value().parse().unwrap(),
                None => return Json(actions::ChatContext::default()),
            }
        };
        let messages = Json(actions::jump_chat(backup.value(), chat_id, info.message_id));
        return messages;
    }
    Json(actions::ChatContext::default())
}

// Used for getting the messages around a specific message ID
#[post("/messages", data = "<info>")]
fn post_messages(cookies: Cookies, info: Form<GetMessages>) -> Json<Vec<actions::Message>> {
    if let Some(backup) = cookies.get("backup") {
        if let Some(chat) = cookies.get("chat") {
            // The required cookies are present, so return the messages
            return Json(actions::get_messages(
                backup.value(),
                chat.value(),
                info.message_id,
                &info.position,
            ));
        }
    }
    // Return an empty vector by default
    Json(Vec::new())
}

#[post("/search", data = "<query>")]
fn post_search(cookies: Cookies, query: Form<Query>) -> Json<Vec<actions::Message>> {
    if let Some(backup) = cookies.get("backup") {
        if let Some(chat) = cookies.get("chat") {
            // The required cookies are present, so return the search results
            return Json(actions::search(backup.value(), chat.value(), &query.string, &query.filters));
        }
    }
    // Return an empty vector by default
    Json(Vec::new())
}

fn configure() -> Config {
    let mut config = Config::active().expect("could not load configuration");
    config.set_port(4000);
    config
}

fn rocket() -> rocket::Rocket {
    rocket::custom(configure())
        .mount(
            "/",
            routes![get_index, get_reader, post_jump, post_messages, post_search],
        )
        .mount("/styles", StaticFiles::from("static/styles"))
        .mount("/scripts", StaticFiles::from("static/scripts"))
        .mount("/fonts", StaticFiles::from("static/fonts"))
        .mount("/", StaticFiles::from(actions::refrigerator()).rank(20))
        .attach(Template::fairing())
}

fn main() {
    // Read environment variables from .env
    dotenv().ok();
    // Start the webserver
    rocket().launch();
}
