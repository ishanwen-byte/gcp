cargo build --release
Write-Host "Applying UPX compression..."
if (Get-Command upx -ErrorAction SilentlyContinue) {
    upx --best --lzma target/release/gcp.exe
    Write-Host "UPX compression applied successfully"
} else {
    Write-Host "UPX not found. Building without compression"
    Write-Host "Install UPX from: https://upx.github.io/"
}