use actix_web::{HttpRequest, HttpResponse, Responder};
use bytes::Bytes;
use futures_util::stream::unfold;
use std::time::Duration;
use tokio::time::sleep;

pub async fn sse_endpoint(_req: HttpRequest) -> impl Responder {
    // We start counting from 0 and increment each iteration
    let s = unfold(0, move |mut counter| async move {
        // Wait 2 seconds
        sleep(Duration::from_secs(2)).await;

        // Build SSE-formatted string
        let msg = format!("data: {}\n\n", counter);
        counter += 1;

        // The unfold returns (Item, NextState)
        // Item must be `Result<Bytes, std::io::Error>` so Actix can handle it
        let item = Ok::<Bytes, std::io::Error>(Bytes::from(msg));
        Some((item, counter))
    });

    // The `streaming` method expects `Stream<Item=Result<Bytes,E>>`
    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(s)
}
