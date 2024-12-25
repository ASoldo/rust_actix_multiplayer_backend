use actix::prelude::*;
use actix_web::{Error, HttpRequest, HttpResponse, web};
use actix_web_actors::ws;

/// Define the WebSocket actor
pub struct MyWs {
    pub id: usize, // If you want to track a client ID
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

/// Handle messages (pong, text, binary data, etc.)
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // Echo the text back
                ctx.text(format!("Server echo: {}", text));
            }
            Ok(ws::Message::Binary(bin)) => {
                ctx.binary(bin);
            }
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                // Keep-alive pong response
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

/// WebSocket handshake and start actor
pub async fn ws_index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let ws = MyWs { id: 0 };
    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}
