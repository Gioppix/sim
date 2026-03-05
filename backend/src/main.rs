pub mod ants;

use actix_cors::Cors;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use actix_ws::Message;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::interval;
use ts_rs::TS;

use ants::{World, WorldConfig, WorldSnapshot, snapshot};

pub const TS_EXPORT_FILE: &str = "all.ts";

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = crate::TS_EXPORT_FILE)]
#[serde(tag = "type", content = "payload")]
pub enum FrontToBack {
    Subscribe,
}

#[derive(Serialize, TS)]
#[ts(export, export_to = crate::TS_EXPORT_FILE)]
#[serde(tag = "type", content = "payload")]
pub enum BackToFront {
    WorldStateUpdate(WorldSnapshot),
}

async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    actix_web::rt::spawn(async move {
        let mut world = World::random(WorldConfig::default());
        let mut stream = stream;
        let mut ticker = interval(Duration::from_millis(100));

        loop {
            tokio::select! {
                msg = stream.next() => {
                    match msg {
                        Some(Ok(Message::Ping(bytes))) => {
                            let _ = session.pong(&bytes).await;
                        }
                        Some(Ok(Message::Close(_))) | None => break,
                        _ => {}
                    }
                }
                _ = ticker.tick() => {
                    world.step();
                    let msg = BackToFront::WorldStateUpdate(snapshot(&world));
                    let json = serde_json::to_string(&msg).unwrap();
                    if session.text(json).await.is_err() {
                        break;
                    }
                }
            }
        }

        let _ = session.close(None).await;
    });

    Ok(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .route("/ws", web::get().to(ws_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
