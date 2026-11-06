@echo off
title KSK App - Debug Mode
color 0B

echo =======================================================
echo   HE THONG NHAP LIEU KHAM SUC KHOE - QD 3176/QD-BYT
echo   Khoi chay moi truong Debug (Rust / Cargo)
echo =======================================================
echo.

set RUST_BACKTRACE=1

echo [*] Dang kiem tra ma nguon va bien dich nhanh...
cargo run

if %ERRORLEVEL% NEQ 0 (
    echo.
    color 0C
    echo [!] Ung dung gap loi bien dich hoac runtime (Ma loi: %ERRORLEVEL%).
) else (
    echo.
    echo [*] Ung dung da dong binh thuong.
)

echo.
echo Nhan phim bat ky de thoat console...
pause >nul
