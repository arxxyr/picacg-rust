# 设置环境变量
$env:RUST_BACKTRACE = "1"

# 运行程序并捕获所有输出
$output = & cargo run 2>&1 | Out-String

# 输出到控制台
Write-Host $output

# 如果包含 panic 信息，保存到文件
if ($output -match "panic") {
    $output | Out-File -FilePath "crash_log.txt" -Encoding UTF8
    Write-Host "`n崩溃日志已保存到 crash_log.txt" -ForegroundColor Red
}
