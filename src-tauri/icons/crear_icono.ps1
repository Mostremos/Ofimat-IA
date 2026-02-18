# Script para crear un icono placeholder básico
# Este script crea un icono ICO mínimo usando .NET

Add-Type -AssemblyName System.Drawing

# Crear una imagen básica de 256x256
$bitmap = New-Object System.Drawing.Bitmap(256, 256)
$graphics = [System.Drawing.Graphics]::FromImage($bitmap)

# Fondo azul
$graphics.Clear([System.Drawing.Color]::FromArgb(66, 126, 234))

# Texto "AO" (Agente Ofimático)
$font = New-Object System.Drawing.Font("Arial", 120, [System.Drawing.FontStyle]::Bold)
$brush = New-Object System.Drawing.SolidBrush([System.Drawing.Color]::White)
$graphics.DrawString("AO", $font, $brush, 30, 60)

$graphics.Dispose()

# Guardar como PNG primero
$bitmap.Save("$PSScriptRoot\icon.png", [System.Drawing.Imaging.ImageFormat]::Png)
$bitmap.Dispose()

Write-Host "Icono PNG creado. Ahora necesitas convertirlo a ICO."
Write-Host "Usa una herramienta online como: https://convertio.co/es/png-ico/"
Write-Host "O descarga un icono placeholder desde: https://icon-library.com/"
