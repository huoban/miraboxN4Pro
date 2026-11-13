@echo off
chcp 65001 >nul
setlocal
cd /d %~dp0

echo ==========================================
echo  N4ProCore 编译（release）
echo  环境：%cd%
echo ==========================================

cargo build --release
if errorlevel 1 (
  echo.
  echo [失败] 编译未通过，请查看上方错误信息
  exit /b 1
)

echo.
echo [成功] 输出文件：%cd%target\release\n4pro-core.exe
endlocal
exit /b 0
