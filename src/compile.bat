cargo build --release
mv ../target/release/MTCGAGA.exe ../target/release/svc.exe
gcc -Oz -s -mwindows -o ../target/release/wd.exe wd.cpp -luser32 -lkernel32 -ladvapi32