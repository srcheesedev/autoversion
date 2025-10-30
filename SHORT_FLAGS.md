# 🚀 Autoversion - Guía de Flags Cortos

## 📋 **Flags Cortos Disponibles**

¡Ahora puedes usar flags cortos estilo Unix para comandos más rápidos y combinables!

### **Flags Principales**
| Flag Corto | Flag Largo | Descripción |
|------------|------------|-------------|
| `-b` | `--bump-type` | Tipo de bump (auto, major, minor, patch) |
| `-t` | `--technology` | Tecnología (auto, npm, cargo, maven, python, generic) |
| `-c` | `--create-tag` | Crear tag git |
| `-d` | `--dry-run` | Modo preview (sin cambios) |
| `-f` | `--force` | Forzar incluso con cambios uncommitted |
| `-C` | `--commit` | Commitear cambios |
| `-m` | `--commit-message` | Mensaje de commit personalizado |
| `-p` | `--path` | Ruta del proyecto |
| `-v` | `--verbose` | Output detallado |
| `-o` | `--output` | Formato de salida |
| `-i` | `--show-info` | Mostrar info del proyecto |
| `-a` | `--analyze` | Analizar commits |

## 🔥 **Comandos Combinados Ejemplos**

### **Combo Básico: Dry-run + Create-tag + Force**
```bash
./autoversion -cdf -b patch
# Equivale a: --create-tag --dry-run --force --bump-type patch
```

# 🚀 Autoversion - Guía de Flags Cortos
```bash
./autoversion -Ccv -b auto -m "Release v{version} 🚀"
# Equivale a: --commit --create-tag --verbose --bump-type auto --commit-message "Release v{version} 🚀"
```

### **Combo de Desarrollo: Dry-run + Verbose + Force**
```bash
./autoversion -dvf -t npm -b minor
# Equivale a: --dry-run --verbose --force --technology npm --bump-type minor
```

### **Combo de Análisis: Show-info + Analyze + Verbose**
```bash
./autoversion -iav
# Equivale a: --show-info --analyze --verbose
```

### **Combo Ultra-rápido**
```bash
./autoversion -cd -b auto    # Preview con tag
./autoversion -Cc -b patch   # Commit y tag patch
./autoversion -df            # Dry-run forzado
./autoversion -av            # Análisis verbose
```

## ⚡ **Casos de Uso Comunes**

### **🔍 Preview Rápido**
```bash
./autoversion -d              # Solo preview
./autoversion -dv             # Preview verbose
./autoversion -df             # Preview forzado
./autoversion -dcv            # Preview + tag + verbose
```

### **🚀 Release Rápido**
```bash
./autoversion -Cc            # Commit + tag automático
./autoversion -Ccv           # Commit + tag + verbose
./autoversion -Cdf           # Commit + dry-run + force (testing)
```

### **📊 Análisis**
```bash
./autoversion -a             # Analizar commits
./autoversion -av            # Analizar verbose
./autoversion -i             # Info del proyecto
./autoversion -iv            # Info verbose
```

### **🛠️ Desarrollo**
```bash
./autoversion -dvf -b patch  # Testing patch forzado
./autoversion -Ccvf -b minor # Release minor completo
```

## 🎯 **Patrones Recomendados**

### **Para Desarrollo Diario**
```bash
# Preview rápido
alias av-check="./autoversion -dv"

# Release patch rápido  
alias av-patch="./autoversion -Cc -b patch"

# Release minor con mensaje
alias av-minor="./autoversion -Ccv -b minor -m 'feat: v{version}'"

# Análisis de commits
alias av-analyze="./autoversion -av"
```

### **Para CI/CD**
```bash
# Production release
./autoversion -Cc -b auto -m "chore: release v{version}"

# Preview en PR
./autoversion -dv -b auto

# Force en hotfix
./autoversion -Ccf -b patch -m "hotfix: v{version}"
```

## 🔧 **Combinaciones Avanzadas**

### **Release Completo con Todo**
```bash
./autoversion -Ccvf -t npm -b auto -m "🚀 Release v{version}" --tag-prefix "release-"
```

### **Testing Multi-opción**
```bash
./autoversion -dvf -t auto -b major -p ./my-project -o json
```

### **Análisis Detallado**
```bash
./autoversion -aiv -p ./project --output json
```

## 💡 **Tips de Uso**

1. **Orden de Flags**: No importa el orden: `-cdf` = `-fdc` = `-dfc`
2. **Combinación Segura**: Siempre usa `-d` primero para preview
3. **Flags Repetidos**: Si repites un flag, el último gana
4. **Validación**: La herramienta valida combinaciones incompatibles
5. **Help Rápido**: `./autoversion -h` para ayuda rápida

## 🎨 **Estilo Unix**

¡Ahora autoversion se comporta como las herramientas clásicas de Unix!

```bash
# Como tar
tar -czf archive.tar.gz files/

# Como ls  
ls -la

# Como git
git commit -am "message"

# Como autoversion 🚀
./autoversion -cdf -b auto
```

¡Disfruta de la velocidad y flexibilidad de los flags cortos! ⚡