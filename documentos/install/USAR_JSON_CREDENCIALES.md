# 📄 Usar JSON de Credenciales de Google

Google te proporcionó un archivo JSON con las credenciales. Aquí te explico cómo usarlo.

## 📋 Contenido del JSON

El archivo JSON de Google típicamente tiene esta estructura:

```json
{
  "installed": {
    "client_id": "tu-client-id.apps.googleusercontent.com",
    "project_id": "tu-project-id",
    "auth_uri": "https://accounts.google.com/o/oauth2/auth",
    "token_uri": "https://oauth2.googleapis.com/token",
    "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
    "client_secret": "tu-client-secret",
    "redirect_uris": ["http://localhost"]
  }
}
```

## 🔧 Opción 1: Leer del JSON (Recomendado para Desarrollo)

Podemos modificar el código para leer las credenciales desde el JSON.

### Paso 1: Colocar el JSON

1. Coloca el archivo JSON en: `src-tauri/credentials.json`
2. Asegúrate de que esté en `.gitignore` (ya está configurado)

### Paso 2: Modificar el Código

Edita `src-tauri/src/google_auth.rs` para leer del JSON.

## 🔧 Opción 2: Copiar Manualmente (Más Simple)

### Paso 1: Abrir el JSON

Abre el archivo JSON que te dio Google con cualquier editor de texto.

### Paso 2: Encontrar los Valores

Busca estas líneas:
- `"client_id"`: Este es tu CLIENT_ID
- `"client_secret"`: Este es tu CLIENT_SECRET

### Paso 3: Editar google_auth.rs

Abre: `src-tauri/src/google_auth.rs`

Busca las líneas 9-10:
```rust
const CLIENT_ID: &str = "TU_CLIENT_ID_AQUI";
const CLIENT_SECRET: &str = "TU_CLIENT_SECRET_AQUI";
```

Reemplaza con los valores del JSON:
```rust
const CLIENT_ID: &str = "tu-client-id-del-json.apps.googleusercontent.com";
const CLIENT_SECRET: &str = "tu-client-secret-del-json";
```

### Paso 4: Guardar y Esperar Recompilación

1. Guarda el archivo
2. Tauri detectará el cambio automáticamente
3. Espera a que recompile (verás mensajes en la consola)
4. Una vez que termine, prueba el botón en la aplicación

## ⚠️ Importante

- **NO commitees** el archivo JSON al repositorio
- **NO compartas** las credenciales públicamente
- El archivo `credentials.json` ya está en `.gitignore`

## 🚀 Próximo Paso

Una vez configuradas las credenciales, prueba el botón "Conectar con Google" en la **aplicación de escritorio** (no en el navegador).
