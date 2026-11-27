# Builds the web app, opens it in the default browser, and keeps serving
# until Ctrl+C, so you can move between the home page, sim/ and editor/.
# build-and-open.ps1 is the same but stops serving once the page has loaded.
#
# Usage: .\build-and-serve.ps1 [-Port 8000] [-Page sim/]
param([int]$Port = 8000, [string]$Page = 'sim/')

& "$PSScriptRoot\build-and-open.ps1" -Port $Port -Page $Page -KeepServing
exit $LASTEXITCODE
