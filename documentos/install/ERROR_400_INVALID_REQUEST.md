# 🐛 Solución: Error 400: invalid_request

## Problema

Al hacer click en "Conectar con Google", aparece el error:
```
Error 400: invalid_request
Detalles de la solicitud: flowName=GeneralOAuthFlow
```

## Causas Comunes

### 1. Redirect URI no coincide exactamente

**La causa más común:** El `redirect_uri` en el código no coincide EXACTAMENTE con el configurado en Google Cloud Console.

**Solución:**
1. Ve a [Google Cloud Console](https://console.cloud.google.com/) > Credenciales
2. Edita tu ID de cliente OAuth 2.0
3. Verifica que en "URI de redirección autorizados" esté EXACTAMENTE:
   ```
   http://localhost:1420/oauth/callback
   ```
4. Sin espacios, sin trailing slash, sin mayúsculas diferentes
5. En el código (`src-tauri/src/google_auth.rs` línea 11), verifica que sea:
   ```rust
   const REDIRECT_URI: &str = "http://localhost:1420/oauth/callback";
   ```

### 2. CLIENT_ID o CLIENT_SECRET con espacios

**Causa:** Espacios extra al copiar las credenciales.

**Solución:**
1. Abre `src-tauri/src/google_auth.rs`
2. Verifica las líneas 9-10:
   ```rust
   const CLIENT_ID: &str = "tu-client-id.apps.googleusercontent.com";
   const CLIENT_SECRET: &str = "tu-client-secret";
   ```
3. Asegúrate de que NO haya espacios:
   - ❌ `" tu-client-id "` (con espacios)
   - ✅ `"tu-client-id"` (sin espacios)

### 3. Tipo de aplicación incorrecto

**Causa:** Las credenciales fueron creadas para otro tipo de aplicación.

**Solución:**
1. Ve a Google Cloud Console > Credenciales
2. Verifica que el tipo sea **"Aplicación de escritorio"** (Desktop app)
3. Si no, crea nuevas credenciales con el tipo correcto

### 4. APIs no habilitadas

**Causa:** Las APIs necesarias no están habilitadas en el proyecto.

**Solución:**
1. Ve a Google Cloud Console > APIs y Servicios > Biblioteca
2. Verifica que estén habilitadas:
   - ✅ Google Calendar API
   - ✅ Gmail API
   - ✅ Google Drive API

## Verificación Paso a Paso

### Paso 1: Verificar URL en la Consola

1. Haz click en "Conectar con Google"
2. En la consola donde ejecutaste `npm run dev`, busca:
   ```
   URL de autorización: https://accounts.google.com/o/oauth2/v2/auth?...
   ```
3. Copia esa URL completa
4. Ábrela manualmente en el navegador
5. Si funciona manualmente, el problema es cómo se abre desde la app
6. Si no funciona, el problema está en los parámetros

### Paso 2: Verificar Credenciales

En la consola deberías ver:
```
Usando CLIENT_ID: 969339938999-xxxxx.apps.googleusercontent.com
Usando REDIRECT_URI: http://localhost:1420/oauth/callback
```

Verifica que:
- El CLIENT_ID termine en `.apps.googleusercontent.com`
- El REDIRECT_URI sea exactamente `http://localhost:1420/oauth/callback`

### Paso 3: Probar URL Manualmente

Copia la URL de autorización de la consola y ábrela manualmente en el navegador. Si funciona, el problema es con cómo se abre desde la aplicación.

## Solución Rápida

1. **Verifica en Google Cloud Console:**
   - URI de redirección: `http://localhost:1420/oauth/callback` (exactamente así)
   - Tipo: Aplicación de escritorio

2. **Verifica en el código:**
   - `REDIRECT_URI` debe ser exactamente `http://localhost:1420/oauth/callback`
   - Sin espacios en CLIENT_ID o CLIENT_SECRET

3. **Recompila:**
   - Guarda los cambios
   - Espera a que Tauri recompile
   - Prueba de nuevo

## Si Persiste el Error

1. Crea nuevas credenciales OAuth 2.0 en Google Cloud Console
2. Asegúrate de que el tipo sea "Aplicación de escritorio"
3. Agrega EXACTAMENTE: `http://localhost:1420/oauth/callback`
4. Actualiza el código con las nuevas credenciales
5. Prueba de nuevo
