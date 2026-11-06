@echo off
chcp 65001 >nul
title KSK App - Debug Mode
color 0B

echo =======================================================
echo   HỆ THỐNG NHẬP LIỆU KHÁM SỨC KHỎE - QĐ 3176/QĐ-BYT
echo   Khởi chạy môi trường Debug (Rust / Cargo)
echo =======================================================
echo.

:: Thiết lập log môi trường để debug chi tiết khi cần
set RUST_BACKTRACE=1

echo [*] Đang kiểm tra mã nguồn và biên dịch nhanh...
cargo run

if %ERRORLEVEL% NEQ 0 (
    echo.
    color 0C
    echo [!] Ứng dụng đã thoát hoặc gặp lỗi biên dịch/runtime (Error Code: %ERRORLEVEL%).
) else (
    echo.
    echo [*] Ứng dụng đã đóng bình thường.
)

echo.
echo Nhấn phím bất kỳ để thoát console...
pause >nul

