Add-Type -AssemblyName System.Drawing

$size = 1024
$bmp = New-Object System.Drawing.Bitmap($size, $size)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = 'HighQuality'
$g.Clear([System.Drawing.Color]::Transparent)

# Round rect background (Bilibili pink #FB7299)
$bgColor = [System.Drawing.Color]::FromArgb(255, 251, 114, 153)
$margin = 20
$r = 180
$bgBrush = New-Object System.Drawing.SolidBrush($bgColor)
$bgPath = New-Object System.Drawing.Drawing2D.GraphicsPath
$bgPath.AddArc($margin, $margin, $r, $r, 180, 90)
$bgPath.AddArc($size - $margin - $r, $margin, $r, $r, 270, 90)
$bgPath.AddArc($size - $margin - $r, $size - $margin - $r, $r, $r, 0, 90)
$bgPath.AddArc($margin, $size - $margin - $r, $r, $r, 90, 90)
$bgPath.CloseFigure()
$g.FillPath($bgBrush, $bgPath)

# Inner glow
$glowColor = [System.Drawing.Color]::FromArgb(35, 255, 255, 255)
$glowBrush = New-Object System.Drawing.SolidBrush($glowColor)
$glowPath = New-Object System.Drawing.Drawing2D.GraphicsPath
$gx = 50; $gy = 50; $gw = $size - 100; $gh = 300; $gr = 140
$glowPath.AddArc($gx, $gy, $gr, $gr, 180, 90)
$glowPath.AddArc($gx + $gw - $gr, $gy, $gr, $gr, 270, 90)
$glowPath.AddArc($gx + $gw - $gr, $gy + $gh - $gr, $gr, $gr, 0, 90)
$glowPath.AddArc($gx, $gy + $gh - $gr, $gr, $gr, 90, 90)
$glowPath.CloseFigure()
$g.FillPath($glowBrush, $glowPath)

# Download arrow
$penWidth = $size * 0.06
$arrowPen = New-Object System.Drawing.Pen([System.Drawing.Color]::White, $penWidth)
$arrowPen.StartCap = 'Round'
$arrowPen.EndCap = 'Round'
$arrowPen.LineJoin = 'Round'

$cx = $size / 2
$cy = $size / 2
$topY = $size * 0.20
$midY = $size * 0.58
$tipY = $size * 0.70
$wing = $size * 0.15

# Vertical shaft
$g.DrawLine($arrowPen, $cx, $topY, $cx, $midY)

# Arrow head (V)
$g.DrawLine($arrowPen, $cx - $wing, $midY - 10, $cx, $tipY)
$g.DrawLine($arrowPen, $cx + $wing, $midY - 10, $cx, $tipY)

# Horizontal tray
$trayY = $size * 0.77
$trayW = $size * 0.28
$g.DrawLine($arrowPen, $cx - $trayW/2, $trayY, $cx + $trayW/2, $trayY)

# Top dot
$dotR = $size * 0.025
$dotBrush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::FromArgb(170, 255, 255, 255))
$g.FillEllipse($dotBrush, $cx - $dotR, $size * 0.13, $dotR * 2, $dotR * 2)

$g.Dispose()

$outputPath = "D:\itharron\Document\Code\rust\bili_downloader\src-tauri\app-icon.png"
$bmp.Save($outputPath, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()

Write-Host "Icon saved to $outputPath ($((Get-Item $outputPath).Length) bytes)"
