@echo off
setlocal

if not defined PORT set "PORT=8000"

cd /d "%~dp0" || exit /b 1

powershell -NoProfile -ExecutionPolicy Bypass -Command "$listeners = Get-NetTCPConnection -LocalPort %PORT% -State Listen -ErrorAction SilentlyContinue; $ids = $listeners | Select-Object -ExpandProperty OwningProcess -Unique; if ($ids) { Write-Output ('Port %PORT% is in use; terminating stale listener(s): ' + ($ids -join ' ')); foreach ($id in $ids) { Stop-Process -Id $id -Force } }"
if errorlevel 1 exit /b %errorlevel%

if exist "pkg\incredicalculator_web.js" if exist "pkg\incredicalculator_web_bg.wasm" goto bundle_ready

echo Browser bundle not found; building it now...
where wasm-pack >nul 2>nul
if errorlevel 1 (
    echo wasm-pack 0.13.1 is required. Install it with: cargo install --locked --version 0.13.1 wasm-pack
    exit /b 1
)
call wasm-pack build --target web --out-dir pkg
if errorlevel 1 exit /b %errorlevel%

:bundle_ready
echo Serving the app at http://127.0.0.1:%PORT%
echo Press Ctrl+C to stop.
echo If you prefer another port, run: set PORT=4173 ^& start-browser.bat

where py >nul 2>nul
if errorlevel 1 goto use_python
py -3 -m http.server %PORT% --bind 127.0.0.1
exit /b %errorlevel%

:use_python
where python >nul 2>nul
if errorlevel 1 goto no_python
python -m http.server %PORT% --bind 127.0.0.1
exit /b %errorlevel%

:no_python
echo Python 3 is required to run the local browser server.
exit /b 1
