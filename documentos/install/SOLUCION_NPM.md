# Solución al Error de npm

## Problema Identificado

Tienes:
- Node.js v24.13.1 (versión muy nueva)
- npm v11.8.0 (versión muy nueva)

El error "Class extends value undefined is not a constructor or null" es un bug conocido de compatibilidad entre npm 11.x y Node.js 24.x.

## Soluciones (en orden de preferencia)

### Solución 1: Limpiar Cache y Reinstalar npm (Más Rápida)

Ejecuta estos comandos en PowerShell:

```powershell
# Limpiar cache de npm
npm cache clean --force

# Reinstalar npm globalmente
npm install -g npm@latest

# Intentar de nuevo
npm install
```

### Solución 2: Usar Versión LTS de Node.js (Más Estable - Recomendada)

Node.js v24 es muy nueva y puede tener problemas. Te recomiendo usar la versión LTS (v20.x):

1. **Descargar Node.js LTS:**
   - Ve a: https://nodejs.org/
   - Descarga la versión **LTS** (v20.x.x) en lugar de la Current (v24.x)
   - Instala sobre la versión actual (se reemplazará)

2. **Verificar:**
   ```powershell
   node --version  # Debería mostrar v20.x.x
   npm --version   # Debería mostrar v10.x.x
   ```

3. **Limpiar cache:**
   ```powershell
   npm cache clean --force
   ```

4. **Instalar dependencias:**
   ```powershell
   npm install
   ```

### Solución 3: Downgrade de npm (Si no quieres cambiar Node.js)

```powershell
npm install -g npm@10.9.2
npm cache clean --force
npm install
```

## Recomendación

**Usa la Solución 2 (Node.js LTS v20.x)** porque:
- Es más estable
- Tiene mejor compatibilidad con Tauri
- Es la versión recomendada para desarrollo

## Después de Resolver

Una vez que `npm install` funcione correctamente, continuamos con:
1. Configurar credenciales de Google OAuth2
2. Probar la aplicación
