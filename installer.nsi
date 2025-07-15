; Script NSIS para instalar Stress
Name "Stress - Herramienta de Pruebas de Carga"
OutFile "stress-setup.exe"
InstallDir "$PROGRAMFILES\Stress"
RequestExecutionLevel admin
Icon "icon.ico"

Section "Stress (CLI)"
  SetOutPath "$INSTDIR"
  File "releases\stress-windows-x64.exe"
  Rename "$INSTDIR\stress-windows-x64.exe" "$INSTDIR\stress.exe"
  CreateShortCut "$DESKTOP\Stress.lnk" "$INSTDIR\stress.exe" "--gui" "" 0
  CreateShortCut "$SMPROGRAMS\Stress (CLI).lnk" "$INSTDIR\stress.exe" "" "" 0
  CreateShortCut "$SMPROGRAMS\Stress.lnk" "$INSTDIR\stress.exe" "--gui" "" 0
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\stress.exe"
  Delete "$DESKTOP\Stress.lnk"
  Delete "$SMPROGRAMS\Stress (CLI).lnk"
  Delete "$SMPROGRAMS\Stress.lnk"
  RMDir /r "$INSTDIR"
SectionEnd 