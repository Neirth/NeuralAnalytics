#!/bin/bash

set -e

# Crear el puente br0 si no existe
if ! ip link show br0 &>/dev/null; then
    echo "Creando el puente br0..."
    ip link add name br0 type bridge
    ip link set dev br0 up

    # Asignar una dirección IP estática a br0
    echo "Asignando dirección IP a br0..."
    ip addr add 192.168.100.1/24 dev br0
else
    echo "El puente br0 ya existe."
fi



# Configurar dnsmasq para br0
echo "Configurando dnsmasq..."
cat <<EOF > /etc/dnsmasq.d/br0.conf
interface=br0
dhcp-range=192.168.100.10,192.168.100.100,12h
dhcp-option=3,192.168.100.1
EOF

# Reiniciar dnsmasq
echo "Reiniciando dnsmasq..."
systemctl restart dnsmasq || dnsmasq --conf-file=/etc/dnsmasq.d/br0.conf --interface=br0

# Verificar la configuración
echo "Configuración final de br0:"
ip addr show br0
echo "dnsmasq configurado en br0 con rango DHCP: 192.168.100.10-192.168.100.100"

# Configurar el helper de QEMU
chmod u+s /usr/lib/qemu/qemu-bridge-helper

# Permitir el acceso a br0 desde QEMU
mkdir -p /etc/qemu
echo "allow br0" > /etc/qemu/bridge.conf

# Dejar el contenedor corriendo en primer plano si se ejecuta directamente
sleep infinity