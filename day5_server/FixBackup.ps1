param(
    [Parameter()]
    [String]$ip
)

Write-Host $ip

$backupfile = 'AC2023.BAK'

$backupfilepath = "C:\DosDrive\$backupfile"

$backupfilebytes = Get-Content $backupfilepath -Encoding byte
$backupfileBytesBase64 = [System.Convert]::ToBase64String($backupfilebytes)

$url = "http://$($ip):8080/fix"

$body = @{
    "filename" = $backupfile
    "b64_data" = $backupfileBytesBase64
}

$res = Invoke-WebRequest -Method 'Post' -Uri $url -Body ($body|ConvertTo-Json) -ContentType "application/json"

$fixedJson = $res.Content | Out-String | ConvertFrom-Json

[System.Convert]::FromBase64String($fixedJson.b64_data) | Set-Content $backupfilepath -Encoding Byte