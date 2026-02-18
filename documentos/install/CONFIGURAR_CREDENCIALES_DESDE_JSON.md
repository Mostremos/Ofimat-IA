# 🔐 Configurar Credenciales desde JSON de Google

## 📋 Paso 1: Abrir el JSON

Abre el archivo JSON que te proporcionó Google. Típicamente se llama:
- `credentials.json`
- `client_secret_XXXXX.json`
- O similar

## 📋 Paso 2: Encontrar los Valores

El JSON tiene esta estructura:

```json
{
  "installed": {
    "client_id": "123456789-abcdefgh.apps.googleusercontent.com",
    "project_id": "tu-project-id",
    "auth_uri": "https://accounts.google.com/o/oauth2/auth",
    "token_uri": "https://oauth2.googleapis.com/token",
    "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
    "client_secret": "GOCSPX-tu-client-secret-aqui",
    "redirect_uris": ["http://localhost"]
  }
}
```

**Necesitas estos dos valores:**
- `client_id` → Este es tu CLIENT_ID
- `client_secret` → Este es tu CLIENT_SECRET

## 📋 Paso 3: Editar el Código

1. Abre el archivo: `src-tauri/src/google_auth.rs`
2. Busca las líneas 9-10:
   ```rust
   const CLIENT_ID: &str = "TU_CLIENT_ID_AQUI";
   const CLIENT_SECRET: &str = "TU_CLIENT_SECRET_AQUI";
   ```
3. Reemplaza con los valores del JSON:
   ```rust
   const CLIENT_ID: &str = "123456789-abcdefgh.apps.googleusercontent.com";
   const CLIENT_SECRET: &str = "GOCSPX-tu-client-secret-aqui";
   ```

**⚠️ IMPORTANTE:**
- Copia los valores EXACTAMENTE como aparecen en el JSON
- No agregues espacios extra
- Mantén las comillas dobles

## 📋 Paso 4: Guardar y Esperar

1. **Guarda** el archivo `google_auth.rs`
2. Tauri detectará el cambio automáticamente
3. Verás en la consola: `Compiling agente-ofimatica...`
4. Espera a que termine la recompilación (30 segundos - 2 minutos)
5. Una vez que termine, la aplicación se actualizará automáticamente

## 📋 Paso 5: Probar

1. En la **aplicación de escritorio** (la ventana que se abrió, NO el navegador)
2. Haz click en **"Conectar con Google"**
3. Se abrirá tu navegador para autorizar
4. Después de autorizar, verás "Autorización Exitosa"
5. La aplicación se conectará automáticamente

## 🔒 Seguridad

- ✅ El archivo JSON ya está en `.gitignore` (no se subirá al repositorio)
- ✅ Las credenciales están solo en tu código local
- ⚠️ **NUNCA** commitees el archivo JSON o el código con las credenciales

## 🐛 Si No Funciona

1. **Verifica que copiaste correctamente:**
   - Sin espacios al inicio o final
   - Con las comillas dobles
   - El CLIENT_ID termina en `.apps.googleusercontent.com`

2. **Verifica que la recompilación terminó:**
   - Deberías ver `Finished` en la consola
   - Si hay errores, aparecerán en rojo

3. **Verifica la URI de redirección en Google Cloud Console:**
   - Debe ser exactamente: `http://localhost:1420/oauth/callback`

## 💡 Alternativa: Usar el JSON Directamente

Si prefieres, podemos modificar el código para leer directamente del archivo JSON. Esto es más seguro pero requiere más cambios. Por ahora, la forma más simple es copiar los valores manualmente.
