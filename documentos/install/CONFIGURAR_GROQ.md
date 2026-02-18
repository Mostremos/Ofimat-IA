# Configurar Groq API (Gratis)

Groq es una excelente alternativa gratuita que usa modelos open source confiables como Llama 3.1 y Mixtral.

## Paso 1: Obtener API Key de Groq

1. Ve a [Groq Console](https://console.groq.com/)
2. Crea una cuenta (es gratis)
3. Ve a **API Keys** en el menú
4. Haz clic en **Create API Key**
5. Copia la API key generada

## Paso 2: Configurar en el Código

1. Abre el archivo `src-tauri/src/ai.rs`
2. Busca la línea:
   ```rust
   const GROQ_API_KEY: &str = "TU_API_KEY_GROQ";
   ```
3. Reemplaza `"TU_API_KEY_GROQ"` con tu API key:
   ```rust
   const GROQ_API_KEY: &str = "tu-api-key-aqui";
   ```

## Paso 3: Elegir Modelo (Opcional)

En `ai.rs` línea 7, puedes cambiar el modelo:

- `llama-3.1-70b-versatile` - Recomendado: muy confiable y preciso
- `llama-3.1-8b-instant` - Más rápido, bueno para respuestas simples
- `mixtral-8x7b-32768` - Buen balance velocidad/calidad

## Ventajas de Groq

- ✅ **Completamente gratis** (sin límites estrictos)
- ✅ **Muy rápido** (inferencia ultra-rápida)
- ✅ **Modelos open source confiables** (Llama, Mixtral)
- ✅ **Sin costos ocultos**
- ✅ **API simple** (similar a OpenAI)

## Solución de Problemas

### Error: "API key de Groq no configurada"
- Verifica que hayas reemplazado `"TU_API_KEY_GROQ"` con tu API key real
- Asegúrate de que la API key esté entre comillas dobles

### Error: "Error de Groq API: 401"
- Verifica que la API key sea correcta
- Asegúrate de que no haya espacios extra en la API key

### Error: "Error de Groq API: 429"
- Has excedido el límite de requests (muy raro, pero puede pasar)
- Espera unos minutos y vuelve a intentar

## Nota sobre Privacidad

Groq procesa las instrucciones en sus servidores. Si necesitas privacidad total, considera usar **Ollama** (local) en su lugar.
