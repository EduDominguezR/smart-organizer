# Mide CPU y memoria de una compilacion en cuatro estados.
# Uso:
#   powershell -ExecutionPolicy Bypass -File .\scripts\measure.ps1 -Exe ".\dist\Smart Organizer.exe" -Label antes
#   powershell -ExecutionPolicy Bypass -File .\scripts\measure.ps1 -Exe "...\Smart Organizer.exe" -Label despues
#
# Usa una carpeta de datos portable y temporal: no toca la configuracion real.
# Cada estado se mide durante 60 segundos, con una muestra por segundo.

param(
    [Parameter(Mandatory = $true)]
    [string]$Exe,
    [string]$Label = "actual",
    [int]$Seconds = 60,
    [string]$Out
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $root

$source = (Resolve-Path $Exe).Path
if (-not $Out) {
    $Out = Join-Path $root "docs\rendimiento-$Label.md"
}
New-Item -ItemType Directory -Force (Split-Path -Parent $Out) | Out-Null

$work = Join-Path $env:TEMP "smart-organizer-measure"
$files = Join-Path $env:TEMP "smart-organizer-measure-files"
$gui = Join-Path $work "Smart Organizer.exe"

function Stop-Tree($processId) {
    if (-not $processId) { return }
    # `taskkill` devuelve 128 cuando el proceso ya termino por su cuenta. Ese
    # caso es normal al cerrar la GUI y no debe abortar la medicion.
    cmd /c "taskkill /PID $processId /T /F >nul 2>&1"
    Start-Sleep -Milliseconds 600
}

function Cleanup {
    Get-CimInstance Win32_Process -Filter "Name = 'Smart Organizer.exe'" -ErrorAction SilentlyContinue |
        Where-Object { $_.ExecutablePath -and $_.ExecutablePath.StartsWith($work, [StringComparison]::OrdinalIgnoreCase) } |
        ForEach-Object { Stop-Tree $_.ProcessId }
    Get-Process "Smart Organizer" -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -and $_.Path.StartsWith($work, [StringComparison]::OrdinalIgnoreCase) } |
        ForEach-Object { Stop-Tree $_.Id }
}

function Write-Config([bool]$watch) {
    $escaped = $files.Replace('\', '\\')
    $text = @"
version = 2
theme = "dark"
language = "es"
advanced = false
background = $($watch.ToString().ToLower())
start_with_windows = false
onboarded = true
window_size = [1180.0, 760.0]
window_pos = [-1.0, -1.0]

[[roots]]
path = "$escaped"
enabled = true
"@
    [System.IO.File]::WriteAllText((Join-Path $work "config.toml"), $text, [System.Text.UTF8Encoding]::new($false))
}

function Start-Gui([bool]$watch) {
    Write-Config $watch
    $process = Start-Process -FilePath $gui -WorkingDirectory $work -PassThru
    # Deja terminar el arranque, la carga del diario y el primer repintado.
    Start-Sleep -Seconds 15
    return $process
}

function Descendants([int]$parentId) {
    $ids = @($parentId)
    $pending = @($parentId)
    while ($pending.Count -gt 0) {
        $next = @()
        foreach ($id in $pending) {
            $children = Get-CimInstance Win32_Process -Filter "ParentProcessId = $id" -ErrorAction SilentlyContinue
            foreach ($child in $children) {
                if ($ids -notcontains [int]$child.ProcessId) {
                    $ids += [int]$child.ProcessId
                    $next += [int]$child.ProcessId
                }
            }
        }
        $pending = $next
    }
    return $ids
}

function Sample([int[]]$ids, [string]$name, [int]$duration) {
    $samples = @()
    $start = Get-Date
    $cpuStart = 0.0
    foreach ($id in $ids) {
        $p = Get-Process -Id $id -ErrorAction SilentlyContinue
        if ($p) { $cpuStart += $p.CPU }
    }
    for ($i = 0; $i -lt $duration; $i++) {
        Start-Sleep -Seconds 1
        $cpu = 0.0
        $ws = 0.0
        $alive = 0
        foreach ($id in $ids) {
            $p = Get-Process -Id $id -ErrorAction SilentlyContinue
            if ($p) {
                $cpu += $p.CPU
                $ws += $p.WorkingSet64
                $alive += 1
            }
        }
        $elapsed = ((Get-Date) - $start).TotalSeconds
        $percent = if ($elapsed -gt 0) { (($cpu - $cpuStart) / $elapsed) * 100.0 } else { 0.0 }
        $samples += [pscustomobject]@{
            Segundo = $i + 1
            CpuPct = [Math]::Round([Math]::Max(0, $percent), 3)
            MemoriaMB = [Math]::Round($ws / 1MB, 2)
            Procesos = $alive
        }
    }
    $avgCpu = ($samples | Measure-Object CpuPct -Average).Average
    $maxCpu = ($samples | Measure-Object CpuPct -Maximum).Maximum
    $avgMem = ($samples | Measure-Object MemoriaMB -Average).Average
    $maxMem = ($samples | Measure-Object MemoriaMB -Maximum).Maximum
    if (($samples | Measure-Object Procesos -Minimum).Minimum -lt 1) {
        throw "No quedaban procesos vivos durante '$name'. Se descarta la muestra."
    }
    return [pscustomobject]@{
        Estado = $name
        CpuMedia = [Math]::Round($avgCpu, 2)
        CpuMaxima = [Math]::Round($maxCpu, 2)
        MemoriaMediaMB = [Math]::Round($avgMem, 1)
        MemoriaMaximaMB = [Math]::Round($maxMem, 1)
        Muestras = $samples
    }
}

function Minimize([System.Diagnostics.Process]$process) {
    if (-not ("Win32Window" -as [type])) {
        Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class Win32Window {
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
}
"@
    }
    $process.Refresh()
    if ($process.MainWindowHandle -ne [IntPtr]::Zero) {
        [Win32Window]::ShowWindow($process.MainWindowHandle, 6) | Out-Null
    }
    Start-Sleep -Seconds 8
}

New-Item -ItemType Directory -Force $work | Out-Null
New-Item -ItemType Directory -Force (Join-Path $work "portable") | Out-Null
New-Item -ItemType Directory -Force $files | Out-Null
if (-not (Test-Path (Join-Path $files "documento.pdf"))) {
    $bytes = [byte[]]::new(96 * 1024)
    (New-Object Random 7).NextBytes($bytes)
    $names = @("factura.pdf", "notas.txt", "foto.png", "informe.docx", "datos.csv", "proyecto.zip", "video.mp4", "audio.mp3")
    for ($i = 0; $i -lt 80; $i++) {
        $name = "$($names[$i % $names.Count])-$i$(([io.path]::GetExtension($names[$i % $names.Count])))"
        [System.IO.File]::WriteAllBytes((Join-Path $files $name), $bytes)
    }
}
Cleanup
Copy-Item $source $gui -Force

$rows = @()
$raw = @()

# 1. Visible, sin vigilante.
$p = Start-Gui $false
$r = Sample @($p.Id) "GUI visible, en reposo" $Seconds
$rows += $r
$raw += $r.Muestras | ForEach-Object { $_ | Add-Member Estado "visible" -PassThru }
Stop-Tree $p.Id
Cleanup

# 2. Minimizada, sin vigilante.
$p = Start-Gui $false
Minimize $p
$r = Sample @($p.Id) "GUI minimizada" $Seconds
$rows += $r
$raw += $r.Muestras | ForEach-Object { $_ | Add-Member Estado "minimizada" -PassThru }
Stop-Tree $p.Id
Cleanup

# 3. En la bandeja con el vigilante en marcha: se suman ambos procesos.
# Se oculta la ventana directamente: el cierre real se prueba aparte y puede
# llegar a eframe como una peticion asincrona que no siempre se puede medir.
$p = Start-Gui $true
if (-not ("Win32Window" -as [type])) {
    Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class Win32Window {
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
}
"@
}
$p.Refresh()
if ($p.MainWindowHandle -ne [IntPtr]::Zero) {
    [Win32Window]::ShowWindow($p.MainWindowHandle, 0) | Out-Null
}
Start-Sleep -Seconds 10
$ids = Descendants $p.Id
if ($ids.Count -lt 2) {
    throw "La GUI y el vigilante no estaban vivos antes de medir la bandeja. Procesos: $($ids -join ', ')"
}
$r = Sample $ids "GUI en bandeja + vigilante" $Seconds
$rows += $r
$raw += $r.Muestras | ForEach-Object { $_ | Add-Member Estado "bandeja" -PassThru }
Stop-Tree $p.Id
Cleanup

# 4. Solo el proceso oculto --watch: sin ventana, sin OpenGL, sin fuentes.
Write-Config $true
$p = Start-Process -FilePath $gui -ArgumentList "--watch", "--hidden" -WorkingDirectory $work -PassThru
Start-Sleep -Seconds 10
$r = Sample @($p.Id) "Vigilante solo (--watch)" $Seconds
$rows += $r
$raw += $r.Muestras | ForEach-Object { $_ | Add-Member Estado "watch" -PassThru }
Stop-Tree $p.Id
Cleanup

$lines = @()
$lines += "# Recursos medidos: $Label"
$lines += ""
$lines += "Windows, $env:COMPUTERNAME. Una muestra por segundo durante $Seconds s."
$lines += "CPU en porcentaje de un nucleo; la memoria es el conjunto de trabajo."
$lines += ""
$lines += "| Estado | CPU media | CPU maxima | Memoria media | Memoria maxima |"
$lines += "| --- | ---: | ---: | ---: | ---: |"
foreach ($row in $rows) {
    $lines += "| $($row.Estado) | $($row.CpuMedia) % | $($row.CpuMaxima) % | $($row.MemoriaMediaMB) MB | $($row.MemoriaMaximaMB) MB |"
}
$lines += ""
$lines += "Ejecutable: ``$([io.path]::GetFileName($source))``, $((Get-Item $source).Length) bytes."
$lines += ""
$lines += "Datos por segundo en ``$([io.path]::GetFileNameWithoutExtension($Out))-muestras.csv``."
[System.IO.File]::WriteAllLines($Out, $lines, [System.Text.UTF8Encoding]::new($false))
$raw | Export-Csv (Join-Path (Split-Path -Parent $Out) "$([io.path]::GetFileNameWithoutExtension($Out))-muestras.csv") -NoTypeInformation -Encoding UTF8

$rows | Format-Table Estado, CpuMedia, CpuMaxima, MemoriaMediaMB, MemoriaMaximaMB -AutoSize
Write-Host "Guardado en $Out"
