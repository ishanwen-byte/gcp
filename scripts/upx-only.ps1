Write-Host "Applying UPX compression..."
if (Get-Command upx -ErrorAction SilentlyContinue) {
    if (Test-Path target/release/gcp.exe) {
        upx --best --lzma target/release/gcp.exe
        Write-Host "UPX compression applied successfully"
    } else {
        Write-Host "Binary not found. Run just build first."
    }
} else {
    Write-Host "UPX not found. Install UPX for compression"
    Write-Host "Download from: https://upx.github.io/"
}