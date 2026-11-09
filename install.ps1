$ErrorActionPreference = "Stop"

$Repo = if ($env:LEARN_GIT_INSTALL_REPO) { $env:LEARN_GIT_INSTALL_REPO } else { "MonsieurLazar/learn-git" }
$Binary = "learn-git"
$InstallDir = if ($env:LEARN_GIT_INSTALL_DIR) { $env:LEARN_GIT_INSTALL_DIR } else { Join-Path $HOME ".local\bin" }
$Version = if ($env:LEARN_GIT_VERSION) { $env:LEARN_GIT_VERSION } else { $null }

$Arch = switch ($env:PROCESSOR_ARCHITECTURE) {
    "AMD64" { "x86_64" }
    "ARM64" { "aarch64" }
    default { throw "unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
}

$Target = "$Arch-pc-windows-msvc"

if (-not $Version) {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers @{ "User-Agent" = "learn-git-installer" }
    $Version = $release.tag_name
}

$Asset = "$Binary-$Version-$Target.tar.gz"
$Base = "https://github.com/$Repo/releases/download/$Version"
$Tmp = Join-Path $env:TEMP ("learn-git-" + [guid]::NewGuid().ToString("N"))

New-Item -ItemType Directory -Path $Tmp | Out-Null

try {
    Invoke-WebRequest -Uri "$Base/$Asset" -OutFile (Join-Path $Tmp $Asset)
    Invoke-WebRequest -Uri "$Base/SHA256SUMS" -OutFile (Join-Path $Tmp "SHA256SUMS")

    $expected = (Get-Content (Join-Path $Tmp "SHA256SUMS") | Where-Object { $_ -match [regex]::Escape($Asset) } | ForEach-Object { ($_ -split '\s+')[0] })
    $actual = (Get-FileHash (Join-Path $Tmp $Asset) -Algorithm SHA256).Hash.ToLower()

    if ($expected -ne $actual) {
        throw "checksum mismatch for $Asset"
    }

    tar -xzf (Join-Path $Tmp $Asset) -C $Tmp

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Copy-Item (Join-Path $Tmp "$Binary.exe") (Join-Path $InstallDir "$Binary.exe") -Force

    Write-Host "installed $Binary $Version to $InstallDir\$Binary.exe"

    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$InstallDir*") {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
        Write-Host "added $InstallDir to your user PATH, restart your terminal"
    }
}
finally {
    Remove-Item -Recurse -Force $Tmp
}
