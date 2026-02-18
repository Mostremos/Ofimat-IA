# 🎨 Crear Iconos para la Aplicación

## Problema

Tauri requiere iconos para compilar la aplicación. En desarrollo, podemos desactivar el bundle, pero para producción necesitarás crear iconos.

## Solución Temporal (Desarrollo)

Ya hemos desactivado el bundle en `tauri.conf.json` para desarrollo. Esto permite compilar sin iconos.

## Solución Permanente (Producción)

Para compilar la aplicación final, necesitarás crear iconos. Tauri requiere:

### Iconos Necesarios

- `icons/32x32.png` - 32x32 píxeles (PNG)
- `icons/128x128.png` - 128x128 píxeles (PNG)
- `icons/128x128@2x.png` - 256x256 píxeles (PNG, para pantallas Retina)
- `icons/icon.ico` - Icono de Windows (ICO, múltiples tamaños)
- `icons/icon.icns` - Icono de macOS (ICNS, múltiples tamaños)

### Opción 1: Generar con Herramientas Online

1. **Crea un icono base:**
   - Diseña un icono simple (puede ser texto, logo, etc.)
   - Tamaño recomendado: 1024x1024 píxeles

2. **Convierte a los formatos necesarios:**
   - **Para PNG:** Usa cualquier editor de imágenes (GIMP, Paint.NET, Photoshop)
   - **Para ICO:** Usa https://convertio.co/es/png-ico/ o https://icoconvert.com/
   - **Para ICNS:** Usa https://cloudconvert.com/png-to-icns

3. **Coloca los archivos en:**
   ```
   src-tauri/icons/
   ```

### Opción 2: Usar Tauri Icon Generator (Recomendado)

1. **Instala la herramienta:**
   ```bash
   npm install -g @tauri-apps/cli
   ```

2. **Crea un icono base (1024x1024 PNG):**
   - Diseña o descarga un icono
   - Guárdalo como `icon.png` en la raíz del proyecto

3. **Genera todos los iconos automáticamente:**
   ```bash
   tauri icon icon.png
   ```

   Esto creará automáticamente todos los iconos necesarios en `src-tauri/icons/`

### Opción 3: Iconos Placeholder Temporales

Si solo quieres probar, puedes crear iconos básicos de un solo color:

1. Crea imágenes PNG de 32x32, 128x128 y 256x256 píxeles
2. Conviértelas a ICO e ICNS usando herramientas online
3. Colócalas en `src-tauri/icons/`

## Activar Bundle para Producción

Una vez que tengas los iconos:

1. Edita `src-tauri/tauri.conf.json`
2. Cambia `"active": false` a `"active": true`
3. Agrega la lista de iconos:
   ```json
   "icon": [
     "icons/32x32.png",
     "icons/128x128.png",
     "icons/128x128@2x.png",
     "icons/icon.icns",
     "icons/icon.ico"
   ]
   ```

## Notas

- Los iconos deben ser cuadrados (misma altura y ancho)
- Formatos soportados: PNG, ICO, ICNS
- Tamaño mínimo recomendado: 512x512 para el icono base
- Para mejor calidad, usa 1024x1024 o superior
