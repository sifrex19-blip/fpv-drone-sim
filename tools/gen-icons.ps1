# Generates sidebar icons + controller thumbnails as PNGs (no external deps).
param([string]$OutDir = "$PSScriptRoot\..\assets")

Add-Type -AssemblyName System.Drawing

function New-Surface([int]$w, [int]$h) {
    $bmp = New-Object System.Drawing.Bitmap($w, $h, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $g.Clear([System.Drawing.Color]::Transparent)
    return @($bmp, $g)
}

function Save-Surface($bmp, $g, [string]$path) {
    $g.Dispose()
    $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bmp.Dispose()
}

function Get-Color([string]$hex) {
    return [System.Drawing.ColorTranslator]::FromHtml($hex)
}

$White = Get-Color '#FFFFFF'
$Purple = Get-Color '#8B5CF6'
$Dark = Get-Color '#1C1C26'
$Mid = Get-Color '#3A3A4C'
$Light = Get-Color '#C8C8D8'
$Screen = Get-Color '#4C3FA8'

$iconDir = Join-Path $OutDir 'ui'
New-Item -ItemType Directory -Force -Path $iconDir | Out-Null
$ctrlDir = Join-Path $OutDir 'controllers'
New-Item -ItemType Directory -Force -Path $ctrlDir | Out-Null

# ---------- nav icons (64x64, white glyphs) ----------

function New-HomeIcon([string]$path) {
    $s = New-Surface 64 64; $bmp, $g = $s
    $pts = @(
        (New-Object System.Drawing.PointF(32, 6)),
        (New-Object System.Drawing.PointF(60, 30)),
        (New-Object System.Drawing.PointF(51, 30)),
        (New-Object System.Drawing.PointF(51, 56)),
        (New-Object System.Drawing.PointF(13, 56)),
        (New-Object System.Drawing.PointF(13, 30)),
        (New-Object System.Drawing.PointF(4, 30))
    )
    $g.FillPolygon((New-Object System.Drawing.SolidBrush($White)), $pts)
    # erase door
    $g.CompositingMode = [System.Drawing.Drawing2D.CompositingMode]::SourceCopy
    $g.FillRectangle((New-Object System.Drawing.SolidBrush([System.Drawing.Color]::Transparent)), 26, 38, 12, 18)
    Save-Surface $bmp $g $path
}

function New-ControllerIcon([string]$path) {
    $s = New-Surface 64 64; $bmp, $g = $s
    $body = New-Object System.Drawing.Rectangle(6, 18, 52, 30)
    $g.FillRoundedRectangle((New-Object System.Drawing.SolidBrush($White)), $body, 13)
    $g.CompositingMode = [System.Drawing.Drawing2D.CompositingMode]::SourceCopy
    $t = [System.Drawing.Color]::Transparent
    # dpad cross (left)
    $g.FillRectangle((New-Object System.Drawing.SolidBrush($t)), 18, 27, 14, 5)
    $g.FillRectangle((New-Object System.Drawing.SolidBrush($t)), 22, 23, 6, 14)
    # buttons (right)
    $g.FillEllipse((New-Object System.Drawing.SolidBrush($t)), 40, 24, 8, 8)
    $g.FillEllipse((New-Object System.Drawing.SolidBrush($t)), 46, 33, 8, 8)
    Save-Surface $bmp $g $path
}

function New-SettingsIcon([string]$path) {
    $s = New-Surface 64 64; $bmp, $g = $s
    $g.TranslateTransform(32, 32)
    $pen = New-Object System.Drawing.Pen($White, 7)
    $g.DrawEllipse($pen, -17, -17, 34, 34)
    for ($i = 0; $i -lt 8; $i++) {
        $g.RotateTransform(45)
        $g.FillRectangle((New-Object System.Drawing.SolidBrush($White)), -4, -29, 8, 12)
    }
    Save-Surface $bmp $g $path
}

function New-AddonsIcon([string]$path) {
    $s = New-Surface 64 64; $bmp, $g = $s
    $body = New-Object System.Drawing.Rectangle(10, 10, 44, 44)
    $g.FillRoundedRectangle((New-Object System.Drawing.SolidBrush($White)), $body, 8)
    # bump on right
    $g.FillEllipse((New-Object System.Drawing.SolidBrush($White)), 46, 24, 16, 16)
    # hole on left
    $g.CompositingMode = [System.Drawing.Drawing2D.CompositingMode]::SourceCopy
    $g.FillEllipse((New-Object System.Drawing.SolidBrush([System.Drawing.Color]::Transparent)), 2, 24, 16, 16)
    Save-Surface $bmp $g $path
}

function New-StatsIcon([string]$path) {
    $s = New-Surface 64 64; $bmp, $g = $s
    $brush = New-Object System.Drawing.SolidBrush($White)
    $g.FillRoundedRectangle($brush, (New-Object System.Drawing.Rectangle(10, 34, 11, 22)), 4)
    $g.FillRoundedRectangle($brush, (New-Object System.Drawing.Rectangle(27, 20, 11, 36)), 4)
    $g.FillRoundedRectangle($brush, (New-Object System.Drawing.Rectangle(44, 8, 11, 48)), 4)
    Save-Surface $bmp $g $path
}

function New-AboutIcon([string]$path) {
    $s = New-Surface 64 64; $bmp, $g = $s
    $pen = New-Object System.Drawing.Pen($White, 6)
    $g.DrawEllipse($pen, 7, 7, 50, 50)
    $brush = New-Object System.Drawing.SolidBrush($White)
    $g.FillEllipse($brush, 28, 17, 8, 8)
    $g.FillRoundedRectangle($brush, (New-Object System.Drawing.Rectangle(28, 30, 8, 19)), 3)
    Save-Surface $bmp $g $path
}

function New-QuitIcon([string]$path) {
    $s = New-Surface 64 64; $bmp, $g = $s
    $pen = New-Object System.Drawing.Pen($White, 7)
    $pen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $pen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $g.DrawArc($pen, 12, 12, 40, 40, -50, 280)
    $g.DrawLine($pen, 32, 4, 32, 30)
    Save-Surface $bmp $g $path
}

function New-SearchIcon([string]$path) {
    $s = New-Surface 64 64; $bmp, $g = $s
    $pen = New-Object System.Drawing.Pen($White, 7)
    $pen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $pen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $g.DrawEllipse($pen, 9, 9, 34, 34)
    $g.DrawLine($pen, 40, 40, 56, 56)
    Save-Surface $bmp $g $path
}

function New-Logo([string]$path) {
    $s = New-Surface 160 160; $bmp, $g = $s
    $pen = New-Object System.Drawing.Pen($Purple, 9)
    $pen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $pen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $c = 80
    $arms = @(@(-46, -46), @(46, -46), @(-46, 46), @(46, 46))
    foreach ($a in $arms) {
        $g.DrawLine($pen, $c, $c, ($c + $a[0]), ($c + $a[1]))
    }
    foreach ($a in $arms) {
        $x = $c + $a[0]; $y = $c + $a[1]
        $g.DrawEllipse($pen, ($x - 20), ($y - 20), 40, 40)
    }
    $brush = New-Object System.Drawing.SolidBrush($Purple)
    $g.FillRoundedRectangle($brush, (New-Object System.Drawing.Rectangle(62, 62, 36, 36)), 9)
    Save-Surface $bmp $g $path
}

# ---------- controller thumbnails (240x150) ----------

function New-RadioThumb([string]$path) {
    $s = New-Surface 240 150; $bmp, $g = $s
    $brush = New-Object System.Drawing.SolidBrush($Mid)
    $g.FillRoundedRectangle($brush, (New-Object System.Drawing.Rectangle(40, 30, 160, 110)), 14)
    # antenna
    $g.FillRectangle($brush, 170, 4, 10, 34)
    # screen
    $g.FillRoundedRectangle((New-Object System.Drawing.SolidBrush($Screen)), (New-Object System.Drawing.Rectangle(92, 96, 56, 34)), 5)
    # gimbals
    $pen = New-Object System.Drawing.Pen($Light, 4)
    $g.DrawEllipse($pen, 56, 50, 34, 34)
    $g.DrawEllipse($pen, 150, 50, 34, 34)
    $g.FillEllipse((New-Object System.Drawing.SolidBrush($Light)), 68, 62, 10, 10)
    $g.FillEllipse((New-Object System.Drawing.SolidBrush($Light)), 162, 62, 10, 10)
    # center badge
    $g.FillRoundedRectangle((New-Object System.Drawing.SolidBrush($Purple)), (New-Object System.Drawing.Rectangle(104, 44, 32, 18)), 5)
    Save-Surface $bmp $g $path
}

function New-GamepadThumb([string]$path) {
    $s = New-Surface 240 150; $bmp, $g = $s
    $brush = New-Object System.Drawing.SolidBrush($Mid)
    $g.FillRoundedRectangle($brush, (New-Object System.Drawing.Rectangle(30, 40, 180, 84)), 34)
    $pen = New-Object System.Drawing.Pen($Light, 5)
    # dpad
    $g.DrawLine($pen, 66, 68, 66, 96); $g.DrawLine($pen, 52, 82, 80, 82)
    # buttons
    $b = New-Object System.Drawing.SolidBrush($Light)
    $g.FillEllipse($b, 156, 60, 14, 14)
    $g.FillEllipse($b, 174, 78, 14, 14)
    $g.FillEllipse($b, 156, 96, 14, 14)
    $g.FillEllipse($b, 138, 78, 14, 14)
    # sticks
    $g.DrawEllipse($pen, 92, 64, 26, 26)
    $g.DrawEllipse($pen, 136, 64, 26, 26)
    Save-Surface $bmp $g $path
}

function New-JoystickThumb([string]$path) {
    $s = New-Surface 240 150; $bmp, $g = $s
    $brush = New-Object System.Drawing.SolidBrush($Mid)
    $g.FillRoundedRectangle($brush, (New-Object System.Drawing.Rectangle(70, 100, 100, 34)), 12)
    $pen = New-Object System.Drawing.Pen($Light, 7)
    $g.DrawLine($pen, 120, 104, 120, 52)
    $g.FillEllipse((New-Object System.Drawing.SolidBrush($Purple)), 104, 30, 32, 32)
    $b = New-Object System.Drawing.SolidBrush($Light)
    $g.FillEllipse($b, 146, 112, 12, 12)
    $g.FillEllipse($b, 84, 112, 12, 12)
    Save-Surface $bmp $g $path
}

function New-ThrottleThumb([string]$path) {
    $s = New-Surface 240 150; $bmp, $g = $s
    $brush = New-Object System.Drawing.SolidBrush($Mid)
    $g.FillRoundedRectangle($brush, (New-Object System.Drawing.Rectangle(60, 40, 60, 90)), 12)
    $g.FillRoundedRectangle($brush, (New-Object System.Drawing.Rectangle(130, 60, 50, 70)), 12)
    $pen = New-Object System.Drawing.Pen($Purple, 6)
    $g.DrawLine($pen, 78, 56, 102, 56)
    $g.FillRoundedRectangle((New-Object System.Drawing.SolidBrush($Purple)), (New-Object System.Drawing.Rectangle(142, 74, 26, 30)), 6)
    Save-Surface $bmp $g $path
}

function New-YokeThumb([string]$path) {
    $s = New-Surface 240 150; $bmp, $g = $s
    $pen = New-Object System.Drawing.Pen($Light, 8)
    $g.DrawEllipse($pen, 70, 30, 100, 100)
    $g.FillRoundedRectangle((New-Object System.Drawing.SolidBrush($Mid)), (New-Object System.Drawing.Rectangle(60, 72, 120, 18)), 8)
    $brush = New-Object System.Drawing.SolidBrush($Purple)
    $g.FillEllipse($brush, 112, 66, 16, 16)
    Save-Surface $bmp $g $path
}

function New-RudderThumb([string]$path) {
    $s = New-Surface 240 150; $bmp, $g = $s
    $brush = New-Object System.Drawing.SolidBrush($Mid)
    $g.FillRoundedRectangle($brush, (New-Object System.Drawing.Rectangle(50, 60, 60, 60)), 10)
    $g.FillRoundedRectangle($brush, (New-Object System.Drawing.Rectangle(130, 60, 60, 60)), 10)
    $pen = New-Object System.Drawing.Pen($Purple, 6)
    $g.DrawLine($pen, 80, 60, 80, 30)
    $g.DrawLine($pen, 160, 60, 160, 30)
    Save-Surface $bmp $g $path
}

function New-VrThumb([string]$path) {
    $s = New-Surface 240 150; $bmp, $g = $s
    $brush = New-Object System.Drawing.SolidBrush($Mid)
    $g.FillRoundedRectangle($brush, (New-Object System.Drawing.Rectangle(50, 45, 140, 64)), 24)
    $g.CompositingMode = [System.Drawing.Drawing2D.CompositingMode]::SourceCopy
    $t = [System.Drawing.Color]::Transparent
    $g.FillEllipse((New-Object System.Drawing.SolidBrush($t)), 74, 60, 40, 40)
    $g.FillEllipse((New-Object System.Drawing.SolidBrush($t)), 126, 60, 40, 40)
    Save-Surface $bmp $g $path
}

# ---- helper: rounded rectangle (PowerShell has no built-in) ----
Update-TypeData -TypeName System.Drawing.Graphics -MemberName FillRoundedRectangle -MemberType ScriptMethod -Value {
    param($brush, $rect, $radius)
    $d = $radius * 2
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddArc($rect.X, $rect.Y, $d, $d, 180, 90)
    $path.AddArc($rect.X + $rect.Width - $d, $rect.Y, $d, $d, 270, 90)
    $path.AddArc($rect.X + $rect.Width - $d, $rect.Y + $rect.Height - $d, $d, $d, 0, 90)
    $path.AddArc($rect.X, $rect.Y + $rect.Height - $d, $d, $d, 90, 90)
    $path.CloseFigure()
    $this.FillPath($brush, $path)
    $path.Dispose()
} -ErrorAction SilentlyContinue

# ---------- generate ----------

New-HomeIcon       (Join-Path $iconDir 'home.png')
New-ControllerIcon (Join-Path $iconDir 'controller.png')
New-SettingsIcon   (Join-Path $iconDir 'settings.png')
New-AddonsIcon     (Join-Path $iconDir 'addons.png')
New-StatsIcon      (Join-Path $iconDir 'stats.png')
New-AboutIcon      (Join-Path $iconDir 'about.png')
New-QuitIcon       (Join-Path $iconDir 'quit.png')
New-Logo           (Join-Path $iconDir 'logo.png')
New-SearchIcon     (Join-Path $iconDir 'search.png')

New-RadioThumb     (Join-Path $ctrlDir '_kind_Radio.png')
New-GamepadThumb   (Join-Path $ctrlDir '_kind_Gamepad.png')
New-JoystickThumb  (Join-Path $ctrlDir '_kind_Joystick.png')
New-ThrottleThumb  (Join-Path $ctrlDir '_kind_Throttle.png')
New-YokeThumb      (Join-Path $ctrlDir '_kind_Yoke.png')
New-RudderThumb    (Join-Path $ctrlDir '_kind_Rudder.png')
New-VrThumb        (Join-Path $ctrlDir '_kind_VR.png')

Write-Host "Icons written to $iconDir and $ctrlDir"
Get-ChildItem $iconDir, $ctrlDir -Filter *.png | Select-Object Name, Length
