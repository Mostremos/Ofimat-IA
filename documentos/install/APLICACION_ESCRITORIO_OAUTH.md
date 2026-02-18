# 📱 OAuth2 para Aplicaciones de Escritorio

## ✅ Configuración Correcta

Para aplicaciones de escritorio, Google OAuth2 funciona de manera diferente a aplicaciones web:

### En Google Cloud Console

1. **Tipo de aplicación:** "Aplicación de escritorio" (Desktop app)
2. **NO se requiere** configurar "URI de redireccionamiento autorizados" en la consola
3. La URI se especifica dinámicamente en cada solicitud OAuth

### En el Código

El código SÍ debe especificar la URI de redirección en la solicitud OAuth:

```rust
const REDIRECT_URI: &str = "http://localhost:1420/oauth/callback";
```

Esta URI se envía en cada solicitud de autorización, y Google la acepta para aplicaciones de escritorio.

## 🔍 Verificación

Con las nuevas credenciales de "Aplicación de escritorio":

1. ✅ No necesitas configurar URI en Google Cloud Console
2. ✅ El código debe seguir usando `http://localhost:1420/oauth/callback`
3. ✅ Google aceptará cualquier URI de localhost para aplicaciones de escritorio

## 🚀 Prueba Ahora

1. Espera a que Tauri recompile con las nuevas credenciales
2. Haz click en "Conectar con Google"
3. Deberías ver en la consola la URL de autorización
4. El navegador debería abrirse y permitirte seleccionar tu cuenta

## 📝 Nota

Las credenciales de "Aplicación de escritorio" son más flexibles que las de "Aplicación web" porque:
- No requieren configuración previa de URIs
- Aceptan cualquier URI de localhost
- Son ideales para aplicaciones de escritorio como Tauri
