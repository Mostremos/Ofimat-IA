# 🐛 Solución de Problemas Comunes

## Problema: "Waiting for your frontend dev server to start"

### Síntoma
Al ejecutar `npm run dev`, ves el mensaje:
```
Warn Waiting for your frontend dev server to start on http://localhost:1420/..
```

Y la aplicación no inicia.

### Causa
Tauri está esperando que Vite inicie el servidor de desarrollo, pero hay un problema de configuración.

### Solución

1. **Verifica que el puerto 1420 esté libre:**
   ```powershell
   Test-NetConnection -ComputerName localhost -Port 1420
   ```

2. **Si el puerto está ocupado:**
   - Cierra otras aplicaciones que usen el puerto 1420
   - O cambia el puerto en `vite.config.ts` y `tauri.conf.json`

3. **Verifica la configuración:**
   - En `tauri.conf.json`, `beforeDevCommand` debe ser `"npm run vite"` (no `"npm run dev"`)
   - En `package.json`, debe existir el script `"vite": "vite"`

4. **Reinicia el proceso:**
   - Presiona `Ctrl+C` para detener
   - Ejecuta de nuevo: `npm run dev`

## Problema: Error al compilar Rust

### Síntoma
Errores de compilación de Rust al ejecutar `npm run dev`.

### Solución

1. **Actualiza Rust:**
   ```bash
   rustup update
   ```

2. **Verifica que Visual C++ Build Tools esté instalado:**
   - Descarga desde: https://visualstudio.microsoft.com/visual-cpp-build-tools/
   - O instala Visual Studio con componentes de C++

3. **Limpia el cache de Cargo:**
   ```bash
   cd src-tauri
   cargo clean
   cd ..
   npm run dev
   ```

## Problema: "No se puede encontrar el módulo"

### Síntoma
Errores de TypeScript sobre módulos no encontrados.

### Solución

1. **Reinstala dependencias:**
   ```bash
   rm -rf node_modules package-lock.json
   npm install
   ```

2. **Verifica que todas las dependencias estén instaladas:**
   ```bash
   npm list --depth=0
   ```

## Problema: Puerto 1420 en uso

### Síntoma
Error: "Port 1420 is already in use"

### Solución

1. **Encuentra qué proceso usa el puerto:**
   ```powershell
   netstat -ano | findstr :1420
   ```

2. **Mata el proceso:**
   ```powershell
   taskkill /PID <número_del_proceso> /F
   ```

3. **O cambia el puerto:**
   - Edita `vite.config.ts`: cambia `port: 1420` a otro número (ej: 1421)
   - Edita `tauri.conf.json`: cambia `devPath` a `http://localhost:1421`

## Problema: La aplicación no se abre

### Síntoma
`npm run dev` se ejecuta sin errores, pero no aparece la ventana de la aplicación.

### Solución

1. **Verifica que no haya errores en la consola:**
   - Busca mensajes de error en rojo

2. **Verifica que Vite esté corriendo:**
   - Deberías ver: `VITE v5.x.x  ready in xxx ms`
   - Abre http://localhost:1420 en tu navegador para verificar

3. **Revisa los logs de Tauri:**
   - Los errores de Rust aparecen en la consola donde ejecutaste `npm run dev`

## Problema: Base de datos no se crea

### Síntoma
La aplicación inicia pero la base de datos no se crea.

### Solución

1. **Verifica permisos de escritura:**
   - Asegúrate de tener permisos para crear archivos en el directorio del proyecto

2. **Crea el directorio manualmente:**
   ```powershell
   New-Item -ItemType Directory -Path "data" -Force
   ```

3. **Verifica la ruta en el código:**
   - Revisa `src-tauri/src/main.rs` para ver cómo se determina la ruta

## Problema: Error de OAuth2

### Síntoma
Error al conectar con Google: "redirect_uri_mismatch" o similar.

### Solución

1. **Verifica las credenciales:**
   - Asegúrate de que `CLIENT_ID` y `CLIENT_SECRET` estén configurados en `google_auth.rs`
   - Verifica que la URI de redirección en Google Cloud Console sea exactamente: `http://localhost:1420/oauth/callback`

2. **Verifica que las APIs estén habilitadas:**
   - Google Calendar API
   - Gmail API
   - Google Drive API

## Obtener Más Ayuda

Si el problema persiste:

1. Revisa los logs completos en la consola
2. Busca el error específico en la documentación de Tauri: https://tauri.app/
3. Verifica la versión de Node.js y Rust:
   ```bash
   node --version
   rustc --version
   ```
