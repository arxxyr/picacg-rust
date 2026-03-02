# PowerShell 部署脚本 - 将编译产物收集到 bin 目录
# 用法: .\scripts\deploy-windows.ps1 [release|debug]

param(
    [string]$Profile = "release"
)

$ErrorActionPreference = "Stop"

Write-Host "=== PicACG 部署脚本 ===" -ForegroundColor Cyan
Write-Host "编译配置: $Profile" -ForegroundColor Green

# 项目根目录
$RootDir = Split-Path -Parent $PSScriptRoot

# 从 Cargo.toml 提取版本号
$CargoToml = Get-Content (Join-Path $RootDir "Cargo.toml") -Raw
if ($CargoToml -match '\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"') {
    $Version = $Matches[1]
} else {
    Write-Host "错误: 无法从 Cargo.toml 提取版本号" -ForegroundColor Red
    exit 1
}
Write-Host "版本号: v$Version" -ForegroundColor Green

# 目录定义
$BinDir = Join-Path $RootDir "bin"
$TargetDir = Join-Path $RootDir "target\$Profile"

# Step 1: 清理旧的 bin 目录
Write-Host ""
Write-Host "[1/4] 清理旧的 bin 目录..." -ForegroundColor Yellow
if (Test-Path $BinDir) {
    Remove-Item -Recurse -Force $BinDir
    Write-Host "  已删除旧目录: $BinDir" -ForegroundColor Gray
}

# Step 2: 复制可执行文件
Write-Host ""
Write-Host "[2/4] 复制可执行文件..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
$ExeName = "picacg.exe"
$ExePath = Join-Path $TargetDir $ExeName
if (Test-Path $ExePath) {
    Copy-Item $ExePath -Destination $BinDir
    Write-Host "  已复制: $ExeName" -ForegroundColor Green
} else {
    Write-Host "  错误: 找不到可执行文件 $ExePath" -ForegroundColor Red
    Write-Host "  请先运行: cargo build --release -p picacg" -ForegroundColor Yellow
    exit 1
}

# Step 3: 复制内置字体
Write-Host ""
Write-Host "[3/4] 复制内置字体..." -ForegroundColor Yellow
$FontSrc = Join-Path $RootDir "assets\fonts\SarasaTermSCNerd\SarasaTermSCNerd-Regular.ttf"
$FontDestDir = Join-Path $BinDir "assets\fonts\SarasaTermSCNerd"
if (Test-Path $FontSrc) {
    New-Item -ItemType Directory -Force -Path $FontDestDir | Out-Null
    Copy-Item $FontSrc -Destination $FontDestDir
    $FontSize = [math]::Round((Get-Item $FontSrc).Length / 1MB, 1)
    Write-Host "  已复制: SarasaTermSCNerd-Regular.ttf (${FontSize}MB)" -ForegroundColor Green
} else {
    Write-Host "  警告: 未找到内置字体 $FontSrc" -ForegroundColor Yellow
    Write-Host "  程序将回退到系统字体" -ForegroundColor Gray
}

# 部署完成提示
Write-Host ""
Write-Host "=== 部署完成 ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "目录结构:" -ForegroundColor Green
Write-Host "bin/"
Write-Host "├── $ExeName"
Write-Host "└── assets/"
Write-Host "    └── fonts/"
Write-Host "        └── SarasaTermSCNerd/"
Write-Host "            └── SarasaTermSCNerd-Regular.ttf"
Write-Host ""

# Step 4: 创建版本压缩包
Write-Host ""
Write-Host "[4/4] 创建版本压缩包..." -ForegroundColor Yellow
$ZipName = "picacg-v$Version.zip"
$ZipPath = Join-Path $BinDir $ZipName

# 删除旧的 zip 文件
Get-ChildItem -Path $BinDir -Filter "picacg-v*.zip" -ErrorAction SilentlyContinue | Remove-Item -Force

# 创建新的 zip 文件
Push-Location $BinDir
try {
    $ItemsToCompress = Get-ChildItem -Path $BinDir -Exclude "*.zip"
    Compress-Archive -Path $ItemsToCompress.FullName -DestinationPath $ZipPath -Force
    Write-Host "  已创建: bin/$ZipName" -ForegroundColor Green
} finally {
    Pop-Location
}

Write-Host ""
Write-Host "运行方式:" -ForegroundColor Green
Write-Host "  cd bin"
Write-Host '  .\picacg.exe'
Write-Host ""
Write-Host "分发方式:" -ForegroundColor Green
Write-Host "  将 bin/$ZipName 发送给用户"
Write-Host ""
