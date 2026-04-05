# Guía de distribución (mantenedores)

## Compilar para todas las plataformas

```bash
./build_release.sh
```

Los ejecutables quedan en `releases/`.

## Crear un release en GitHub

```bash
git tag v1.x.x
git push origin v1.x.x
# GitHub Actions compilará y publicará el release automáticamente
```

## Paquete macOS (.app / .pkg)

```bash
./create_app.sh   # genera Stress.app
./create_pkg.sh   # genera .pkg instalable
```

## Instalador Windows (.exe)

Requiere [NSIS](https://nsis.sourceforge.io/):

```cmd
cargo build --release --target x86_64-pc-windows-msvc
copy target\release\stress.exe releases\stress-windows-x64.exe
build_installer.bat
```

## Paquete Linux (.deb)

```bash
dpkg-buildpackage -b -us -uc
sudo dpkg -i stress_*.deb
```

## Docker

```bash
docker build -t stress .
docker run -it --rm stress --help
```

## Plataformas soportadas

| SO      | Requisitos mínimos | Dependencias |
|---------|--------------------|--------------|
| macOS   | 10.15+ (Catalina)  | Ninguna (binario estático) |
| Linux   | Ubuntu 18.04+      | libssl3, libgtk-3-0, libwebkit2gtk-4.0-37 |
| Windows | Windows 10+ 64-bit | Visual C++ Redistributable |
