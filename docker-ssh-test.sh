#!/bin/bash

# Script para crear un servidor SSH de prueba con Docker
# Este servidor incluye herramientas de monitoreo del sistema

echo "🚀 Creando servidor SSH de prueba..."

# Crear imagen Docker con SSH y herramientas de monitoreo
cat > Dockerfile.ssh-test << 'EOF'
FROM ubuntu:22.04

# Instalar SSH y herramientas de monitoreo
RUN apt-get update && apt-get install -y \
    openssh-server \
    htop \
    iotop \
    nethogs \
    sysstat \
    procps \
    curl \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Configurar SSH
RUN mkdir /var/run/sshd
RUN echo 'root:password123' | chpasswd
RUN sed -i 's/#PermitRootLogin prohibit-password/PermitRootLogin yes/' /etc/ssh/sshd_config

# Crear script de monitoreo
RUN echo '#!/bin/bash\n\
while true; do\n\
    echo "=== $(date) ==="\n\
    echo "CPU:"\n\
    top -bn1 | grep "Cpu(s)" | awk "{print \$2}" | cut -d"%" -f1\n\
    echo "MEMORY:"\n\
    free -m | grep Mem | awk "{print \$3/\$2*100}"\n\
    echo "DISK:"\n\
    df -h / | tail -1 | awk "{print \$5}" | sed "s/%//"\n\
    echo "LOAD:"\n\
    uptime | awk "{print \$10 \$11 \$12}"\n\
    sleep 5\n\
done' > /usr/local/bin/monitor.sh

RUN chmod +x /usr/local/bin/monitor.sh

EXPOSE 22

CMD ["/usr/sbin/sshd", "-D"]
EOF

# Construir la imagen
echo "📦 Construyendo imagen Docker..."
docker build -f Dockerfile.ssh-test -t ssh-test-server .

# Ejecutar el contenedor
echo "🔧 Iniciando servidor SSH..."
docker run -d \
    --name ssh-test-server \
    -p 2222:22 \
    ssh-test-server

echo "✅ Servidor SSH iniciado!"
echo ""
echo "📋 Información de conexión:"
echo "   Host: localhost"
echo "   Puerto: 2222"
echo "   Usuario: root"
echo "   Contraseña: password123"
echo ""
echo "🔍 Para verificar que funciona:"
echo "   ssh -p 2222 root@localhost"
echo ""
echo "🛑 Para detener el servidor:"
echo "   docker stop ssh-test-server"
echo "   docker rm ssh-test-server" 