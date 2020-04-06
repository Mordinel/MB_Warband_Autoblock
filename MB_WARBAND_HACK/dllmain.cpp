// dllmain.cpp : Defines the entry point for the DLL application.

#include "pch.h"

DWORD WINAPI HackThread(HMODULE hModule) {
    AllocConsole();
    FILE* f;
    freopen_s(&f, "CONOUT$", "w", stdout);

    LPCWSTR tmpResourceLock;
    HANDLE enableSound = LoadResource(hModule, FindResource(hModule, L"enable", L"WAVE"));
    HANDLE disableSound = LoadResource(hModule, FindResource(hModule, L"disable", L"WAVE"));

    uintptr_t moduleBase = (uintptr_t)GetModuleHandle(L"mb_warband.exe");

    printf("Press INSERT to toggle Autoblock\n");
    printf("Press END to eject the DLL\n");

    bool bAutoblock = false;

    for (;;) {
        if (GetAsyncKeyState(VK_END) & 1) {
            break;
        }

        if (GetAsyncKeyState(VK_INSERT) & 1) {
            bAutoblock = !bAutoblock;
            if (bAutoblock) {
                PlaySound((LPCWSTR)enableSound, hModule, SND_MEMORY | SND_ASYNC | SND_NODEFAULT);
                printf("Autoblock On\n");
            }
            else 
            {
                PlaySound((LPCWSTR)disableSound, hModule, SND_MEMORY | SND_ASYNC | SND_NODEFAULT);
                printf("Autoblock Off\n");
            }
        }

        uintptr_t* autoBlockPtr = (uintptr_t*)(moduleBase + 0x47C2F4);

        if (autoBlockPtr) {
            if (bAutoblock) {
                *autoBlockPtr = 0;
            }
            else {
                *autoBlockPtr = 1;
            }
        }
        Sleep(5);
    }


    // Free memory
    if (f != NULL) {
        fclose(f);
    }

    if (enableSound != NULL) {
        FreeResource(enableSound);
    }

    if (disableSound != NULL) {
        FreeResource(disableSound);
    }

    FreeConsole();
    FreeLibraryAndExitThread(hModule, 0);

    return 0;
}

BOOL APIENTRY DllMain( HMODULE hModule,
                       DWORD  ul_reason_for_call,
                       LPVOID lpReserved
                     )
{
    switch (ul_reason_for_call)
    {
    case DLL_PROCESS_ATTACH:
        CloseHandle(CreateThread(nullptr, 0, (LPTHREAD_START_ROUTINE)HackThread, hModule, 0, nullptr));
    case DLL_THREAD_ATTACH:
    case DLL_THREAD_DETACH:
    case DLL_PROCESS_DETACH:
        break;
    }
    return TRUE;
}

