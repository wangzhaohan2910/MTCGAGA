cargo build --release
mv ../target/release/MTCGAGA.exe ../target/release/svc.exe
gcc -static -o ../target/release/watchdog.exe watchdog.c -lwtsapi32 -luserenv -mwindows