# Compila los dos ejecutables sin rutas personales y los deja en dist/.
# Uso: powershell -ExecutionPolicy Bypass -File .\build-release.ps1

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

$path = "$HOME\.cargo\bin;$env:PATH"
$env:PATH = $path

$target = if ($env:CARGO_TARGET_DIR) {
    $env:CARGO_TARGET_DIR
} else {
    Join-Path $root "target"
}
$cliTarget = Join-Path $target "cli-only"

# CARGO_ENCODED_RUSTFLAGS usa 0x1f como separador: las rutas con espacios no
# se parten como pasaria con RUSTFLAGS.
$sep = [char]0x1f
$env:CARGO_ENCODED_RUSTFLAGS = @(
    "--remap-path-prefix=$env:USERPROFILE\.cargo=cargo",
    "--remap-path-prefix=$env:USERPROFILE=home",
    "--remap-path-prefix=$root=src"
) -join $sep
Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue

cargo build --release --target-dir $target
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
cargo build --release --no-default-features --features cli --target-dir $cliTarget
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

New-Item -ItemType Directory -Force (Join-Path $root "dist") | Out-Null

function Copy-Release($source, $destination) {
    for ($i = 0; $i -lt 5; $i++) {
        try {
            Copy-Item $source $destination -Force -ErrorAction Stop
            return
        } catch {
            Start-Sleep -Seconds 1
        }
    }
    throw "No se pudo escribir $destination. Cierra Smart Organizer y vuelve a ejecutar el script."
}

Copy-Release (Join-Path $target "release\smart-organizer.exe") (Join-Path $root "dist\Smart Organizer.exe")
Copy-Release (Join-Path $cliTarget "release\smart-organizer-cli.exe") (Join-Path $root "dist\smart-organizer-cli.exe")

# Firma opcional. Si faltan las variables, el ejecutable se publica sin firmar
# con un aviso: compilar nunca debe fallar por no tener un certificado a mano.
$timestamp = if ($env:SIGN_TIMESTAMP_URL) {
    $env:SIGN_TIMESTAMP_URL
} else {
    "http://timestamp.digicert.com"
}
$signtool = Get-Command signtool.exe -ErrorAction SilentlyContinue
if (-not $signtool -and $env:SIGNTOOL_PATH) {
    $signtool = Get-Item $env:SIGNTOOL_PATH -ErrorAction SilentlyContinue
}
if (-not $signtool) {
    $signtool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Filter signtool.exe -Recurse -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match "\\x64\\signtool\.exe$" } |
        Select-Object -Last 1
}

$signing = $false
if ($signtool) {
    $signtoolPath = if ($signtool -is [System.Management.Automation.ApplicationInfo]) {
        $signtool.Source
    } else {
        $signtool.FullName
    }
    if ($env:SIGN_CERT_THUMBPRINT) {
        $signArgs = @("sign", "/sha1", $env:SIGN_CERT_THUMBPRINT, "/fd", "sha256", "/tr", $timestamp, "/td", "sha256")
        $signing = $true
    } elseif ($env:SIGN_CERT_PATH -and $env:SIGN_CERT_PASSWORD) {
        $signArgs = @("sign", "/f", $env:SIGN_CERT_PATH, "/p", $env:SIGN_CERT_PASSWORD, "/fd", "sha256", "/tr", $timestamp, "/td", "sha256")
        $signing = $true
    }
}

if (-not $signtool) {
    Write-Warning "No se encontro signtool.exe. Los ejecutables quedan sin firmar."
} elseif (-not $signing) {
    Write-Warning "Sin SIGN_CERT_THUMBPRINT o SIGN_CERT_PATH + SIGN_CERT_PASSWORD. Los ejecutables quedan sin firmar."
} else {
    foreach ($file in @("Smart Organizer.exe", "smart-organizer-cli.exe")) {
        $path = Join-Path $root "dist\$file"
        try {
            & $signtoolPath @signArgs $path
            if ($LASTEXITCODE -ne 0) {
                Write-Warning "No se pudo firmar $file (codigo $LASTEXITCODE). Se deja sin firmar."
            }
        } catch {
            Write-Warning "No se pudo firmar ${file}: $($_.Exception.Message). Se deja sin firmar."
        }
    }
}

# Hashes de todo lo que se publica, para comprobar la descarga.
$sums = Join-Path $root "dist\SHA256SUMS.txt"
$lines = Get-ChildItem (Join-Path $root "dist") -File |
    Where-Object { $_.Name -ne "SHA256SUMS.txt" } |
    Sort-Object Name |
    ForEach-Object {
        $hash = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $($_.Name)"
    }
[System.IO.File]::WriteAllLines($sums, $lines, [System.Text.UTF8Encoding]::new($false))

Get-ChildItem (Join-Path $root "dist") -File | Select-Object Name, Length, LastWriteTime
