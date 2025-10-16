if (Test-Path target/release/gcp.exe) {
    $size = (Get-Item target/release/gcp.exe).Length
    Write-Host "Current binary: $([math]::Round($size/1KB, 2)) KB"
    if (Get-Command upx -ErrorAction SilentlyContinue) {
        Write-Host "UPX is available for compression"
    } else {
        Write-Host "UPX not available - install for smaller size"
    }
} else {
    Write-Host "Binary not found. Run just build first."
}