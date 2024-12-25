use actix_web::{HttpRequest, HttpResponse, Responder};
use async_stream::try_stream;
use bytes::Bytes;
use futures_core::stream::Stream;
use std::io;
use std::time::Duration;
use tokio::time::interval;

/// Returns a stream of `Result<Bytes, io::Error>`
fn my_sse_stream() -> impl Stream<Item = Result<Bytes, io::Error>> {
    try_stream! {
        let mut counter = 0;
        let mut ticker = interval(Duration::from_secs(2));

        loop {
            ticker.tick().await;
            let data = format!("data: {}\n\n", counter);
            counter += 1;

            // Yields Ok(Bytes) because it's a "try_stream!"
            yield Bytes::from(data);
        }
    }
}

pub async fn sse_endpoint(_req: HttpRequest) -> impl Responder {
    let stream = my_sse_stream();

    HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(stream)
}
