# Configurar API Key de Gemini

Para usar la funcionalidad de IA con Gemini, necesitas obtener una API key de Google.

## Paso 1: Obtener API Key

1. Ve a [Google AI Studio](https://aistudio.google.com/app/apikey)
2. Inicia sesión con tu cuenta de Google (preferiblemente la misma que usas para Google Workspace)
3. Haz clic en "Create API Key" o "Get API Key"
4. Selecciona el proyecto de Google Cloud (o crea uno nuevo)
5. Copia la API key que se genera

## Paso 2: Configurar en el Código

1. Abre el archivo `src-tauri/src/gemini.rs`
2. Busca la línea:
   ```rust
   const GEMINI_API_KEY: &str = "TU_API_KEY_AQUI";
   ```
3. Reemplaza `"TU_API_KEY_AQUI"` con tu API key:
   ```rust
   const GEMINI_API_KEY: &str = "TU_API_KEY_REAL_AQUI";
   ```

## Paso 3: Verificar

1. Compila el proyecto: `npm run dev`
2. Si hay errores de compilación relacionados con Gemini, verifica que la API key esté correctamente configurada

## Notas Importantes

- **Seguridad**: La API key está hardcodeada en el código. Para producción, considera almacenarla de forma más segura (variables de entorno, archivo de configuración encriptado, etc.)
- **Límites**: Google AI Studio tiene límites de uso gratuito. Revisa los límites en [Google AI Studio](https://aistudio.google.com/)
- **Workspace Pro**: Si tienes Google Workspace Pro, puedes tener acceso a Gemini con mejores límites

## Solución de Problemas

### Error: "API key de Gemini no configurada"
- Verifica que hayas reemplazado `"TU_API_KEY_AQUI"` con tu API key real
- Asegúrate de que la API key esté entre comillas dobles

### Error: "Error de Gemini API: 400"
- Verifica que la API key sea válida
- Verifica que no hayas excedido los límites de uso

### Error: "Error de Gemini API: 403"
- Verifica que la API key tenga permisos para usar Gemini API
- Asegúrate de que la API de Gemini esté habilitada en tu proyecto de Google Cloud
