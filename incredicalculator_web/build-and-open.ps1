# Builds the web app, opens it in the default browser, and stops serving
# once the page has fetched the wasm module.
#
# Usage: .\dev.ps1 [-Port 8000]
param([int]$Port = 8000)

$web = $PSScriptRoot
$root = Split-Path $web -Parent

cargo build --target wasm32-unknown-unknown --manifest-path "$web\Cargo.toml"
if ($LASTEXITCODE) { exit $LASTEXITCODE }

# The wasm-bindgen CLI must match the wasm-bindgen crate version in Cargo.lock.
$lock = Select-String -Path "$root\Cargo.lock" -Pattern '^name = "wasm-bindgen"$' -Context 0, 1
$want = $lock.Context.PostContext[0] -replace '^version = "(.*)"$', '$1'
$have = 'none'
if (Get-Command wasm-bindgen -ErrorAction SilentlyContinue) {
    $have = (wasm-bindgen --version) -replace '^wasm-bindgen ', ''
}
if ($have -ne $want) {
    Write-Host "wasm-bindgen $want is required (found: $have). Install it with:"
    Write-Host "  cargo install --locked wasm-bindgen-cli --version $want"
    exit 1
}

wasm-bindgen --target web --out-dir "$web\pkg" "$root\target\wasm32-unknown-unknown\debug\incredicalculator_web.wasm"
if ($LASTEXITCODE) { exit $LASTEXITCODE }

# Set only after the native commands: in Windows PowerShell 5.1, redirected
# cargo warnings on stderr would otherwise abort the script.
$ErrorActionPreference = 'Stop'

$mime = @{
    '.html' = 'text/html; charset=utf-8'
    '.js'   = 'text/javascript'
    '.wasm' = 'application/wasm'
    '.css'  = 'text/css'
    '.json' = 'application/json'
}

$listener = [System.Net.HttpListener]::new()
$listener.Prefixes.Add("http://localhost:$Port/")
$listener.Start()
Write-Host "Serving http://localhost:$Port/ until the page loads..."
Start-Process "http://localhost:$Port/"

# Give up if the browser never fetches the wasm module.
$deadline = (Get-Date).AddSeconds(30)
try {
    while ($true) {
        $pending = $listener.GetContextAsync()
        # Poll so Ctrl+C and the deadline still work while waiting.
        while (-not $pending.Wait(250)) {
            if ((Get-Date) -gt $deadline) { throw 'Timed out waiting for the browser to load the page.' }
        }
        $ctx = $pending.Result
        $res = $ctx.Response

        $rel = [Uri]::UnescapeDataString($ctx.Request.Url.AbsolutePath).TrimStart('/')
        if (-not $rel) { $rel = 'index.html' }
        $file = [IO.Path]::GetFullPath((Join-Path $web $rel))

        if ($file.StartsWith($web, [StringComparison]::OrdinalIgnoreCase) -and (Test-Path $file -PathType Leaf)) {
            $bytes = [IO.File]::ReadAllBytes($file)
            $type = $mime[[IO.Path]::GetExtension($file)]
            if ($type) { $res.ContentType = $type }
            # Never cache, so every run picks up the fresh build.
            $res.Headers['Cache-Control'] = 'no-store'
            $res.ContentLength64 = $bytes.Length
            $res.OutputStream.Write($bytes, 0, $bytes.Length)
        } else {
            $res.StatusCode = 404
        }
        $res.Close()

        if ($file -like '*.wasm' -and $res.StatusCode -eq 200) { break }
    }
    Write-Host 'Page loaded; server stopped.'
} finally {
    $listener.Stop()
}
