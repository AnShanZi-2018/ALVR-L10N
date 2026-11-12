# Publishes this repository to GitHub in one go.
#
#   powershell -ExecutionPolicy Bypass -File publish.ps1 -Token ghp_xxx -Owner <your-github-name>
#
# The token needs: repo (create + push) and workflow (push .github/workflows).
# It is used only by this script and is removed from the git remote afterwards.
param(
    [Parameter(Mandatory = $true)][string]$Token,
    [Parameter(Mandatory = $true)][string]$Owner,
    [string]$Repo = "ALVR-L10N",
    [string]$Tag = "v1.0.0",
    [string]$Zip = "$PSScriptRoot\dist\ALVR-L10N-1.0.0-win64.zip",
    [string]$Proxy = "http://127.0.0.1:65532"
)

$ErrorActionPreference = "Stop"
$headers = @{
    Authorization          = "Bearer $Token"
    "User-Agent"           = "ALVR-L10N-publisher"
    Accept                 = "application/vnd.github+json"
}
$api = "https://api.github.com"

Write-Host "[1/4] checking repository $Owner/$Repo"
try {
    Invoke-RestMethod "$api/repos/$Owner/$Repo" -Headers $headers | Out-Null
    Write-Host "      already exists"
}
catch {
    Write-Host "      creating"
    $body = @{
        name        = $Repo
        description = "Standalone localization tool for ALVR (Chinese / Japanese / Korean)"
        private     = $false
        has_issues  = $true
        has_wiki    = $false
    } | ConvertTo-Json
    Invoke-RestMethod -Method Post "$api/user/repos" -Headers $headers -Body $body -ContentType "application/json" | Out-Null
}

Write-Host "[2/4] pushing source"
Push-Location $PSScriptRoot
try {
    git remote remove origin 2>$null
    git remote add origin "https://x-access-token:$Token@github.com/$Owner/$Repo.git"
    git -c http.proxy=$Proxy -c https.proxy=$Proxy push -u origin main
}
finally {
    git remote set-url origin "https://github.com/$Owner/$Repo.git"
    Pop-Location
}

Write-Host "[3/4] creating release $Tag"
$release = $null
try {
    $release = Invoke-RestMethod "$api/repos/$Owner/$Repo/releases/tags/$Tag" -Headers $headers
}
catch {
    $notes = @"
## ALVR-L10N 1.0.0

Standalone localization tool for ALVR — Simplified/Traditional Chinese, Japanese and Korean.

* Bundled 100% localized build for ALVR **v20.14.1** (Dashboard + Launcher, with in-app language switch)
* Compatibility mode for other ALVR versions (in-place patch, no rebuild required)
* Only the language packs you tick are installed into ALVR
* Tool UI in English / 简体中文 / 繁體中文 / 日本語 / 한국어
* ` + "`Add language pack…`" + ` for community packs, ` + "`Import config file…`" + ` for session.json
* Automatic backup of session.json and rollback if the patched build does not start

Unpack anywhere, run ` + "`ALVR-L10N.exe`" + `, tick languages, press **Apply**.
"@
    $body = @{ tag_name = $Tag; name = "ALVR-L10N 1.0.0"; body = $notes; draft = $false; prerelease = $false } | ConvertTo-Json
    $release = Invoke-RestMethod -Method Post "$api/repos/$Owner/$Repo/releases" -Headers $headers -Body $body -ContentType "application/json"
}

Write-Host "[4/4] uploading asset"
if (Test-Path $Zip) {
    $name = Split-Path $Zip -Leaf
    $upload = "$api/repos/$Owner/$Repo/releases/$($release.id)/assets?name=$name"
    Invoke-RestMethod -Method Post $upload -Headers $headers -InFile $Zip -ContentType "application/zip" | Out-Null
    Write-Host "      https://github.com/$Owner/$Repo/releases/tag/$Tag"
}
else {
    Write-Host "      asset not found: $Zip"
}

Write-Host "done: https://github.com/$Owner/$Repo"
