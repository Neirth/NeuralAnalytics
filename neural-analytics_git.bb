SUMMARY = "Neural Analytics - Advanced EEG Signal Processing and Analysis"
DESCRIPTION = "Complete neural analytics application built in Rust for real-time EEG signal processing, \
machine learning analysis, and interactive visualization. Features a modern GUI interface with \
advanced neural network models for pattern recognition and classification."

# When used as a Yocto layer, the recipe uses local sources via file:// protocol
# For external consumption, modify SRC_URI to use git:// protocol

HOMEPAGE = "https://github.com/neirth/neural-analytics"
LICENSE = "GPL-3.0-only"
LIC_FILES_CHKSUM = "file://LICENSE.md;md5=94c359fdcd998d0d4b751c0501592d63"

# Local source configuration (default for layer usage)
SRC_URI = "file://${THISDIR}"
S = "${WORKDIR}/neural-analytics"

# Alternative: For external git consumption, uncomment these lines:
# SRC_URI = "git://github.com/neirth/neural-analytics.git;protocol=https;branch=main"
# SRCREV = "${AUTOREV}"
# S = "${WORKDIR}/git"

# Graphics backend configuration - can be overridden in local.conf
NEURAL_ANALYTICS_GRAPHICS_BACKEND ??= "drm" # Options: "wayland", "drm", "minimal"

# Core dependencies always required
DEPENDS = " \
    bluez5 \
    glibc \
    fontconfig \
    freetype \
    alsa-lib \
    systemd \
    dbus \
    udev \
    zlib \
    openssl \
    pkgconfig-native \
"

# Graphics dependencies based on backend selection
DEPENDS:append = "${@bb.utils.contains('NEURAL_ANALYTICS_GRAPHICS_BACKEND', 'wayland', ' mesa virtual/libgl virtual/egl wayland wayland-protocols libxkbcommon', '', d)}"
DEPENDS:append = "${@bb.utils.contains('NEURAL_ANALYTICS_GRAPHICS_BACKEND', 'drm', ' mesa virtual/libgl libdrm udev', '', d)}"
DEPENDS:append = "${@bb.utils.contains('NEURAL_ANALYTICS_GRAPHICS_BACKEND', 'minimal', ' libdrm', '', d)}"

# Runtime dependencies
RDEPENDS:${PN} = " \
    bluez5 \
    glibc \
    fontconfig \
    freetype \
    alsa-lib \
    systemd \
    dbus \
    udev \
    openssl \
"

# Graphics runtime dependencies based on backend
RDEPENDS:${PN}:append = "${@bb.utils.contains('NEURAL_ANALYTICS_GRAPHICS_BACKEND', 'wayland', ' mesa wayland libxkbcommon', '', d)}"
RDEPENDS:${PN}:append = "${@bb.utils.contains('NEURAL_ANALYTICS_GRAPHICS_BACKEND', 'drm', ' mesa libdrm', '', d)}"
RDEPENDS:${PN}:append = "${@bb.utils.contains('NEURAL_ANALYTICS_GRAPHICS_BACKEND', 'minimal', ' libdrm', '', d)}"

# Rust configuration
inherit cargo

# Build configuration
EXTRA_OECARGO_PATHS = "${S}"
CARGO_SRC_DIR = ""

# Installation paths
NEURAL_ANALYTICS_INSTALL_DIR = "/app"
NEURAL_ANALYTICS_BIN_DIR = "${NEURAL_ANALYTICS_INSTALL_DIR}/bin"
NEURAL_ANALYTICS_DATA_DIR = "${NEURAL_ANALYTICS_INSTALL_DIR}/data"
NEURAL_ANALYTICS_CONFIG_DIR = "${NEURAL_ANALYTICS_INSTALL_DIR}/config"
NEURAL_ANALYTICS_LIB_DIR = "${NEURAL_ANALYTICS_INSTALL_DIR}/lib"

# Build environment
export RUST_TARGET_PATH = "${WORKDIR}"
export PKG_CONFIG_ALLOW_CROSS = "1"
export PKG_CONFIG_PATH = "${STAGING_LIBDIR}/pkgconfig:${STAGING_DATADIR}/pkgconfig"
export PKG_CONFIG_LIBDIR = "${STAGING_LIBDIR}/pkgconfig"
export PKG_CONFIG_SYSROOT_DIR = "${STAGING_DIR_HOST}"

# Compilation flags for graphics dependencies
export CFLAGS:append = " -I${STAGING_INCDIR}"
export LDFLAGS:append = " -L${STAGING_LIBDIR}"

# Graphics backend specific flags
export CFLAGS:append = "${@bb.utils.contains('NEURAL_ANALYTICS_GRAPHICS_BACKEND', 'drm', ' -I${STAGING_INCDIR}/libdrm', '', d)}"
export CFLAGS:append = "${@bb.utils.contains('NEURAL_ANALYTICS_GRAPHICS_BACKEND', 'wayland', ' -I${STAGING_INCDIR}/wayland', '', d)}"

do_configure:prepend() {
    # Copy local sources to build directory when using file:// protocol
    if [ "${SRC_URI}" = "file://${THISDIR}" ]; then
        bbplain "Using local sources from layer directory..."
        cp -r ${THISDIR}/* ${S}/
        # Remove the recipe file from source directory to avoid conflicts
        rm -f ${S}/neural-analytics_git.bb
    fi
    
    # Ensure Cargo.lock exists
    if [ ! -f ${S}/Cargo.lock ]; then
        cd ${S}
        cargo generate-lockfile
    fi
    
    # Verify workspace structure
    if [ ! -f ${S}/Cargo.toml ]; then
        bbfatal "Cargo.toml not found in ${S}. Please check the source configuration."
    fi
    
    # Check if neural_analytics_gui exists
    if [ ! -d ${S}/packages/neural_analytics_gui ]; then
        bbfatal "neural_analytics_gui package not found. Please check the workspace structure."
    fi
    
    # Check if vendor directory exists
    if [ ! -d ${S}/vendor ]; then
        bbwarn "vendor directory not found in ${S}. Precompiled libraries will not be installed."
    fi
}

do_compile() {
    cd ${S}
    
    # Build the workspace in release mode
    cargo build \
        --release \
        --target-dir=${B}/target \
        --manifest-path=${S}/Cargo.toml \
        --workspace
}

do_install() {
    # Create application directories
    install -d ${D}${NEURAL_ANALYTICS_INSTALL_DIR}
    install -d ${D}${NEURAL_ANALYTICS_BIN_DIR}
    install -d ${D}${NEURAL_ANALYTICS_DATA_DIR}
    install -d ${D}${NEURAL_ANALYTICS_CONFIG_DIR}
    install -d ${D}${NEURAL_ANALYTICS_LIB_DIR}
    
    # Install main GUI binary
    install -m 0755 ${B}/target/release/neural_analytics_gui ${D}${NEURAL_ANALYTICS_BIN_DIR}/
    
    # Install precompiled libraries from vendor directory
    if [ -d ${S}/vendor ]; then
        bbplain "Installing precompiled libraries from vendor directory..."
        cp -r ${S}/vendor/* ${D}${NEURAL_ANALYTICS_LIB_DIR}/
        
        # Ensure proper permissions for libraries
        find ${D}${NEURAL_ANALYTICS_LIB_DIR} -name "*.so*" -exec chmod 755 {} \;
        find ${D}${NEURAL_ANALYTICS_LIB_DIR} -name "*.a" -exec chmod 644 {} \;
    else
        bbwarn "vendor directory not found. Skipping precompiled libraries installation."
    fi
    
    # Install dataset if present
    if [ -d ${S}/dataset ]; then
        cp -r ${S}/dataset/* ${D}${NEURAL_ANALYTICS_DATA_DIR}/
    fi
    
    # Create systemd service directory
    install -d ${D}${systemd_system_unitdir}
    
    # Install systemd service file with graphics backend specific configuration
    cat > ${D}${systemd_system_unitdir}/neural-analytics.service << EOF
[Unit]
Description=Neural Analytics EEG Processing Service
After=network.target bluetooth.service
Wants=bluetooth.service

[Service]
Type=simple
User=root
Group=root
WorkingDirectory=${NEURAL_ANALYTICS_INSTALL_DIR}
Environment=LD_LIBRARY_PATH=${NEURAL_ANALYTICS_LIB_DIR}
ExecStart=${NEURAL_ANALYTICS_BIN_DIR}/neural_analytics_gui
Restart=always
RestartSec=10
${@bb.utils.contains('NEURAL_ANALYTICS_GRAPHICS_BACKEND', 'wayland', 'Environment=WAYLAND_DISPLAY=wayland-0', '', d)}
${@bb.utils.contains('NEURAL_ANALYTICS_GRAPHICS_BACKEND', 'drm', 'Environment=SLINT_BACKEND=linuxkms-skia', '', d)}
${@bb.utils.contains('NEURAL_ANALYTICS_GRAPHICS_BACKEND', 'minimal', 'Environment=SLINT_BACKEND=linuxkms-software', '', d)}

[Install]
WantedBy=multi-user.target
EOF

    # Create desktop entry
    install -d ${D}${datadir}/applications
    cat > ${D}${datadir}/applications/neural-analytics.desktop << EOF
[Desktop Entry]
Name=Neural Analytics
Comment=EEG Signal Processing and Analysis
Exec=${NEURAL_ANALYTICS_BIN_DIR}/neural_analytics_gui
Icon=neural-analytics
Terminal=false
Type=Application
Categories=Science;Education;Medical;
StartupNotify=true
EOF

    # Create wrapper script for easier execution
    install -d ${D}${bindir}
    cat > ${D}${bindir}/neural-analytics << EOF
#!/bin/sh
# Set library path to include precompiled libraries
export LD_LIBRARY_PATH=${NEURAL_ANALYTICS_LIB_DIR}:\${LD_LIBRARY_PATH}
cd ${NEURAL_ANALYTICS_INSTALL_DIR}
exec ${NEURAL_ANALYTICS_BIN_DIR}/neural_analytics_gui "\$@"
EOF
    chmod +x ${D}${bindir}/neural-analytics
}

# Package configuration
PACKAGES = "${PN} ${PN}-dev ${PN}-dbg"

FILES:${PN} = " \
    ${NEURAL_ANALYTICS_INSTALL_DIR}/* \
    ${systemd_system_unitdir}/neural-analytics.service \
    ${datadir}/applications/neural-analytics.desktop \
    ${bindir}/neural-analytics \
"

FILES:${PN}-dev = " \
    ${includedir} \
    ${libdir}/*.so \
    ${libdir}/pkgconfig \
"

FILES:${PN}-dbg = " \
    ${NEURAL_ANALYTICS_INSTALL_DIR}/.debug \
    ${bindir}/.debug \
"

# Systemd integration
inherit systemd
SYSTEMD_SERVICE:${PN} = "neural-analytics.service"
SYSTEMD_AUTO_ENABLE:${PN} = "enable"

# Skip QA checks that might be problematic for Rust binaries
INSANE_SKIP:${PN} = "already-stripped ldflags"

# Architecture compatibility
COMPATIBLE_MACHINE = "(qemux86|qemux86-64|qemuarm|qemuarm64|raspberrypi|raspberrypi4)"

# Runtime recommendations
RRECOMMENDS:${PN} = " \
    kernel-module-bluetooth \
    bluez5-utils \
    mesa-demos \
    wayland-utils \
"
