# ✅ Verificación de Configuración OAuth2

## Configuración en Google Cloud Console

### ✅ Orígenes autorizados de JavaScript
```
http://localhost
```
**Correcto** - Esto permite que JavaScript desde localhost pueda hacer requests.

### ✅ URIs de redireccionamiento autorizados
```
http://localhost:1420/oauth/callback
```
**Correcto** - Esta es la URI exacta que usa la aplicación.

## ⚠️ Verificaciones Adicionales

### 1. Verificar que NO haya espacios extra

En Google Cloud Console, verifica que:
- ❌ NO haya espacios al inicio o final
- ❌ NO haya trailing slash: `http://localhost:1420/oauth/callback/` (incorrecto)
- ✅ Exactamente: `http://localhost:1420/oauth/callback`

### 2. Verificar el CLIENT_ID en el código

En `src-tauri/src/google_auth.rs`, verifica que uses variables de entorno:
```rust
fn get_client_id() -> String {
    std::env::var("GOOGLE_CLIENT_ID")
        .unwrap_or_else(|_| "TU_CLIENT_ID_AQUI".to_string())
}
```

Debe:
- ✅ Terminar en `.apps.googleusercontent.com`
- ✅ NO tener espacios
- ✅ Coincidir exactamente con el de Google Cloud Console
- ✅ Estar configurado en el archivo `.env` como `GOOGLE_CLIENT_ID`

### 3. Verificar el CLIENT_SECRET

En `src-tauri/src/google_auth.rs`, verifica que uses variables de entorno:
```rust
fn get_client_secret() -> String {
    std::env::var("GOOGLE_CLIENT_SECRET")
        .unwrap_or_else(|_| "TU_CLIENT_SECRET_AQUI".to_string())
}
```

Debe:
- ✅ NO tener espacios
- ✅ Coincidir exactamente con el de Google Cloud Console
- ✅ Estar configurado en el archivo `.env` como `GOOGLE_CLIENT_SECRET`

### 4. Verificar el REDIRECT_URI en el código

En `src-tauri/src/google_auth.rs` línea 11, debe ser:
```rust
const REDIRECT_URI: &str = "http://localhost:1420/oauth/callback";
```

Debe coincidir EXACTAMENTE con el configurado en Google Cloud Console.

## 🔍 Debug: Ver la URL Generada

Cuando hagas click en "Conectar con Google", en la consola deberías ver:

```
Usando CLIENT_ID: [primeros 20 caracteres]...
Usando REDIRECT_URI: http://127.0.0.1:1420/oauth/callback
URL de autorización: https://accounts.google.com/o/oauth2/v2/auth?client_id=...
```

**Nota**: El CLIENT_ID se muestra parcialmente por seguridad.

**Copia esa URL completa** y ábrela manualmente en el navegador. Si funciona manualmente, el problema está en cómo se abre desde la app. Si no funciona, el problema está en los parámetros.

## 🐛 Solución: Probar URL Manualmente

1. Haz click en "Conectar con Google"
2. Copia la URL completa de la consola
3. Pégalo en el navegador manualmente
4. Si funciona → El problema es cómo se abre desde la app
5. Si no funciona → El problema está en los parámetros o configuración de Google

## 📝 Nota sobre "Orígenes autorizados"

Los "Orígenes autorizados de JavaScript" son para aplicaciones web que hacen requests desde el navegador. Para aplicaciones de escritorio OAuth2, no son estrictamente necesarios, pero no hacen daño tenerlos.

Lo importante es que la **URI de redireccionamiento** sea exacta.
