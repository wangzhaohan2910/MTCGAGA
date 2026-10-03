# MTCGAGA

Make TCGA Great Again.

MTCGAGA is a Windows-only Rust program that serves a small local web control panel for performing interactive host actions: running shell/PowerShell commands, injecting text and input events, capturing keyboard events and screen images, and producing ZIP archives of filesystem paths. The README below documents the program's real behavior as implemented in src/main.rs.

Important: this tool can execute arbitrary commands, capture keyboard and screen data, and simulate input. Only build and run it on systems where you have explicit authorization.

## Quick facts
- Language: Rust
- Entry point: src/main.rs
- Default listening port: 1145
- Bind address: 0.0.0.0 (accessible on all interfaces, including localhost)
- No console window at runtime (compiled with windows_subsystem = "windows")
- Creates hidden log files: C:\Windows\opt.log and C:\Windows\full.log
- License: CC0-1.0 (see LICENSE)

## Build
Requirements:
- Windows (10/11)
- Rust toolchain (cargo, rustc)
- Build environment for native crates (MSVC or appropriate toolchain)

From the repository root:
cargo build --release

The release binary will be at:
target/release/MTCGAGA.exe

You may rename it as needed.

## Run
Usage:
./target/release/MTCGAGA.exe [PORT]

- If you pass a port (e.g., 8080), the server listens on that port.
  Example: ./target/release/MTCGAGA.exe 8080 → visit http://127.0.0.1:8080/
- If no argument is given, the program listens on port 1145: http://127.0.0.1:1145

The program binds to 0.0.0.0:<port>, so it will be reachable on localhost and any other interface on that host (subject to firewall rules).

## HTTP endpoints
- GET /  
  Returns a simple HTML control page (an interactive form and a live-updating display of the two log files). The page contains:
  - A form that can submit one or more action items (select + text input). The UI allows adding/removing items in the browser before submitting.
  - An image element updated from the WebSocket-based screen stream.
  - A pre element showing contents of the keyboard and event logs read from C:\Windows\full.log and C:\Windows\opt.log.

- POST /  
  The form on the root page POSTs to `/`. The server expects a form with zero or more name/value pairs; each pair is interpreted as one action. For each pair the server matches the field name (the action code) and executes the corresponding behavior.

  Actions implemented (name → behavior):
  - cmd: spawn `cmd /c <value>` (non-blocking). Created with no console (CREATE_NO_WINDOW) so it runs detached.
  - pow: spawn `powershell -Command <value>` (non-blocking).
  - cmb: run `cmd /c <value>` and await its exit (blocking).
  - pob: run `powershell -Command <value>` and await its exit (blocking).
  - clr: clear both log files (C:\Windows\full.log and C:\Windows\opt.log) — truncates them to zero length.
  - txt: inject the supplied text into the currently active window (uses Enigo).
  - zip: treat the value as a file or directory path on disk, recursively collect it into an in-memory ZIP archive, and return that ZIP as the HTTP response with filename archive.zip and content-type application/zip.
  - mre: mouse relative move — value should be two whitespace-separated numbers: dx dy (relative coordinates).
  - mab: mouse absolute move — value should be two whitespace-separated numbers: x y (absolute coordinates).
  - scr: scroll vertically — value parsed as an integer (scroll amount).
  - key: simulate key/mouse events. The server parses space-separated tokens; tokens have prefixes to indicate press/release/click and target:
      - tokens starting with `p` = press, `r` = release, or (no prefix) = click.
      - second character indicates which button/key: `l` left, `r` right, `c` middle, otherwise `Other(<num>)` is used for other key codes.
      - Examples:
        - `"pl"` → press left mouse button
        - `"rl"` → release left mouse button
        - `"cl"` or `"l"` → click left mouse button
        - `"p123"` → press key with code 123
  - dly: sleep/delay — value parsed as a number of nanoseconds (Duration::from_nanos_u128). This delays handling using tokio::sleep.

  If at least one `zip` action is present in the request, the server returns the generated ZIP file (application/zip). If no zip action is present the handler returns HTTP 204 No Content.

- GET /screen.webp (WebSocket upgrade)
  This route expects a WebSocket upgrade. Upon upgrade the server enters an infinite loop capturing the primary display as a WebP image (webp-screenshot-rust) with include_cursor = true, and sends the binary WebP frames as WebSocket messages. The HTML page opens a WebSocket to `/screen.webp` and updates an <img> element with each received frame.

## Logging and files
- On startup the program opens (and creates if missing) two hidden files:
  - C:\Windows\opt.log
  - C:\Windows\full.log
  Both files are opened with hidden attributes (OpenOptions::attributes(2)) and are used as appendable logs.
- device_query is used to hook local keyboard events (on_key_down/on_key_up) and append readable markers to C:\Windows\full.log.
- rdev::listen is used (inside a spawned task) to capture lower-level device events and append text to C:\Windows\opt.log.
- The root page reads both files and displays their contents on the web UI.

## Behavior details and caveats
- Many actions perform privileged or invasive operations:
  - Arbitrary command execution (cmd/pow/cmb/pob).
  - Keyboard event logging and screen capture (privacy sensitive).
  - Input simulation that can move cursor and inject keystrokes.
  - Reading arbitrary filesystem paths for zipping.
- Commands spawned with creation_flags 0x08000000 run without creating a console window (detached/hidden).
- The WebSocket screen streamer loops without an explicit delay — frame rate will be as fast as capture and network allow. This can be CPU intensive.
- zip action expects the given path(s) to be accessible to the process; file permissions or missing paths will produce errors.
- The program runs without a console window (windows_subsystem = "windows"); you must inspect logs or use Task Manager to see/terminate the process.
- Because the server binds to 0.0.0.0, ensure your firewall rules permit/deny access appropriately before exposing the service.

## Examples
- Start on default port:
  ./target/release/MTCGAGA.exe
  Visit: http://127.0.0.1:1145/

- Start on port 8080:
  ./target/release/MTCGAGA.exe 8080
  Visit: http://127.0.0.1:8080/

- Run a single background command via curl:
  curl -X POST -F 'cmd=whoami' http://127.0.0.1:1145/

- Request a blocking PowerShell command:
  curl -X POST -F 'pob=Get-Process' http://127.0.0.1:1145/

- Request a zip of a path and save it:
  curl -X POST -F 'zip=C:\path\to\folder' http://127.0.0.1:1145/ --output archive.zip

## Security & legal notice
This program can capture user input and screen data and can execute arbitrary commands. Use it only on systems where you have explicit permission. Do not deploy this tool on production, shared, or untrusted networks without careful access controls. The author and repository do not endorse unethical or illegal usage; you are responsible for complying with all applicable laws and policies.

## Project structure
MTCGAGA/
- Cargo.toml
- Cargo.lock
- LICENSE (CC0-1.0)
- README.md
- src/main.rs
- .gitignore

## Dependencies (high-level)
- axum — HTTP server and WebSocket support
- tokio — async runtime
- device_query, rdev — keyboard / device events
- enigo — input simulation (mouse/keyboard/text injection)
- webp-screenshot-rust — screen capture
- zip — archive creation

## License
This project is released under the Creative Commons Zero v1.0 Universal (CC0-1.0). See LICENSE for details.
