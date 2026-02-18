# 🚀 Primer Uso de la Aplicación

## ✅ Estado Actual

¡La aplicación está funcionando correctamente!

- ✅ Ventana de la aplicación se abre
- ✅ Interfaz visible en localhost:1420
- ✅ Base de datos creada en `data/database.db`
- ⚠️ Botón "Conectar con Google" requiere configuración

## 🔧 Configurar Credenciales de Google OAuth2

Para que el botón "Conectar con Google" funcione, necesitas configurar las credenciales OAuth2.

### Paso 1: Obtener Credenciales de Google

Sigue la guía completa en: [`documentos/install/CONFIGURAR_GOOGLE_OAUTH.md`](CONFIGURAR_GOOGLE_OAUTH.md)

**Resumen rápido:**
1. Ve a [Google Cloud Console](https://console.cloud.google.com/)
2. Crea un proyecto o selecciona uno existente
3. Habilita las APIs: Calendar, Gmail, Drive
4. Crea credenciales OAuth 2.0 (tipo: Aplicación de escritorio)
5. Copia el Client ID y Client Secret

### Paso 2: Configurar en el Código

Edita el archivo: `src-tauri/src/google_auth.rs`

Busca estas líneas (alrededor de la línea 10-11):
```rust
const CLIENT_ID: &str = "TU_CLIENT_ID_AQUI";
const CLIENT_SECRET: &str = "TU_CLIENT_SECRET_AQUI";
```

Reemplázalas con tus credenciales reales:
```rust
const CLIENT_ID: &str = "tu-client-id-real.apps.googleusercontent.com";
const CLIENT_SECRET: &str = "tu-client-secret-real";
```

### Paso 3: Probar la Conexión

1. Guarda el archivo `google_auth.rs`
2. Tauri detectará el cambio y recompilará automáticamente
3. Una vez que termine la recompilación, haz click en "Conectar con Google"
4. Se abrirá tu navegador para autorizar la aplicación
5. Después de autorizar, verás "Autorización Exitosa"
6. La aplicación se conectará automáticamente

## 🐛 Solución de Problemas

### El botón no hace nada

**Causa:** Las credenciales no están configuradas.

**Solución:**
1. Abre la consola del navegador (F12)
2. Haz click en "Conectar con Google"
3. Verás un error en la consola indicando que las credenciales no están configuradas
4. Sigue el Paso 2 arriba para configurarlas

### Error: "redirect_uri_mismatch"

**Causa:** La URI de redirección en Google Cloud Console no coincide.

**Solución:**
- Verifica que la URI sea exactamente: `http://localhost:1420/oauth/callback`
- Sin espacios, sin trailing slash adicional

### Error al abrir el navegador

**Causa:** Problema con el comando para abrir el navegador.

**Solución:**
- El código intenta abrir el navegador automáticamente
- Si no funciona, copia la URL que aparece en la consola y ábrela manualmente

## 📝 Próximos Pasos

Una vez configuradas las credenciales:

1. **Probar la conexión:** Click en "Conectar con Google"
2. **Ver eventos:** Después de conectar, deberías ver eventos del calendario
3. **Siguiente fase:** Implementar la integración real con Google Calendar API

## 💡 Notas

- Las credenciales OAuth2 son específicas de tu proyecto de Google Cloud
- Los tokens se guardan automáticamente en la base de datos
- No necesitas reconectar cada vez que abres la aplicación (hasta que expire el token)
