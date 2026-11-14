# Starts a second BejaClient launcher with its own data (accounts, settings, profiles, WebView2 profile), so you can log in
# with another Minecraft account and test party/friend features side by side with the normal launcher.
#
#   npm run instance2            -> instance "2"
#   npm run instance2 -- friend  -> instance "friend"
#
# Needs the dev server (npm run dev) and a debug build (npm run tauri:dev once). Data lives in
# %APPDATA%\BejaClient\launcher-instances\<name>; the big caches (beja-libs, runtimes) are shared.
param([string]$Name = '2')

$root = Split-Path -Parent $PSScriptRoot
$src = Join-Path $root 'src-tauri\target\debug\app.exe'
if (-not (Test-Path $src)) { Write-Error "No debug build at $src - run 'npm run tauri:dev' once first."; exit 1 }

# run a copy so the normal dev build can still be rebuilt while this instance is open
$exe = Join-Path (Split-Path $src) "app-instance-$Name.exe"
Copy-Item $src $exe -Force

$env:BEJA_INSTANCE = $Name
Start-Process -FilePath $exe -WorkingDirectory (Split-Path $src)
Write-Host "BejaClient instance '$Name' started."
