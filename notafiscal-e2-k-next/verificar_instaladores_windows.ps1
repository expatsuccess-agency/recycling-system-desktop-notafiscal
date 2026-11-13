param([string]$BundleDir = "src-tauri/target/release/bundle")
$ErrorActionPreference = "Stop"
if (-not (Test-Path $BundleDir)) { throw "Diretorio de pacotes ausente: $BundleDir" }
$packages = @(Get-ChildItem -Path $BundleDir -Recurse -File | Where-Object { $_.Extension -in @('.exe','.msi') })
if ($packages.Count -eq 0) { throw 'INSTALADOR_WINDOWS_NAO_ENCONTRADO' }
foreach ($pkg in $packages) {
 if ($pkg.Length -le 0) { throw "Pacote vazio: $($pkg.FullName)" }
 $hash = Get-FileHash -Algorithm SHA256 $pkg.FullName
 Write-Host "PACOTE=$($pkg.FullName) SHA256=$($hash.Hash) TAMANHO=$($pkg.Length)"
}
Write-Host 'PACOTES_WINDOWS_ENCONTRADOS. A instalacao e restauracao devem ser testadas em VM Windows isolada.'
