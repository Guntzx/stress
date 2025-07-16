# Guía para Probar el Monitoreo SSH

## Opción 1: Usar Docker (Recomendado)

### 1. Crear servidor SSH de prueba
```bash
# Dar permisos de ejecución al script
chmod +x docker-ssh-test.sh

# Ejecutar el script
./docker-ssh-test.sh
```

### 2. Configurar en la aplicación
- **Host:** localhost
- **Puerto:** 2222
- **Usuario:** root
- **Contraseña:** password123

### 3. Probar conexión
- Ve a "Opciones Generales" → "Monitoreo del Sistema"
- Selecciona "🌐 Remoto (SSH)"
- Ingresa los datos de conexión
- Haz clic en "🔗 Probar conexión"

## Opción 2: Usar WSL2 (Windows)

### 1. Habilitar SSH en WSL2
```bash
# En WSL2
sudo apt update
sudo apt install openssh-server

# Configurar SSH
sudo mkdir /var/run/sshd
sudo echo 'root:password123' | sudo chpasswd
sudo sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config

# Iniciar SSH
sudo service ssh start
```

### 2. Obtener IP de WSL2
```bash
# En PowerShell
wsl hostname -I
```

### 3. Configurar en la aplicación
- **Host:** [IP de WSL2]
- **Puerto:** 22
- **Usuario:** root
- **Contraseña:** password123

## Opción 3: Usar VirtualBox/VM

### 1. Crear VM Ubuntu
- Descargar Ubuntu Server
- Instalar en VirtualBox
- Configurar red en modo "Bridge"

### 2. Configurar SSH
```bash
sudo apt update
sudo apt install openssh-server
sudo systemctl enable ssh
sudo systemctl start ssh
```

### 3. Configurar en la aplicación
- **Host:** [IP de la VM]
- **Puerto:** 22
- **Usuario:** [tu_usuario]
- **Contraseña:** [tu_contraseña]

## Opción 4: Servicios Cloud Gratuitos

### 1. Oracle Cloud Free Tier
- Crear cuenta gratuita
- Crear instancia Ubuntu
- Configurar SSH

### 2. Google Cloud Platform
- Usar créditos gratuitos
- Crear instancia Compute Engine

### 3. AWS Free Tier
- Crear instancia EC2
- Usar key pair para SSH

## Verificación

### 1. Probar conexión básica
```bash
ssh -p [puerto] [usuario]@[host]
```

### 2. Verificar herramientas de monitoreo
```bash
# En el servidor remoto
htop
free -h
df -h
uptime
```

### 3. Probar desde la aplicación
- Habilitar monitoreo SSH
- Iniciar una prueba de carga
- Verificar que aparecen las métricas en tiempo real

## Solución de Problemas

### Error: Connection refused
- Verificar que SSH está corriendo
- Verificar puerto y firewall
- Probar con `telnet [host] [puerto]`

### Error: Authentication failed
- Verificar usuario y contraseña
- Verificar que el usuario tiene permisos SSH

### Error: Host key verification failed
- Agregar `-o StrictHostKeyChecking=no` al comando SSH
- O limpiar known_hosts: `ssh-keygen -R [host]`

### Métricas no aparecen
- Verificar que las herramientas están instaladas
- Verificar permisos de ejecución
- Revisar logs de la aplicación

## Comandos Útiles

### En el servidor remoto
```bash
# Instalar herramientas de monitoreo
sudo apt install htop iotop nethogs sysstat

# Verificar que SSH está corriendo
sudo systemctl status ssh

# Ver logs de SSH
sudo tail -f /var/log/auth.log
```

### En la máquina local
```bash
# Probar conexión SSH
ssh -v [usuario]@[host] -p [puerto]

# Verificar puerto abierto
telnet [host] [puerto]

# Escanear puertos
nmap -p 22 [host]
``` 