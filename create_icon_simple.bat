@echo off
REM Script simple para convertir icon.png a icon.ico usando ImageMagick
REM Requiere tener convert en el PATH

convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico

echo "icon.ico generado." 