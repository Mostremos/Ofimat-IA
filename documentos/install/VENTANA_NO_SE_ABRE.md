# 🔍 Solución: Ventana de Tauri No Se Abre

## Problema

La compilación termina exitosamente pero la ventana de la aplicación no se abre.

## Posibles Causas y Soluciones

### 1. Verificar que Tauri esté ejecutándose

**Síntoma:** La consola muestra "Finished" pero no se abre la ventana.

**Solución:**
- Busca en la barra de tareas si hay una ventana minimizada
- Verifica en el Administrador de Tareas si hay un proceso `agente-ofimatica.exe` corriendo
- Revisa si hay múltiples monitores - la ventana podría estar en otro monitor

### 2. Verificar la consola del navegador

**Síntoma:** La ventana se abre pero está en blanco o hay errores.

**Solución:**
1. Abre las herramientas de desarrollador (F12 o clic derecho > Inspeccionar)
2. Revisa la pestaña "Console" para ver errores de JavaScript
3. Revisa la pestaña "Network" para ver si hay problemas cargando recursos

### 3. Verificar permisos de Windows

**Síntoma:** Windows bloquea la aplicación.

**Solución:**
- Verifica el Firewall de Windows
- Revisa si Windows Defender está bloqueando la aplicación
- Ejecuta PowerShell como Administrador si es necesario

### 4. Verificar la configuración de Tauri

**Síntoma:** La ventana debería abrirse pero no lo hace.

**Solución:**
- Verifica que `tauri.conf.json` tenga `"visible": true` en la configuración de ventana
- Asegúrate de que `devPath` apunte correctamente a `http://localhost:1420`

### 5. Limpiar y Recompilar

**Síntoma:** Problemas persistentes después de cambios.

**Solución:**
```bash
# Detén el proceso (Ctrl+C)
# Limpia el cache de Rust
cd src-tauri
cargo clean
cd ..

# Vuelve a ejecutar
npm run dev
```

### 6. Verificar Logs de Tauri

**Síntoma:** Necesitas más información sobre qué está pasando.

**Solución:**
- Los logs de Tauri aparecen en la misma consola donde ejecutaste `npm run dev`
- Busca mensajes de error en rojo
- Verifica si hay mensajes sobre WebView2

### 7. Verificar WebView2

**Síntoma:** Tauri requiere WebView2 en Windows.

**Solución:**
- WebView2 viene preinstalado en Windows 11
- En Windows 10, puede necesitar instalación: https://developer.microsoft.com/microsoft-edge/webview2/

## Debugging Avanzado

### Habilitar Logs Detallados

Agrega esto temporalmente en `src-tauri/src/main.rs`:

```rust
fn main() {
    // Habilitar logs detallados
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    
    tauri::Builder::default()
    // ... resto del código
}
```

### Verificar que Vite esté corriendo

Abre en tu navegador: http://localhost:1420

Si ves la interfaz, entonces el problema es con Tauri, no con Vite.

## Estado Actual

- ✅ Compilación exitosa
- ✅ Vite corriendo en localhost:1420
- ❌ Ventana de Tauri no se abre

## Próximos Pasos

1. Verifica si hay un proceso `agente-ofimatica.exe` en el Administrador de Tareas
2. Revisa la consola donde ejecutaste `npm run dev` para ver si hay mensajes adicionales
3. Intenta abrir http://localhost:1420 en tu navegador para verificar que el frontend funciona
4. Si todo lo anterior está bien, puede ser un problema con WebView2 o permisos de Windows
