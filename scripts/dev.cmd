@echo off
pushd "%~dp0.."
call "%~dp0with-msvc.cmd" npm run tauri dev
set "CODE=%ERRORLEVEL%"
popd
exit /b %CODE%
