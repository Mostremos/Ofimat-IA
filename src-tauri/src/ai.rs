use crate::database::Database;
use serde::{Deserialize, Serialize};

// Función helper para obtener API key desde variables de entorno
fn get_groq_api_key() -> String {
    std::env::var("GROQ_API_KEY")
        .unwrap_or_else(|_| {
            eprintln!("⚠️ GROQ_API_KEY no encontrada. Usando placeholder.");
            "TU_API_KEY_GROQ".to_string()
        })
}
const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/chat/completions";
const GROQ_MODEL: &str = "llama-3.1-8b-instant"; // Modelo actualizado: rápido y confiable
// Alternativas disponibles si este no funciona:
// - "mixtral-8x7b-32768" - Más potente, mejor para tareas complejas
// - "gemma2-9b-it" - Buen balance velocidad/calidad
// - "llama-3.3-70b-versatile" - Si está disponible (versión más nueva)

// Alternativa: Ollama (local, completamente gratis)
// const OLLAMA_URL: &str = "http://localhost:11434/api/generate";
// const OLLAMA_MODEL: &str = "llama3.1";

#[derive(Debug, Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f64,
    max_tokens: i32,
    response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Debug, Deserialize)]
struct GroqResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    content: String,
}

/// Estructura para representar un evento extraído de lenguaje natural
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedEvent {
    pub title: String,
    #[serde(rename = "start_date")]
    pub start_date: Option<String>, // YYYY-MM-DD (fecha NUEVA si es edición)
    #[serde(rename = "start_time")]
    pub start_time: Option<String>, // HH:MM
    #[serde(rename = "end_date")]
    pub end_date: Option<String>,   // YYYY-MM-DD
    #[serde(rename = "end_time")]
    pub end_time: Option<String>,   // HH:MM
    pub is_all_day: bool,
    pub description: Option<String>,
    pub location: Option<String>,
    pub attendees: Option<Vec<String>>,
    #[serde(rename = "action")]
    pub action: Option<String>, // "create", "edit", "delete" - determina la acción a realizar
    #[serde(rename = "event_id")]
    pub event_id: Option<String>, // ID del evento a editar/eliminar (si aplica)
    #[serde(rename = "original_date")]
    pub original_date: Option<String>, // YYYY-MM-DD - fecha ORIGINAL del evento a editar (si se menciona)
}

/// Llamar a Groq API (Llama/Mixtral) para procesar una instrucción en lenguaje natural
/// 
/// `recent_events` es una lista de eventos recientes (últimos 10) para contexto
pub async fn process_natural_language(
    _db: &Database, 
    instruction: &str, 
    current_date: &str,
    recent_events: Option<&str>, // JSON string con eventos recientes para contexto
) -> Result<ParsedEvent, String> {
    // Obtener API key desde variables de entorno
    let api_key = get_groq_api_key();
    
    // Verificar que la API key esté configurada
    if api_key == "TU_API_KEY_GROQ" || api_key.is_empty() {
        return Err("API key de Groq no configurada. Por favor configura GROQ_API_KEY en un archivo .env o como variable de entorno. Obtén una gratis en https://console.groq.com/".to_string());
    }

    // Construir contexto de eventos recientes
    let events_context = if let Some(events_json) = recent_events {
        format!("\n\nEVENTOS RECIENTES (para referencia y edición):\n{}\n\nINSTRUCCIONES CRÍTICAS para identificar eventos:\n\n1. FECHA ORIGINAL vs FECHA NUEVA (⚠️ MUY IMPORTANTE):\n   - Si la instrucción menciona DOS fechas con palabras como 'mover', 'cambiar', 'corregir', 'moviéndolo', 'cambiándolo':\n     * La PRIMERA fecha mencionada es SIEMPRE la fecha ORIGINAL del evento existente\n     * La SEGUNDA fecha mencionada es la fecha NUEVA a la que se mueve el evento\n     * Ejemplos:\n       * \"evento del día 20-02-2024 moviéndolo a 24-02-2026\" → original_date = \"2024-02-20\", start_date = \"2026-02-24\"\n       * \"cambiar el evento de fecha 15-03-2024 a 20-03-2024\" → original_date = \"2024-03-15\", start_date = \"2024-03-20\"\n       * \"mover el evento del 10-01-2024 al 15-01-2024\" → original_date = \"2024-01-10\", start_date = \"2024-01-15\"\n   - ⚠️ CRÍTICO: SIEMPRE debes incluir 'original_date' cuando se mencionan dos fechas\n   - ⚠️ CRÍTICO: Busca en los eventos recientes el evento que tenga fecha EXACTA igual a 'original_date' en el campo 'date'\n   - ⚠️ CRÍTICO: Usa el 'id' (sin sufijo) del evento que coincida con 'original_date', NO uses un event_id aleatorio
   - ⚠️ CRÍTICO: Si el evento tiene fecha 2024, usa el ID del evento de 2024, NO uses IDs de eventos de otros años\n   - Si la instrucción menciona solo una fecha sin contexto de 'mover' o 'cambiar', esa fecha va en 'start_date' (no en original_date)\n\n2. TÍTULO DEL EVENTO:\n   - SIEMPRE copia el título EXACTO del evento existente de los eventos recientes\n   - NUNCA inventes un título nuevo como '¡Feliz cumpleaños!' o cualquier otro\n   - Si editas un evento, el título DEBE ser el mismo que aparece en los eventos recientes\n   - Si encuentras el evento por original_date, copia su título EXACTO\n\n3. IDENTIFICACIÓN DE EVENTOS (⚠️ PASO A PASO OBLIGATORIO):\n   PASO 1: Si la instrucción menciona una fecha ORIGINAL (primera fecha en contexto de mover/cambiar):\n     a) Extrae la fecha ORIGINAL y ponla en 'original_date' (formato: YYYY-MM-DD, ej: \"20-02-2024\" → \"2024-02-20\")\n     b) Busca en los eventos recientes el evento cuyo campo 'date' coincida EXACTAMENTE con 'original_date'\n     c) Si encuentras UN evento con esa fecha:\n        * Usa su 'event_id' EXACTO\n        * Copia su 'title' EXACTO\n        * Verifica su 'is_all_day' para preservar el tipo de evento\n     d) Si NO encuentras ningún evento con esa fecha:\n        * Devuelve 'action': 'error'\n        * 'title': 'No encontré ningún evento con la fecha [fecha mencionada]. Por favor verifica la fecha o menciona el título del evento.'\n     e) Si encuentras MÚLTIPLES eventos con esa fecha:\n        * Si la instrucción también menciona un título, busca el que coincida con AMBOS (fecha Y título)\n        * Si no hay título mencionado, devuelve 'action': 'error' pidiendo más especificidad\n   PASO 2: Si NO hay fecha ORIGINAL pero hay un título mencionado:\n     a) Busca el evento con ese título EXACTO en los eventos recientes\n     b) Usa su 'event_id' y copia su información\n   PASO 3: Si NO puedes identificar el evento con certeza:\n     a) NO inventes un event_id\n     b) Devuelve 'action': 'error' con 'title' explicando qué información falta\n\n4. PRESERVACIÓN DE HORARIOS Y PARTICIPANTES:\n   - Si la instrucción dice \"con los mismos horarios\" o \"con los mismos participantes\" o \"mismos horarios\" o \"mismos participantes\":\n     * NO pongas 'is_all_day: true' a menos que el evento original en los eventos recientes tenga 'is_all_day: true'\n     * Si el evento original NO es de todo el día (tiene 'is_all_day: false'), usa 'is_all_day: false' o no lo incluyas\n     * Deja 'start_time' y 'end_time' como null si se dice \"mismos horarios\" - el backend los preservará automáticamente\n     * Deja 'attendees' como null si se dice \"mismos participantes\" - el backend los preservará automáticamente\n   - Si la instrucción NO menciona horarios o participantes, deja esos campos como null para que se preserven\n\n5. CAMPOS A INCLUIR:\n   - Si editas un evento, SOLO incluye en el JSON los campos que se mencionan EXPLÍCITAMENTE para cambiar\n   - Los demás campos (horarios, participantes, descripción, ubicación) se mantendrán del evento original\n   - NUNCA inventes títulos, fechas o IDs de eventos. Si no estás seguro, pide clarificación.\n\n6. FORMATO DE FECHAS:\n   - Las fechas pueden venir en formato DD-MM-YYYY (ej: \"20-02-2024\") o YYYY-MM-DD (ej: \"2024-02-20\")\n   - SIEMPRE convierte a formato YYYY-MM-DD para 'original_date' y 'start_date'\n   - Ejemplo: \"20-02-2024\" → \"2024-02-20\", \"24-02-2026\" → \"2026-02-24\"", events_json)
    } else {
        String::new()
    };

    // Crear el prompt para el modelo con la fecha actual
    let system_prompt = format!(r#"Eres un asistente que extrae información de eventos de calendario de instrucciones en lenguaje natural.
IMPORTANTE: La fecha actual es {}. Usa esta fecha como referencia para calcular fechas relativas.

Responde SOLO con un JSON válido, sin texto adicional, usando este formato exacto:

{{
  "title": "Título del evento" (SIEMPRE requerido, nunca null. Si editas, COPIA EXACTAMENTE el título del evento existente de los eventos recientes, NO inventes un título nuevo),
  "start_date": "YYYY-MM-DD" o null (fecha NUEVA si es edición, fecha de inicio si es creación),
  "start_time": "HH:MM" o null si no se especifica o es todo el día,
  "end_date": "YYYY-MM-DD" o null,
  "end_time": "HH:MM" o null,
  "is_all_day": true o false (SIEMPRE requerido, nunca null),
  "description": "Descripción" o null,
  "location": "Ubicación" o null,
  "attendees": ["email1@ejemplo.com", "email2@ejemplo.com"] o null,
  "action": "create" o "edit" o "delete" o "error" (SIEMPRE requerido, nunca null),
  "event_id": "id-del-evento" o null (REQUERIDO si action es "edit" o "delete", null si action es "create" o "error"),
  "original_date": "YYYY-MM-DD" o null (fecha ORIGINAL del evento a editar - ⚠️ CRÍTICO: Si la instrucción menciona DOS fechas con palabras como 'mover', 'cambiar', 'corregir', 'moviéndolo', 'cambiándolo', 'del día X a Y', 'de fecha X a Y':\n     * La PRIMERA fecha mencionada es SIEMPRE la fecha ORIGINAL del evento existente\n     * La SEGUNDA fecha mencionada es la fecha NUEVA (va en start_date)\n     * Ejemplos:\n       - "evento del día 20-02-2024 moviéndolo a 24-02-2026" → original_date = "2024-02-20", start_date = "2026-02-24"\n       - "cambiar el evento de fecha 20-02-2024 moviéndolo a 24-02-2026" → original_date = "2024-02-20", start_date = "2026-02-24"\n       - "mover el evento del 10-01-2024 al 15-01-2024" → original_date = "2024-01-10", start_date = "2024-01-15"\n     * ⚠️ IMPORTANTE: Convierte el formato de fecha a YYYY-MM-DD (ej: "20-02-2024" → "2024-02-20")\n     * ⚠️ IMPORTANTE: Si hay original_date, DEBES buscar el evento con esa fecha en los eventos recientes y usar su event_id EXACTO)
}}

⚠️ IMPORTANTE: Si NO puedes identificar con certeza el evento a editar o te falta información crítica, devuelve:
{{"action": "error", "title": "Mensaje de error claro para el usuario explicando qué información falta", "event_id": null, "is_all_day": false}}

⚠️ CRÍTICO - LISTA DE EVENTOS VACÍA:
Si la lista de eventos recientes está vacía ([] o no se proporciona), NO inventes un event_id.
Si la instrucción es editar/eliminar un evento pero no hay eventos en la lista, devuelve:
{{"action": "error", "title": "No encontré ningún evento en el calendario que coincida con tu solicitud. Por favor, verifica la fecha o menciona el título exacto del evento.", "event_id": null, "is_all_day": false}}

Instrucciones CRÍTICAS:
1. ACCIÓN (action):
   - "create": Para crear un nuevo evento (default)
   - "edit": Si la instrucción menciona "corregir", "editar", "modificar", "cambiar", "actualizar", "arreglar", "mover" un evento existente
   - "delete": Si la instrucción menciona "eliminar", "borrar", "quitar" un evento
   - "error": Si NO puedes identificar con CERTEZA ABSOLUTA el evento a editar/eliminar o te falta información crítica
   - ⚠️ CRÍTICO: Si es "edit" o "delete", DEBES incluir "event_id" con el ID EXACTO del evento de los eventos recientes. Si NO estás 100% seguro del event_id, usa "action": "error" y explica qué falta.
   - ⚠️ CRÍTICO: NUNCA inventes un event_id. Si no puedes encontrarlo en los eventos recientes, devuelve "action": "error".
   - ⚠️ CRÍTICO: Si la lista de eventos está vacía y la instrucción es editar/eliminar, SIEMPRE devuelve "action": "error" con un mensaje explicando que no se encontraron eventos.
   - IMPORTANTE: Si la instrucción tiene múltiples acciones (ej: "corrige X y elimina Y"), procesa SOLO la primera acción mencionada. El usuario puede hacer otra instrucción para la segunda acción.

2. CÁLCULO DE FECHAS Y HORARIOS:
   - FECHA ACTUAL: {} (úsala como referencia absoluta)
   - "el próximo [día]" = el próximo [día] DESPUÉS de la fecha actual (nunca antes)
   - "mañana" = fecha actual + 1 día
   - "pasado mañana" = fecha actual + 2 días
   - Si se menciona un día de la semana, calcula la PRÓXIMA ocurrencia DESPUÉS de la fecha actual
   - Si no se especifica fecha, usa la fecha de hoy
   - ⚠️ CRÍTICO: Si la instrucción dice "con los mismos horarios" o "mismos horarios":
     * PRIMERO: Busca el evento original en los eventos recientes usando el 'event_id' o 'original_date'
     * SEGUNDO: Revisa el campo 'is_all_day' del evento original
     * TERCERO: Si el evento original tiene 'is_all_day: false' (tiene horarios específicos), DEBES usar 'is_all_day: false'
     * CUARTO: Si el evento original tiene 'is_all_day: true' (es de todo el día), entonces usa 'is_all_day: true'
     * ⚠️ NUNCA pongas 'is_all_day: true' si el evento original NO es de todo el día - esto destruiría los horarios
     * Deja 'start_time' y 'end_time' como null para que el backend preserve los horarios originales
   - ⚠️ CRÍTICO: Si la instrucción es editar un evento existente (action: "edit"):
     * SIEMPRE revisa el campo 'is_all_day' del evento original en los eventos recientes
     * Si el evento original tiene horarios (is_all_day: false), NUNCA lo conviertas a todo el día (is_all_day: true)
     * Si el evento original es de todo el día (is_all_day: true), puedes mantenerlo así o cambiarlo según la instrucción
   - Si NO se menciona "mismos horarios" y no se especifica hora, pero estás editando un evento existente:
     * Usa el mismo 'is_all_day' que el evento original (cópialo de los eventos recientes)
   - Si NO se menciona "mismos horarios" y no se especifica hora, y es un evento NUEVO (action: "create"):
     * Asume que es un evento de todo el día (is_all_day: true)
   - Para eventos de todo el día, start_time y end_time deben ser null
   - NUNCA uses fechas pasadas, siempre calcula hacia el futuro

3. EXTRACCIÓN DE DATOS:
   - Extrae emails de participantes si se mencionan
   - Sé preciso y NO inventes información que no esté en la instrucción
   - Si editas un evento, SIEMPRE incluye el título del evento EXISTENTE (cópialo exactamente de los eventos recientes, NO inventes un título nuevo)
   - Si editas un evento, solo incluye los campos que se mencionan explícitamente para cambiar
   - NUNCA uses null para "title", siempre usa el título EXACTO del evento existente de los eventos recientes
   - Si NO puedes identificar con certeza el evento a editar, devuelve: {{"action": "error", "title": "No puedo identificar el evento. Por favor menciona el título exacto del evento o una fecha más específica."}}{}"#, current_date, current_date, events_context);

    let user_prompt = format!("Extrae la información del evento de esta instrucción: \"{}\"", instruction);

    // Construir el request para Groq
    let request = GroqRequest {
        model: GROQ_MODEL.to_string(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt,
            },
            Message {
                role: "user".to_string(),
                content: user_prompt,
            },
        ],
        temperature: 0.3, // Baja temperatura para respuestas más precisas y consistentes
        max_tokens: 1024,
        response_format: Some(ResponseFormat {
            format_type: "json_object".to_string(),
        }),
    };

    eprintln!("🤖 Llamando a Groq API (modelo: {})...", GROQ_MODEL);
    eprintln!("📝 Instrucción: {}", instruction);
    
    // Llamar a Groq API
    let api_key = get_groq_api_key();
    let client = reqwest::Client::new();
    let response = client
        .post(GROQ_API_URL)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Error al hacer request a Groq: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        eprintln!("❌ Error de Groq API: {} - {}", status, error_text);
        return Err(format!("Error de Groq API: {} - {}", status, error_text));
    }

    let groq_response: GroqResponse = response
        .json()
        .await
        .map_err(|e| format!("Error al parsear respuesta de Groq: {}", e))?;

    // Extraer el texto de la respuesta
    let response_text = groq_response
        .choices
        .first()
        .and_then(|c| Some(c.message.content.clone()))
        .ok_or_else(|| "No se recibió respuesta de Groq".to_string())?;

    eprintln!("📥 Respuesta de Groq: {}", response_text);

    // Limpiar la respuesta (puede tener markdown o texto adicional)
    let cleaned_text = response_text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim()
        .to_string();

    // Parsear el JSON con manejo especial para title null
    // Primero parsear como Value para manejar title null
    let mut json_value: serde_json::Value = serde_json::from_str(&cleaned_text)
        .map_err(|e| format!("Error al parsear JSON de Groq: {}. Respuesta: {}", e, cleaned_text))?;
    
    // Verificar si title es null o vacío
    let title_is_empty = json_value.get("title")
        .and_then(|t| {
            if t.is_null() {
                Some(true)
            } else {
                t.as_str().map(|s| s.is_empty())
            }
        })
        .unwrap_or(true);
    
    // Si title es null/vacío y es una acción de edición, intentar obtenerlo de los eventos recientes
    let mut recovered_title: Option<String> = None;
    if title_is_empty && json_value.get("action").and_then(|a| a.as_str()) == Some("edit") {
        if let Some(events_json) = recent_events {
            if let Ok(events_vec) = serde_json::from_str::<Vec<serde_json::Value>>(events_json) {
                // Extraer event_id y original_date primero para evitar problemas de borrow
                let event_id_opt = json_value.get("event_id").and_then(|id| id.as_str()).map(|s| s.to_string());
                let original_date_opt = json_value.get("original_date").and_then(|d| d.as_str()).map(|s| s.to_string());
                
                // Primero intentar por event_id
                if let Some(event_id) = &event_id_opt {
                    if let Some(event) = events_vec.iter().find(|e| {
                        e.get("id").and_then(|id| id.as_str()) == Some(event_id.as_str())
                    }) {
                        if let Some(title) = event.get("title").and_then(|t| t.as_str()) {
                            recovered_title = Some(title.to_string());
                            eprintln!("⚠️ Título estaba vacío/null, usando título del evento existente (por event_id): {}", title);
                        }
                    }
                }
                
                // Si aún está vacío, intentar por original_date
                if recovered_title.is_none() {
                    if let Some(original_date) = &original_date_opt {
                        if let Some(event) = events_vec.iter().find(|e| {
                            e.get("date").and_then(|d| d.as_str()) == Some(original_date.as_str())
                        }) {
                            if let Some(title) = event.get("title").and_then(|t| t.as_str()) {
                                recovered_title = Some(title.to_string());
                                eprintln!("⚠️ Título estaba vacío/null, usando título del evento existente (por original_date {}): {}", original_date, title);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Aplicar el título recuperado si existe
    if let Some(title) = recovered_title {
        json_value["title"] = serde_json::Value::String(title);
    }
    
    // Si title sigue siendo null o vacío después de intentar recuperarlo, usar un título por defecto
    let title_still_empty = json_value.get("title")
        .and_then(|t| {
            if t.is_null() {
                Some(true)
            } else {
                t.as_str().map(|s| s.is_empty())
            }
        })
        .unwrap_or(true);
    
    if title_still_empty {
        json_value["title"] = serde_json::Value::String("Evento sin título".to_string());
        eprintln!("⚠️ Título vacío después de validación, usando título por defecto");
    }
    
    // Ahora parsear el JSON corregido a ParsedEvent
    let parsed: ParsedEvent = serde_json::from_value(json_value)
        .map_err(|e| format!("Error al convertir JSON a ParsedEvent: {}", e))?;

    eprintln!("✅ Evento parseado: {:?}", parsed);

    // VALIDACIÓN ESTRICTA: Si es una acción de edición o eliminación, verificar que tenga event_id
    if let Some(action) = &parsed.action {
        if (action == "edit" || action == "delete") && parsed.event_id.is_none() {
            return Err(format!(
                "⚠️ La IA intentó {} un evento pero no proporcionó un event_id. Por favor, sé más específico mencionando el título exacto del evento o la fecha exacta del evento que quieres {}.",
                action,
                if action == "edit" { "editar" } else { "eliminar" }
            ));
        }
    }

    Ok(parsed)
}
