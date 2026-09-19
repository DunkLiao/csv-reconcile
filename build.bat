@echo off
setlocal EnableExtensions EnableDelayedExpansion

REM ============================================================
REM  CSV Compare - Windows Desktop Build Script
REM
REM  Builds the release executable and installers with Tauri 2,
REM  then stages a copy of the executable into the "portable"
REM  folder in the project root.
REM
REM  Usage:
REM    build.bat              Build release EXE + NSIS / MSI bundles
REM    build.bat portable     Build release EXE only (no installer)
REM    build.bat debug        Build debug EXE + bundles
REM    build.bat help         Show this help
REM
REM  Requirements:
REM    - Node.js 18+  (https://nodejs.org)
REM    - Rust / Cargo (https://rustup.rs)
REM    - Microsoft Edge WebView2 Runtime (preinstalled on Windows 10/11)
REM ============================================================

cd /d "%~dp0"

set "MODE=%~1"
if "%MODE%"=="" set "MODE=release"

if /I "%MODE%"=="-h"     goto show_help
if /I "%MODE%"=="--help" goto show_help
if /I "%MODE%"=="/?"     goto show_help
if /I "%MODE%"=="help"   goto show_help

echo.
echo ============================================================
echo   CSV Compare - Desktop Build
echo ============================================================
echo.

REM ------------------------------------------------------------
REM  1. Check the toolchain
REM ------------------------------------------------------------
where node >nul 2>nul
if errorlevel 1 (
    echo [ERROR] Node.js was not found in PATH.
    echo         Install it from https://nodejs.org/ and try again.
    goto fail
)

where npm >nul 2>nul
if errorlevel 1 (
    echo [ERROR] npm was not found in PATH.
    goto fail
)

where cargo >nul 2>nul
if errorlevel 1 (
    echo [ERROR] Rust / Cargo was not found in PATH.
    echo         Install it from https://rustup.rs/ and try again.
    goto fail
)

echo [1/4] Toolchain check passed.
echo       Node  :
node --version
echo       npm   :
call npm --version
echo       Cargo :
cargo --version
echo.

REM ------------------------------------------------------------
REM  2. Install frontend dependencies when missing
REM ------------------------------------------------------------
if not exist "node_modules" (
    echo [2/4] Installing npm dependencies ^(first run^)...
    call npm install
    if errorlevel 1 (
        echo [ERROR] npm install failed.
        goto fail
    )
) else (
    echo [2/4] npm dependencies already present - skipping install.
)
echo.

REM ------------------------------------------------------------
REM  3. Build the desktop application
REM ------------------------------------------------------------
echo [3/4] Building desktop application ^(mode: %MODE%^)...
echo.

if /I "%MODE%"=="portable" goto build_portable
if /I "%MODE%"=="debug"    goto build_debug
goto build_release

:build_release
REM Full release build: frontend + EXE + NSIS installer + MSI
call npm run build:desktop
goto after_build

:build_portable
REM Release EXE only, without installer bundles
call npm run tauri -- build --no-bundle
goto after_build

:build_debug
REM Debug build (faster, with debug symbols)
call npm run tauri -- build --debug
goto after_build

:after_build
if errorlevel 1 (
    echo.
    echo [ERROR] Build failed. See the output above for details.
    goto fail
)

REM ------------------------------------------------------------
REM  4. Stage a copy of the executable into the portable folder
REM ------------------------------------------------------------
if /I "%MODE%"=="debug" (
    set "SRC_EXE=src-tauri\target\debug\CSVCompare.exe"
) else (
    set "SRC_EXE=src-tauri\target\release\CSVCompare.exe"
)
set "DST_DIR=portable"
set "DST_EXE=%DST_DIR%\CSVCompare.exe"

echo.
echo [4/4] Staging executable into "%DST_DIR%"...

if not exist "%DST_DIR%" mkdir "%DST_DIR%"

if exist "!SRC_EXE!" (
    copy /Y "!SRC_EXE!" "!DST_EXE!" >nul
    if errorlevel 1 (
        echo [WARN] Failed to copy "!SRC_EXE!" to "!DST_EXE!".
    ) else (
        echo       Copied: !DST_EXE!
    )
) else (
    echo [WARN] Executable not found: !SRC_EXE!
    echo        Skipping copy into the portable folder.
)

REM ------------------------------------------------------------
REM  Report the produced artifacts (mode aware)
REM ------------------------------------------------------------
echo.
echo ============================================================
echo   Build completed successfully.
echo ============================================================
echo.
echo Artifacts produced:

if /I "%MODE%"=="portable" goto report_portable
if /I "%MODE%"=="debug"    goto report_debug
goto report_release

:report_release
if exist "src-tauri\target\release\CSVCompare.exe" echo   [EXE]     src-tauri\target\release\CSVCompare.exe
if exist "src-tauri\target\release\bundle\nsis"    echo   [NSIS]    src-tauri\target\release\bundle\nsis\
if exist "src-tauri\target\release\bundle\msi"     echo   [MSI]     src-tauri\target\release\bundle\msi\
if exist "%DST_EXE%"                               echo   [PORTABLE] %DST_EXE%
goto report_done

:report_portable
if exist "src-tauri\target\release\CSVCompare.exe" (
    echo   [EXE]     src-tauri\target\release\CSVCompare.exe
) else (
    echo   (no artifact detected - check the build output)
)
if exist "%DST_EXE%" echo   [PORTABLE] %DST_EXE%
goto report_done

:report_debug
if exist "src-tauri\target\debug\CSVCompare.exe" echo   [EXE]     src-tauri\target\debug\CSVCompare.exe
if exist "src-tauri\target\debug\CSVCompare.exe" echo   [NOTE]    Debug bundle output is under src-tauri\target\debug\bundle\
if exist "%DST_EXE%"                             echo   [PORTABLE] %DST_EXE%
goto report_done

:report_done
echo.
echo Done.
endlocal
exit /b 0

REM ------------------------------------------------------------
REM  Helpers
REM ------------------------------------------------------------
:show_help
echo.
echo CSV Compare - Windows Desktop Build Script
echo.
echo Usage:
echo   build.bat              Build release EXE + NSIS / MSI bundles
echo   build.bat portable     Build release EXE only (no installer)
echo   build.bat debug        Build debug EXE + bundles
echo   build.bat help         Show this help
echo.
echo A copy of the built executable is staged into the "portable" folder.
echo.
endlocal
exit /b 0

:fail
echo.
echo Build aborted.
endlocal
exit /b 1
