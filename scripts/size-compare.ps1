Write-Host "=== Detailed Size Comparison ==="
if (Test-Path target/release/gcp.exe) {
    Copy-Item target/release/gcp.exe target/release/gcp_backup.exe -Force
    try {
        upx -t target/release/gcp.exe 2>$null
        Write-Host "Binary is already UPX compressed"
        try {
            upx -d target/release/gcp.exe 2>$null
        } catch { }
        $compressedSize = (Get-Item target/release/gcp_backup.exe).Length
        $uncompressedSize = (Get-Item target/release/gcp.exe).Length
        Write-Host "Compressed: $([math]::Round($compressedSize/1KB, 2)) KB"
        Write-Host "Uncompressed: $([math]::Round($uncompressedSize/1KB, 2)) KB"
        $savings = [math]::Round((($uncompressedSize-$compressedSize)/$uncompressedSize*100), 1)
        Write-Host "Size reduction: $savings%"
        Move-Item target/release/gcp_backup.exe target/release/gcp.exe -Force
    } catch {
        Write-Host "Binary is not compressed"
        $uncompressedSize = (Get-Item target/release/gcp.exe).Length
        Write-Host "Uncompressed: $([math]::Round($uncompressedSize/1KB, 2)) KB"
        if (Get-Command upx -ErrorAction SilentlyContinue) {
            Write-Host "Compressing for comparison..."
            upx --best --lzma target/release/gcp.exe
            $compressedSize = (Get-Item target/release/gcp.exe).Length
            Write-Host "Compressed: $([math]::Round($compressedSize/1KB, 2)) KB"
            $savings = [math]::Round((($uncompressedSize-$compressedSize)/$uncompressedSize*100), 1)
            Write-Host "Size reduction: $savings%"
        } else {
            Write-Host "Install UPX to see compression benefits"
        }
    }
    Remove-Item target/release/gcp_backup.exe -Force -ErrorAction SilentlyContinue
} else {
    Write-Host "Binary not found. Run just build first."
}