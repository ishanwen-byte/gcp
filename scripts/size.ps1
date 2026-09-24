param()
if (Test-Path target/release/gcp.exe) {
    $s = (Get-Item target/release/gcp.exe).Length
    Write-Host ("binary: " + [math]::Round($s/1KB) + " KB (" + $s + " bytes)")
} else {
    Write-Host "binary not found; run just build"
}
