# Neural Analytics BitBake Recipe - Integration Guide

## Repository as a BitBake Recipe

The Neural Analytics repository serves as a complete, self-contained BitBake recipe for Yocto/OpenEmbedded and Poky Linux systems. The included `neural-analytics_git.bb` recipe at the project root transforms this repository into a buildable package that integrates seamlessly with embedded Linux distributions.

### Recipe Architecture Philosophy

This repository functions as both source code and BitBake recipe, providing a unified development and deployment experience:

1. **Self-Contained Recipe**: Complete BitBake recipe included in the source repository
2. **Poky Linux Integration**: Designed for seamless integration with Poky-based distributions
3. **Embedded Optimization**: Configurable graphics backends for various embedded platforms
4. **Precompiled Libraries**: Support for vendor-specific libraries in `/app/lib`
5. **Systemd Integration**: Native service management with automatic startup

## Development Configurations

### Local Development Mode

For testing the recipe with local code changes without committing to git, temporarily modify the `SRC_URI`:

```bitbake
# Comment out git configuration
# SRC_URI = "git://github.com/neirth/neural-analytics.git;protocol=https;branch=main"
# SRCREV = "${AUTOREV}"

# Use local configuration
SRC_URI = "file://${THISDIR}"
S = "${WORKDIR}/neural-analytics"

# Copy local files
do_unpack:append() {
    cp -r ${THISDIR}/* ${S}/
}
```

### Production Configuration

The current configuration is optimized for production builds:

```bitbake
SRC_URI = "git://github.com/neirth/neural-analytics.git;protocol=https;branch=main"
SRCREV = "${AUTOREV}"
```

## Integration with Poky Linux Projects

Since this repository **is** the recipe itself, integration follows Yocto's standard patterns for self-contained recipe repositories:

### Method 1: Git Submodule Integration (Recommended)

Add this repository as a git submodule to your Poky-based project:

```bash
# In your Poky build directory
cd sources/
git submodule add https://github.com/neirth/neural-analytics.git meta-neural-analytics

# The repository structure becomes:
# sources/
#   poky/
#   meta-openembedded/
#   meta-neural-analytics/  <- Our repository
#     neural-analytics_git.bb  <- The recipe
#     packages/               <- Source code
#     dataset/               <- Data files
```

### Method 2: Direct Clone in Sources Directory

Clone the repository directly into your sources directory:

```bash
# In your Poky build environment
cd sources/
git clone https://github.com/neirth/neural-analytics.git meta-neural-analytics

# Add to conf/bblayers.conf
echo 'BBLAYERS += "${TOPDIR}/../sources/meta-neural-analytics"' >> conf/bblayers.conf
```

### Method 3: Layer Configuration for Recipe Discovery

Create a minimal layer.conf to make BitBake discover the recipe:

```bash
# Create conf/layer.conf in the repository root
mkdir -p conf/
cat > conf/layer.conf << 'EOF'
BBPATH .= ":${LAYERDIR}"
BBFILES += "${LAYERDIR}/*.bb ${LAYERDIR}/*.bbappend"
BBFILE_COLLECTIONS += "neural-analytics"
BBFILE_PATTERN_neural-analytics = "^${LAYERDIR}/"
BBFILE_PRIORITY_neural-analytics = "6"
LAYERDEPENDS_neural-analytics = "core"
LAYERSERIES_COMPAT_neural-analytics = "mickledore nanbield scarthgap"
EOF
```

## Poky Linux Build Workflow

### 1. Environment Setup

```bash
# Initialize Poky build environment
source poky/oe-init-build-env build-neural-analytics

# Add the neural-analytics repository to your layer configuration
echo 'BBLAYERS += "${TOPDIR}/../sources/meta-neural-analytics"' >> conf/bblayers.conf

# Configure target machine and include neural-analytics package
echo 'MACHINE = "qemux86-64"' >> conf/local.conf
echo 'IMAGE_INSTALL:append = " neural-analytics"' >> conf/local.conf
```

### 2. Graphics Backend Configuration (Optional)

```bash
# Configure graphics backend in local.conf
echo 'NEURAL_ANALYTICS_GRAPHICS_BACKEND = "drm"' >> conf/local.conf

# Available backends: "wayland", "drm", "minimal"
```

### 3. Build Execution

```bash
# Build only the neural analytics package
bitbake neural-analytics

# Build complete image with neural analytics included
bitbake core-image-minimal

# Or build with additional features
bitbake core-image-sato
```

### 4. Testing and Deployment

```bash
# Test in QEMU
runqemu qemux86-64

# Deploy to target hardware
dd if=tmp/deploy/images/qemux86-64/core-image-minimal-qemux86-64.wic of=/dev/sdX bs=4M
```

## Repository Structure for BitBake Consumption

When consumed as a Yocto layer, the repository structure becomes:

```
meta-neural-analytics/           <- Repository root (acts as layer)
├── conf/
│   └── layer.conf              <- Layer configuration (create if needed)
├── neural-analytics_git.bb     <- Main recipe file
├── packages/                   <- Source code (used by recipe)
├── dataset/                    <- Data files (copied to /app/data)
├── vendor/                     <- Precompiled libraries (copied to /app/lib)
└── docs/                       <- Documentation
```

## Development and Local Testing

### Local Development with File Protocol

For local development and testing without git commits, the recipe supports file:// protocol:

```bitbake
# Modify SRC_URI in neural-analytics_git.bb for local development
SRC_URI = "file://${THISDIR}"
S = "${WORKDIR}/neural-analytics"
```

This allows you to test recipe changes locally before committing to the repository.

### Production Git Configuration

For production builds, the recipe uses the standard git protocol:

```bitbake
SRC_URI = "git://github.com/neirth/neural-analytics.git;protocol=https;branch=main"
SRCREV = "${AUTOREV}"
```

## Recipe Automation and Helper Scripts

The following scripts can be added to your repository to facilitate development workflows:

### Development Mode Script (`dev-mode.sh`)
```bash
#!/bin/bash
# Switch to local development mode
cp neural-analytics_git.bb neural-analytics_git.bb.backup
sed -i 's/SRC_URI = "git:/# SRC_URI = "git:/' neural-analytics_git.bb
sed -i 's/SRCREV/# SRCREV/' neural-analytics_git.bb
echo 'SRC_URI = "file://${THISDIR}"' >> neural-analytics_git.bb
echo "Development mode activated - using local sources"
```

### Production Mode Script (`prod-mode.sh`)
```bash
#!/bin/bash
# Restore production configuration
if [ -f neural-analytics_git.bb.backup ]; then
    mv neural-analytics_git.bb.backup neural-analytics_git.bb
    echo "Production mode restored - using git sources"
else
    echo "Backup not found - cannot restore production mode"
fi
```

### Build Integration Script (`setup-layer.sh`)

```bash
#!/bin/bash
# Setup neural-analytics as a Yocto layer
POKY_BUILD_DIR="${1:-build}"

# Ensure we have layer.conf
if [ ! -f conf/layer.conf ]; then
    mkdir -p conf/
    cat > conf/layer.conf << 'EOF'
BBPATH .= ":${LAYERDIR}"
BBFILES += "${LAYERDIR}/*.bb ${LAYERDIR}/*.bbappend"
BBFILE_COLLECTIONS += "neural-analytics"
BBFILE_PATTERN_neural-analytics = "^${LAYERDIR}/"
BBFILE_PRIORITY_neural-analytics = "6"
LAYERDEPENDS_neural-analytics = "core"
LAYERSERIES_COMPAT_neural-analytics = "mickledore nanbield scarthgap"
EOF
fi

# Add to existing Poky build if path provided
if [ -d "${POKY_BUILD_DIR}/conf" ]; then
    LAYER_PATH=$(realpath .)
    echo "BBLAYERS += \"${LAYER_PATH}\"" >> ${POKY_BUILD_DIR}/conf/bblayers.conf
    echo "IMAGE_INSTALL:append = \" neural-analytics\"" >> ${POKY_BUILD_DIR}/conf/local.conf
    echo "Neural Analytics layer integrated into ${POKY_BUILD_DIR}"
else
    echo "Layer configuration created. Add this directory to your BBLAYERS in bblayers.conf"
fi
```

## Vendor Library Support

The recipe automatically handles precompiled libraries:

```bash
# Add vendor libraries to repository
mkdir vendor/
cp -r /path/to/precompiled/libs/* vendor/

# Commit vendor directory
git add vendor/
git commit -m "Add vendor libraries for embedded deployment"
git push origin main
```

The recipe will automatically:

- Copy `vendor/*` to `/app/lib/`
- Configure `LD_LIBRARY_PATH` for runtime
- Set proper permissions for shared libraries

## Poky Linux Distribution Considerations

### Target Platform Configuration

```bitbake
# For Raspberry Pi
MACHINE = "raspberrypi4"
NEURAL_ANALYTICS_GRAPHICS_BACKEND = "drm"

# For x86_64 embedded systems
MACHINE = "genericx86-64"
NEURAL_ANALYTICS_GRAPHICS_BACKEND = "wayland"

# For minimal embedded systems
MACHINE = "qemuarm"
NEURAL_ANALYTICS_GRAPHICS_BACKEND = "minimal"
```

### Image Integration

```bitbake
# Create custom image including neural analytics
inherit core-image
IMAGE_INSTALL += "neural-analytics bluez5 mesa"
IMAGE_FEATURES += "package-management"
```

## Best Practices for Yocto Integration

1. **Repository as Layer**: Treat this repository as a self-contained Yocto layer
2. **Version Management**: Use git tags for stable releases in production builds
3. **Layer Dependencies**: Ensure all LAYERDEPENDS are satisfied in your build environment
4. **Cross-Compilation**: The recipe handles Rust cross-compilation automatically
5. **Testing**: Always test with `bitbake neural-analytics` before including in images
6. **Documentation**: Keep this integration guide updated with recipe changes

This repository exemplifies Yocto's principle of self-contained, reusable components that can be easily integrated into embedded Linux distributions while maintaining both development flexibility and production reliability.
