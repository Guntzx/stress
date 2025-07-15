@echo off
REM Script para convertir icon.svg a icon.ico usando Inkscape e ImageMagick
REM Requiere tener ambos instalados y en el PATH

REM Convertir SVG a PNG
inkscape -z -e icon.png -w 256 -h 256 icon.svg

REM Convertir PNG a ICO
convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico

del icon.png

echo "icon.ico generado." 