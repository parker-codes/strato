#[macro_use]
extern crate rocket;

use rocket::form::Form;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::{Shutdown, State};

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(channel::<ServerUpdate>(1024).0)
        .mount("/", routes![index, post, events])
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, UriDisplayQuery))]
#[serde(crate = "rocket::serde")]
struct Message {
    #[field(validate = len(..30))]
    pub room: String,
    #[field(validate = len(..20))]
    pub username: String,
    pub message: String,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

/// Receive a message from a form submission and broadcast it to any receivers.
#[post("/message", data = "<form>")]
fn post(form: Form<Message>, queue: &State<Sender<ServerUpdate>>) {
    let update = ServerUpdate::GameStateChanged;

    // A send 'fails' if there are no active subscribers. That's okay.
    let _res = queue.send(update);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
enum ServerUpdate {
    GameStateChanged,
    PlayerJoined,
    PlayerStartedTurn,
    PlayerEndedTurn,
}

/// Returns an infinite stream of server-sent events. Each event is a message
/// pulled from a broadcast queue.
#[get("/events")]
async fn events(queue: &State<Sender<ServerUpdate>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);
        }
    }
}
