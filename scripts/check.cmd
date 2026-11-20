@echo off
pushd "%~dp0.."
call "%~dp0with-msvc.cmd" cargo check --manifest-path src-tauri\Cargo.toml
set "CODE=%ERRORLEVEL%"
popd
exit /b %CODE%
