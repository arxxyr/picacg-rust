param(
    # 你自己的代理地址，http / socks5 都行
    [string]$Proxy = "http://127.0.0.1:10808"
)

# 1) 设置当前 PowerShell 进程的代理环境变量
$env:HTTP_PROXY  = $Proxy
$env:HTTPS_PROXY = $Proxy
$env:ALL_PROXY   = $Proxy
$env:NO_PROXY    = "localhost,127.0.0.1,::1,.local"

Write-Host "已设置代理为: $Proxy"
Write-Host "HTTP_PROXY  = $($env:HTTP_PROXY)"
Write-Host "HTTPS_PROXY = $($env:HTTPS_PROXY)"
Write-Host "ALL_PROXY   = $($env:ALL_PROXY)"
Write-Host "NO_PROXY    = $($env:NO_PROXY)"

# 2) 在这个带代理的环境里运行 claude-code
# 如果你是全局安装：npm install -g claude-code
# 直接用命令名即可
claude --dangerously-skip-permissions

# 如果你是项目里本地安装，可以改成：
# npx claude-code
