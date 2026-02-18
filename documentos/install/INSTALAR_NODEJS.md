# Instalación de Node.js

## Opción 1: Instalador Oficial (Recomendado - Más Simple)

1. **Descargar Node.js:**
   - Ve a: https://nodejs.org/
   - Descarga la versión **LTS** (Long Term Support) - actualmente v20.x o v22.x
   - Elige el instalador para Windows (.msi)

2. **Instalar:**
   - Ejecuta el instalador descargado
   - Sigue el asistente (acepta todo por defecto)
   - **IMPORTANTE:** Asegúrate de que la opción "Add to PATH" esté marcada (viene marcada por defecto)

3. **Verificar instalación:**
   - Cierra y vuelve a abrir PowerShell/Terminal
   - Ejecuta:
     ```bash
     node --version
     npm --version
     ```
   - Deberías ver números de versión

4. **Si no funciona después de instalar:**
   - Reinicia tu computadora
   - O cierra todas las ventanas de terminal y abre una nueva

## Opción 2: NVM-Windows (Gestor de Versiones - Más Flexible)

Si prefieres poder cambiar entre versiones de Node.js fácilmente:

1. **Descargar NVM-Windows:**
   - Ve a: https://github.com/coreybutler/nvm-windows/releases
   - Descarga `nvm-setup.exe` (la última versión)

2. **Instalar:**
   - Ejecuta el instalador
   - Sigue el asistente

3. **Instalar Node.js con NVM:**
   - Abre PowerShell como **Administrador**
   - Ejecuta:
     ```bash
     nvm install lts
     nvm use lts
     ```

4. **Verificar:**
   ```bash
   node --version
   npm --version
   ```

## Verificación Final

Una vez instalado, desde el directorio del proyecto ejecuta:

```bash
cd "D:\Proyectos\Agente ofimática\Code"
node --version
npm --version
npm install
```

## Solución de Problemas

### "node no se reconoce como comando"
- Reinicia la terminal
- Si persiste, reinicia Windows
- Verifica que Node.js esté en el PATH:
  ```powershell
  $env:PATH -split ';' | Select-String nodejs
  ```

### Error de permisos al instalar paquetes
- Ejecuta PowerShell como Administrador
- O configura npm para no usar permisos elevados:
  ```bash
  npm config set prefix "$env:APPDATA\npm"
  ```
