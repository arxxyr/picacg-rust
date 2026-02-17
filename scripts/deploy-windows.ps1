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
$BinAssetsDir = Join-Path $BinDir "assets"
$BinFontsDir = Join-Path $BinAssetsDir "fonts"
$TargetDir = Join-Path $RootDir "target\$Profile"
$AssetsDir = Join-Path $RootDir "assets"

# Step 1: 清理旧的 bin 目录
Write-Host ""
Write-Host "[1/5] 清理旧的 bin 目录..." -ForegroundColor Yellow
if (Test-Path $BinDir) {
    Remove-Item -Recurse -Force $BinDir
    Write-Host "  已删除旧目录: $BinDir" -ForegroundColor Gray
}

# Step 2: 创建目录结构
Write-Host ""
Write-Host "[2/5] 创建目录结构..." -ForegroundColor Yellow
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
New-Item -ItemType Directory -Force -Path $BinFontsDir | Out-Null
Write-Host "  已创建: bin/" -ForegroundColor Gray
Write-Host "  已创建: bin/assets/fonts/" -ForegroundColor Gray

# Step 3: 复制可执行文件
Write-Host ""
Write-Host "[3/5] 复制可执行文件..." -ForegroundColor Yellow
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

# Step 4: 复制字体文件
Write-Host ""
Write-Host "[4/5] 复制字体文件..." -ForegroundColor Yellow
$FontsSourceDir = Join-Path $AssetsDir "fonts"
if (Test-Path $FontsSourceDir) {
    # 只复制 Regular 字体（运行时必须）
    $FontFile = Join-Path $FontsSourceDir "SarasaTermSCNerd-Regular.ttf"
    if (Test-Path $FontFile) {
        Copy-Item $FontFile -Destination $BinFontsDir
        Write-Host "  已复制: SarasaTermSCNerd-Regular.ttf" -ForegroundColor Green
    } else {
        Write-Host "  警告: 未找到字体文件" -ForegroundColor DarkYellow
    }
} else {
    Write-Host "  警告: 字体目录不存在 $FontsSourceDir" -ForegroundColor DarkYellow
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
Write-Host "        └── SarasaTermSCNerd-Regular.ttf"

# Step 5: 创建版本压缩包
Write-Host ""
Write-Host "[5/5] 创建版本压缩包..." -ForegroundColor Yellow
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
