# Builds the RP firmware (release) and reports flash and RAM usage against
# the limits in memory.x, plus the largest symbols in each.
#
# Usage, from anywhere:
#   powershell -File incredicalculator_rp/size-report.ps1
#   powershell -File incredicalculator_rp/size-report.ps1 -NoBuild -Top 20
#
# Needs llvm-size and llvm-nm on PATH (installed with LLVM).

param(
    [switch]$NoBuild,
    [int]$Top = 10
)

$ErrorActionPreference = 'Stop'
$crateDir = $PSScriptRoot
$repoRoot = Split-Path $crateDir -Parent
$target = 'thumbv8m.main-none-eabihf'
$elf = Join-Path $repoRoot "target/$target/release/incredicalculator_rp"

foreach ($tool in 'llvm-size', 'llvm-nm') {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        throw "$tool not found on PATH; install LLVM or add it to PATH."
    }
}

if (-not $NoBuild) {
    cargo build --release --manifest-path (Join-Path $crateDir 'Cargo.toml') --target $target
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

# --- Limits from memory.x -----------------------------------------------------
$memoryX = Get-Content (Join-Path $crateDir 'memory.x') -Raw
function Get-RegionLength([string]$name) {
    $m = [regex]::Match($memoryX, "(?m)^\s*$name\s*:\s*ORIGIN\s*=\s*\S+\s*,\s*LENGTH\s*=\s*(\d+)\s*([KM]?)")
    if (-not $m.Success) { throw "Region $name not found in memory.x" }
    $n = [int64]$m.Groups[1].Value
    switch ($m.Groups[2].Value) { 'K' { $n *= 1KB } 'M' { $n *= 1MB } }
    $n
}
$flashLimit = Get-RegionLength 'FLASH'
$ramLimit = Get-RegionLength 'RAM'

# --- Section sizes ------------------------------------------------------------
$flash = 0; $ram = 0
$ramSections = @{}
foreach ($line in & llvm-size -A $elf) {
    if ($line -notmatch '^(\.\S+)\s+(\d+)\s+(\d+)$') { continue }
    $name = $Matches[1]; $size = [int64]$Matches[2]; $addr = [int64]$Matches[3]
    if ($size -eq 0) { continue }
    if ($addr -ge 0x10000000 -and $addr -lt 0x20000000) { $flash += $size }
    elseif ($addr -ge 0x20000000 -and $addr -lt 0x20080000) {
        $ram += $size
        $ramSections[$name] = $size
        # .data is stored in flash and copied to RAM at boot.
        if ($name -eq '.data') { $flash += $size }
    }
}

function Format-KiB([int64]$bytes) { '{0,8:N1} KiB' -f ($bytes / 1KB) }
function Format-Row([string]$label, [int64]$used, [int64]$limit) {
    $pct = 100.0 * $used / $limit
    '{0,-32}{1} of{2}  ({3,5:N1}%)' -f $label, (Format-KiB $used), (Format-KiB $limit), $pct
}

Write-Host ''
Write-Host 'incredicalculator_rp memory usage' -ForegroundColor Cyan
Write-Host (Format-Row 'Flash (code + data)' $flash $flashLimit)
Write-Host (Format-Row 'RAM (static: data/bss/uninit)' $ram $ramLimit)
foreach ($s in $ramSections.GetEnumerator() | Sort-Object Value -Descending) {
    Write-Host ('    {0,-22}{1}' -f $s.Key, (Format-KiB $s.Value))
}
$stack = $ramLimit - $ram
$stackColor = if ($stack -lt 16KB) { 'Red' } elseif ($stack -lt 48KB) { 'Yellow' } else { 'Green' }
Write-Host ('{0,-32}{1}' -f 'Left for core 0 stack', (Format-KiB $stack)) -ForegroundColor $stackColor
Write-Host '  (heap and stack peaks at run time are logged as "memory: ..." over defmt)'

# --- Largest symbols ----------------------------------------------------------
$symbols = foreach ($line in & llvm-nm --print-size --size-sort --demangle $elf) {
    if ($line -match '^([0-9a-f]{8}) ([0-9a-f]{8}) (\S) (.+)$') {
        [pscustomobject]@{
            Addr = [Convert]::ToInt64($Matches[1], 16)
            Size = [Convert]::ToInt64($Matches[2], 16)
            Name = ($Matches[4] -replace '::h[0-9a-f]{16}$', '')
        }
    }
}
function Show-Largest([string]$title, $items) {
    Write-Host ''
    Write-Host $title -ForegroundColor Cyan
    foreach ($s in $items | Sort-Object Size -Descending | Select-Object -First $Top) {
        $name = if ($s.Name.Length -gt 90) { $s.Name.Substring(0, 87) + '...' } else { $s.Name }
        Write-Host ('{0}  {1}' -f (Format-KiB $s.Size), $name)
    }
}
Show-Largest "Largest RAM symbols" ($symbols | Where-Object { $_.Addr -ge 0x20000000 -and $_.Addr -lt 0x20080000 })
Show-Largest "Largest flash symbols" ($symbols | Where-Object { $_.Addr -ge 0x10000000 -and $_.Addr -lt 0x20000000 })
