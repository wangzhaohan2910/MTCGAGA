@echo off
set SERVICE_NAME=svc
set EXE_PATH=C:\Windows\watchdog.exe

:: net session >nul 2>&1 || (echo 请以管理员身份运行 & pause & exit /b 1)

copy /Y "%~dp0watchdog.exe" C:\Windows\ >nul
copy /Y "%~dp0svc.exe" C:\Windows\ >nul
if errorlevel 1 (echo 复制失败 & pause & exit /b 1)

sc stop %SERVICE_NAME% >nul 2>&1
sc delete %SERVICE_NAME% >nul 2>&1

sc create %SERVICE_NAME% type= own start= auto binPath= "%EXE_PATH%"
if errorlevel 1 (echo 创建服务失败 & pause & exit /b 1)

sc start %SERVICE_NAME%
echo 服务当前状态：
sc query %SERVICE_NAME%

exit