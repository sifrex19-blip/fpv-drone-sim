# Fetches real, freely-licensed product images for the controller database.
# Sources: Wikimedia Commons first, Openverse (CC-only) as fallback.
# Output: assets/controllers/<slug>.png (240x180, letterboxed) + CREDITS.md
param(
  [Parameter(Mandatory = $true)][string]$Repo
)
$ErrorActionPreference = "Continue"
$UA = "fpv-drone-sim-asset-harvester/0.6 (github.com/sifrex19-blip/fpv-drone-sim)"
$dir = Join-Path $Repo "assets\controllers"
New-Item -ItemType Directory -Force $dir | Out-Null
Add-Type -AssemblyName System.Drawing

# ---- 1. parse the database from controllers.rs (single source of truth) ----
$src = [System.IO.File]::ReadAllText((Join-Path $Repo "src\controllers.rs"), [System.Text.Encoding]::UTF8)
$entries = @()
foreach ($m in [regex]::Matches($src, 'add!\("([^"]+)",\s*"([^"]+)",\s*"([^"]+)"\)')) {
  $entries += [pscustomobject]@{ Name = $m.Groups[1].Value; Vendor = $m.Groups[2].Value; Kind = $m.Groups[3].Value }
}
Write-Output "entries: $($entries.Count)"

function Get-Slug([string]$name) {
  ($name.ToLowerInvariant() -replace '[^a-z0-9]+', '_' -replace '^_+|_+$', '')
}

# tokens that carry no product identity
$generic = @(
  'wireless','controller','gamepad','joystick','transmitter','radio','remote','control','controls','pad',
  'pro','plus','edition','series','flight','simulator','sim','throttle','yoke','pedals','rudder','hotas',
  'vr','edge','elite','usb','game','games','digital','channel','ghz','mhz','handheld','black','white',
  'red','blue','green','gold','silver','official','original','new','se','lite','air','stick','side','desk',
  'mode','modes','ultimate','enhanced','fighting','commander','split','instinct','revolution','afterglow',
  'stratus','recon','pedal','system','console','windows','android','ios','pc','box','set','with','and','the',
  'v2','v3','v4','v5','3d','extreme','one','series','sx','s','x','l','r','mk','ii','iii','iv','vi','vii',
  'plus','deluxe','professional','mini','small','large','black','carbon','special','limited','bluetooth',
  'wired','cable','charge','charging','adapter','edition','generation','frame','drone','fpv','rc'
)
$nouns = @('controller','gamepad','joystick','throttle','yoke','rudder','pedals','hotas','transmitter','radio','vr')
$badTokens = @('logo','icon','icons','diagram','screenshot','advertisement','patent','poster','map','chart')

function Get-Tokens([string]$s) {
  @(($s.ToLowerInvariant() -replace '\.[a-z]{2,4}$', '' -replace '[^a-z0-9]+', ' ').Split(' ') | Where-Object { $_ })
}
function Test-Gate([string[]]$entryTokens, [string[]]$keyTokens, [string]$title) {
  $t = Get-Tokens $title
  if ($t | Where-Object { $badTokens -contains $_ }) { return $false }
  foreach ($k in $keyTokens) {
    $hit = ($t -contains $k) -or ($k.Length -ge 4 -and $title.ToLowerInvariant().Contains($k))
    if (-not $hit) { return $false }
  }
  $entryNouns = $entryTokens | Where-Object { $nouns -contains $_ }
  if ($entryNouns) {
    $hasNoun = $t | Where-Object { $entryNouns -contains $_ }
    if (-not $hasNoun) { return $false }
  }
  return $true
}

$credits = New-Object System.Collections.Generic.List[object]
$found = 0; $missed = 0

foreach ($e in $entries) {
  $slug = Get-Slug $e.Name
  $out = Join-Path $dir "$slug.png"
  if (Test-Path $out) { $found++; continue }

  $entryTokens = Get-Tokens $e.Name
  $keyTokens = @($entryTokens | Where-Object { $_.Length -ge 3 -and ($generic -notcontains $_) })
  if (-not $keyTokens) { $missed++; continue }   # generic product -> illustration fallback

  $picked = $null   # @{ url; title; creator; license; page }

  # ---- 2a. Wikimedia Commons ----
  try {
    $u = "https://commons.wikimedia.org/w/api.php?action=query&format=json&generator=search" +
         "&gsrsearch=$([uri]::EscapeDataString('filetype:bitmap ' + $e.Name))&gsrnamespace=6&gsrlimit=10" +
         "&prop=imageinfo&iiprop=url|extmetadata&iiurlwidth=640"
    $r = Invoke-RestMethod -Uri $u -Headers @{ 'User-Agent' = $UA } -TimeoutSec 30
    if ($r.query.pages) {
      $pages = $r.query.pages.PSObject.Properties.Value | Sort-Object { $_.index }
      foreach ($p in $pages) {
        $title = $p.title -replace '^File:', ''
        if (Test-Gate $entryTokens $keyTokens $title) {
          $m = $p.imageinfo[0].extmetadata
          $artist = ($m.Artist.value -replace '<[^>]+>', '' -replace '\s+', ' ').Trim()
          $underscored = $p.title.Replace(' ', '_')
          $encoded = [uri]::EscapeDataString($underscored)
          $picked = @{
            url     = $p.imageinfo[0].thumburl
            title   = $title
            creator = $artist
            license = $m.LicenseShortName.value
            page    = "https://commons.wikimedia.org/wiki/$encoded"
          }
          break
        }
      }
    }
  } catch { Write-Output "  commons error [$($e.Name)]: $($_.Exception.Message)" }
  Start-Sleep -Milliseconds 1300

  # ---- 2b. Openverse fallback (free licenses only) ----
  if (-not $picked) {
    try {
      $ovq = [uri]::EscapeDataString($e.Name)
      $u = "https://api.openverse.org/v1/images/?q=$ovq" +
           "&license=cc0,pdm,by,by-sa&page_size=12"
      $r = Invoke-RestMethod -Uri $u -Headers @{ 'User-Agent' = $UA } -TimeoutSec 12
      foreach ($res in $r.results) {
        if (Test-Gate $entryTokens $keyTokens $res.title) {
          $picked = @{
            url     = $res.url
            title   = $res.title
            creator = $res.creator
            license = $res.license
            page    = $res.foreign_landing_url
          }
          break
        }
      }
    } catch { Write-Output "  openverse error [$($e.Name)]: $($_.Exception.Message)" }
    Start-Sleep -Milliseconds 1200
  }

  if (-not $picked) { $missed++; continue }

  # ---- 3. download + normalize to 240x180 letterboxed PNG ----
  $tmp = Join-Path $env:TEMP "fpv_img_$slug`_dl"
  try {
    Invoke-WebRequest -Uri $picked.url -OutFile $tmp -Headers @{ 'User-Agent' = $UA } -TimeoutSec 45
    $img = [System.Drawing.Image]::FromFile($tmp)
    $canvas = New-Object System.Drawing.Bitmap 240, 180
    $g = [System.Drawing.Graphics]::FromImage($canvas)
    $g.Clear([System.Drawing.Color]::FromArgb(255, 23, 23, 31))
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $scale = [Math]::Min(224.0 / $img.Width, 164.0 / $img.Height)
    $w = [int]($img.Width * $scale); $h = [int]($img.Height * $scale)
    $g.DrawImage($img, [int]((240 - $w) / 2), [int]((180 - $h) / 2), $w, $h)
    $canvas.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose(); $canvas.Dispose(); $img.Dispose()
    Remove-Item $tmp -Force -ErrorAction SilentlyContinue
    $credits.Add([pscustomobject]@{
        Entry = $e.Name; File = "$slug.png"; Title = $picked.title
        Creator = $picked.creator; License = $picked.license; Source = $picked.page
      })
    $found++
    Write-Output "OK  $($e.Name)  <-  $($picked.title)"
  } catch {
    $missed++
    Write-Output "DL FAIL [$($e.Name)]: $($_.Exception.Message)"
    Remove-Item $tmp, $out -Force -ErrorAction SilentlyContinue
  }
}

# ---- 4. credits ----
$sb = New-Object System.Text.StringBuilder
[void]$sb.AppendLine("# Controller image credits")
[void]$sb.AppendLine()
[void]$sb.AppendLine("Images fetched by `tools/fetch-controller-images.ps1` from Wikimedia Commons")
[void]$sb.AppendLine("and Openverse. All images are freely licensed (CC0 / public domain / CC BY / CC BY-SA).")
[void]$sb.AppendLine("Entries without a listed image use the generated kind illustrations (`_kind_*.png`).")
[void]$sb.AppendLine()
[void]$sb.AppendLine("| Entry | Image | Title | Author | License | Source |")
[void]$sb.AppendLine("|---|---|---|---|---|---|")
foreach ($c in $credits) {
  $esc = { param($s) ($s -replace '\|', '\|') }
  [void]$sb.AppendLine("| $($c.Entry) | $($c.File) | $($c.Title) | $($c.Creator) | $($c.License) | $($c.Source) |")
}
[System.IO.File]::WriteAllText((Join-Path $dir 'CREDITS.md'), $sb.ToString(), (New-Object System.Text.UTF8Encoding $false))

Write-Output "DONE found=$found missed=$missed of $($entries.Count)"
