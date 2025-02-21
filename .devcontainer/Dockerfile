FROM --platform=linux/amd64 brainflow/brainflow:5.16.0 AS wrlinux-devcontainer

# Configurar shell a bash
SHELL ["/bin/bash", "-c"]

# Instalar dependencias y herramientas
ENV DEBIAN_FRONTEND=noninteractive
RUN apt update && apt install -yqq \
        build-essential cmake gcc g++ gdb \
        curl libssl-dev python3 python3-pip \
        openssh-client bridge-utils net-tools \
        inetutils-ping qemu-system-aarch64 \
        qemu-system-x86 bridge-utils iproute2 dnsmasq \
        gcc-aarch64-linux-gnu && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Configurar pip y Conan
RUN pip3 install --upgrade pip && pip3 install "conan>=2.0"

# Configurar directorios y entorno para Wind River Linux
RUN mkdir -p /opt/qnx-sdk

# Configurar perfiles de Conan para Wind River Linux
RUN mkdir -p /root/.conan2/profiles
COPY .devcontainer/conan-profiles /root/.conan2/profiles

# Copiar docker_entrypoint.sh
COPY .devcontainer/docker_entrypoint.sh /usr/local/bin/docker_entrypoint.sh
RUN chmod +x /usr/local/bin/docker_entrypoint.sh

# Comando por defecto para systemd
ENTRYPOINT ["/usr/local/bin/docker_entrypoint.sh"]
        