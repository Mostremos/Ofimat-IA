# 🔐 Configuración de Google OAuth2

Guía paso a paso para configurar las credenciales de Google OAuth2 necesarias para la aplicación.

## 📋 Requisitos Previos

- Cuenta de Google Workspace (o cuenta personal de Google)
- Acceso a [Google Cloud Console](https://console.cloud.google.com/)

## 🚀 Pasos para Configurar

### Paso 1: Crear o Seleccionar Proyecto en Google Cloud Console

1. Ve a [Google Cloud Console](https://console.cloud.google.com/)
2. Si no tienes un proyecto:
   - Click en el selector de proyectos (arriba a la izquierda)
   - Click en "NUEVO PROYECTO"
   - Nombre: "Agente Ofimático" (o el que prefieras)
   - Click en "CREAR"
3. Si ya tienes un proyecto, selecciónalo del selector

### Paso 2: Habilitar las APIs Necesarias

1. En el menú lateral, ve a **"APIs y Servicios"** > **"Biblioteca"**
2. Busca y habilita las siguientes APIs (una por una):
   - ✅ **Google Calendar API**
     - Busca "Google Calendar API"
     - Click en el resultado
     - Click en "HABILITAR"
   - ✅ **Gmail API**
     - Busca "Gmail API"
     - Click en el resultado
     - Click en "HABILITAR"
   - ✅ **Google Drive API**
     - Busca "Google Drive API"
     - Click en el resultado
     - Click en "HABILITAR"

### Paso 3: Crear Credenciales OAuth 2.0

1. Ve a **"APIs y Servicios"** > **"Credenciales"**
2. Click en **"CREAR CREDENCIALES"** (arriba)
3. Selecciona **"ID de cliente OAuth 2.0"**

### Paso 4: Configurar la Aplicación OAuth

1. **Tipo de aplicación:** Selecciona **"Aplicación de escritorio"**
2. **Nombre:** "Agente Ofimático Desktop" (o el que prefieras)
3. **URI de redirección autorizados:**
   - Click en "+ AGREGAR URI"
   - Agrega: `http://localhost:1420/oauth/callback`
   - ⚠️ **IMPORTANTE:** Debe ser exactamente esta URL (sin espacios, sin trailing slash)
   - ❌ **NO agregues** `tauri://localhost:1420` - no es necesario
4. Click en **"CREAR"**

### Paso 5: Obtener las Credenciales

Después de crear, verás una ventana con:
- **ID de cliente** (Client ID)
- **Secreto de cliente** (Client Secret)

**⚠️ IMPORTANTE:** Guarda estos valores de forma segura. El secreto de cliente solo se muestra una vez.

### Paso 6: Configurar en el Proyecto

Edita el archivo: `src-tauri/src/google_auth.rs`

Busca estas líneas (alrededor de la línea 9-10):
```rust
const CLIENT_ID: &str = "TU_CLIENT_ID_AQUI";
const CLIENT_SECRET: &str = "TU_CLIENT_SECRET_AQUI";
```

Reemplaza con tus credenciales reales:
```rust
const CLIENT_ID: &str = "tu-client-id-real.apps.googleusercontent.com";
const CLIENT_SECRET: &str = "tu-client-secret-real";
```

### Paso 7: Verificar la Configuración

1. Guarda el archivo `google_auth.rs`
2. Tauri detectará el cambio y recompilará automáticamente
3. En la **aplicación de escritorio** (no en el navegador), click en "Conectar con Google"
4. Deberías ver la pantalla de autorización de Google

## 🔒 Seguridad

### ⚠️ NUNCA hagas esto:

- ❌ No commitees las credenciales al repositorio
- ❌ No compartas las credenciales públicamente
- ❌ No uses las mismas credenciales en múltiples proyectos sin restricciones

### ✅ SÍ haz esto:

- ✅ Mantén las credenciales en `.gitignore`
- ✅ Usa variables de entorno para producción
- ✅ Restringe los permisos OAuth al mínimo necesario
- ✅ Revisa periódicamente las credenciales en Google Cloud Console

## 🐛 Solución de Problemas

### Error: "redirect_uri_mismatch"

**Causa:** La URI de redirección no coincide exactamente.

**Solución:**
1. Ve a Google Cloud Console > Credenciales
2. Edita tu ID de cliente OAuth 2.0
3. Verifica que la URI sea exactamente: `http://localhost:1420/oauth/callback`
4. Asegúrate de que no haya espacios extra o caracteres especiales

### Error: "access_denied"

**Causa:** El usuario canceló la autorización o no tiene permisos.

**Solución:**
- Asegúrate de estar usando una cuenta con acceso a Google Workspace
- Verifica que las APIs estén habilitadas en el proyecto

### Error: "invalid_client"

**Causa:** Las credenciales son incorrectas.

**Solución:**
- Verifica que copiaste correctamente el Client ID y Client Secret
- Asegúrate de que no haya espacios extra al copiar
- Verifica que estás usando las credenciales del proyecto correcto

### Error: "window.__TAURI_IPC__ is not a function"

**Causa:** Estás probando en el navegador web (localhost:1420) en lugar de en la aplicación de escritorio.

**Solución:**
- ✅ Usa la **ventana de la aplicación Tauri** (la aplicación de escritorio)
- ❌ NO uses el navegador web en localhost:1420
- La API de Tauri solo funciona dentro de la aplicación, no en el navegador

## 📝 Notas Adicionales

- Las credenciales OAuth 2.0 son específicas del proyecto de Google Cloud
- Puedes crear múltiples credenciales OAuth para diferentes entornos (desarrollo, producción)
- Los tokens de acceso se guardan automáticamente en la base de datos SQLite
- Los tokens expiran después de un tiempo; la app los renovará automáticamente

## 🔄 Próximos Pasos

Una vez configuradas las credenciales:
1. Implementar el servidor HTTP local para recibir el callback OAuth ✅ (Ya implementado)
2. Guardar los tokens de forma segura en la base de datos ✅ (Ya implementado)
3. Implementar refresh token automático (Próximo paso)
