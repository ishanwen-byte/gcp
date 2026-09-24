if ((Test-Path just_smoke.txt) -and ((Get-Content just_smoke.txt -Raw) -match 'Hello World')) {
    Remove-Item just_smoke.txt
    Write-Host "smoke OK"
    exit 0
} else {
    Write-Host "smoke FAILED"
    exit 1
}
