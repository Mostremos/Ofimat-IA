# Configurar Vertex AI para Gemini

Vertex AI es la plataforma empresarial de Google para usar modelos de IA como Gemini. Usa OAuth2 en lugar de API keys, lo que es más seguro.

## Paso 1: Habilitar Vertex AI API

1. Ve a [Google Cloud Console](https://console.cloud.google.com/)
2. Selecciona tu proyecto (o crea uno nuevo)
3. Ve a **APIs & Services** > **Library**
4. Busca "Vertex AI API"
5. Haz clic en **Enable** (Habilitar)

## Paso 2: Obtener el Project ID

1. En Google Cloud Console, ve a la página principal del proyecto
2. El **Project ID** se muestra en la parte superior (no confundir con Project Number)
3. Copia el Project ID (ejemplo: `mi-proyecto-123456`)

## Paso 3: Configurar en el Código

1. Abre el archivo `src-tauri/src/gemini.rs`
2. Busca la línea:
   ```rust
   const PROJECT_ID: &str = "TU_PROJECT_ID";
   ```
3. Reemplaza `"TU_PROJECT_ID"` con tu Project ID:
   ```rust
   const PROJECT_ID: &str = "tu-project-id-aqui";
   ```

## Paso 4: Verificar OAuth2

Vertex AI usa el mismo token OAuth2 que Google Calendar. Asegúrate de que:
- Ya estés conectado con Google en la aplicación
- El token OAuth2 tenga los scopes necesarios (ya los tienes configurados)

## Ventajas de Vertex AI

- ✅ **Más seguro**: No necesitas API keys hardcodeadas
- ✅ **Usa OAuth2**: El mismo sistema de autenticación que ya tienes
- ✅ **Mejor para empresas**: Diseñado para entornos empresariales
- ✅ **Mejores límites**: Generalmente tiene mejores límites que la API gratuita
- ✅ **Workspace Pro**: Si tienes Google Workspace Pro, tienes acceso incluido

## Solución de Problemas

### Error: "Project ID de Google Cloud no configurado"
- Verifica que hayas reemplazado `"TU_PROJECT_ID"` con tu Project ID real
- Asegúrate de que el Project ID esté entre comillas dobles

### Error: "Error de Vertex AI: 403"
- Verifica que Vertex AI API esté habilitada en tu proyecto
- Asegúrate de que el token OAuth2 tenga permisos para Vertex AI
- Puede ser necesario agregar el scope `https://www.googleapis.com/auth/cloud-platform` en `google_auth.rs`

### Error: "Error de Vertex AI: 404"
- Verifica que el Project ID sea correcto
- Asegúrate de que Vertex AI API esté habilitada en ese proyecto

### Error: "No hay token de Google"
- Conéctate primero con Google usando el botón "Conectar con Google" en la aplicación

## Nota sobre Scopes

Si obtienes un error 403, puede ser necesario agregar el scope de Vertex AI. Actualmente los scopes en `google_auth.rs` son:
- `calendar`
- `gmail.readonly`
- `drive.readonly`

Si es necesario, podemos agregar `https://www.googleapis.com/auth/cloud-platform` para acceso completo a Vertex AI.
