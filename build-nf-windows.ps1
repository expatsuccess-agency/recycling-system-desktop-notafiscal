$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot
Write-Host "Construindo V8-NotaFiscal para Windows (NSIS)"
npm run build:web
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
npm run tauri:build -- --bundles nsis
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
