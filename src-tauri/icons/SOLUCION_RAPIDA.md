# Solución Rápida: Convertir PNG a ICO

Ya tienes el archivo `icon.png` creado. Ahora necesitas convertirlo a `icon.ico`.

## Opción 1: Convertidor Online (Más Rápido - 2 minutos)

1. Ve a: https://convertio.co/es/png-ico/
2. Sube el archivo: `src-tauri/icons/icon.png`
3. Descarga el `icon.ico` resultante
4. Colócalo en: `src-tauri/icons/icon.ico`

## Opción 2: Usar Tauri CLI (Si está instalado)

```bash
npm install -g @tauri-apps/cli
tauri icon src-tauri/icons/icon.png
```

Esto generará todos los iconos necesarios automáticamente.

## Opción 3: Usar Python (Si tienes Pillow instalado)

```bash
pip install Pillow
python -c "from PIL import Image; img = Image.open('src-tauri/icons/icon.png'); img.save('src-tauri/icons/icon.ico', format='ICO', sizes=[(256,256)])"
```

## Después de crear icon.ico

Una vez que tengas `icon.ico` en `src-tauri/icons/`, ejecuta de nuevo:

```bash
npm run dev
```

La compilación debería continuar sin errores.
