use crate::database::Database;
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration, TimeZone};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarEvent {
    pub id: String,
    pub title: String,
    pub start: String,
    pub end: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub is_all_day: bool, // Indica si es un evento de todo el día
    pub attendees: Option<Vec<String>>, // Lista de emails de participantes
}

/// Estructura para crear un nuevo evento desde el frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub title: String,
    pub start: String, // RFC3339 para eventos con hora, o YYYY-MM-DD para todo el día
    pub end: String,   // RFC3339 para eventos con hora, o YYYY-MM-DD para todo el día
    pub description: Option<String>,
    pub location: Option<String>,
    pub is_all_day: bool,
    pub attendees: Option<Vec<String>>, // Lista de emails de participantes
}

#[derive(Debug, Deserialize)]
struct GoogleCalendarResponse {
    items: Vec<GoogleCalendarEvent>,
}

#[derive(Debug, Deserialize)]
struct GoogleCalendarEvent {
    id: String,
    summary: String,
    start: GoogleDateTime,
    end: GoogleDateTime,
    description: Option<String>,
    location: Option<String>,
    attendees: Option<Vec<GoogleAttendee>>,
}

#[derive(Debug, Deserialize)]
struct GoogleAttendee {
    email: String,
    display_name: Option<String>,
    response_status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleDateTime {
    #[serde(rename = "dateTime")]
    date_time: Option<String>, // RFC3339 format with time and timezone
    date: Option<String>,      // YYYY-MM-DD format for all-day events
}

/// Extraer fecha del prompt del usuario (formato DD-MM-YYYY o DD/MM/YYYY)
/// Retorna la fecha en formato NaiveDate si se encuentra
pub fn extract_date_from_prompt(instruction: &str) -> Option<chrono::NaiveDate> {
    let instruction_lower = instruction.to_lowercase();
    let words: Vec<&str> = instruction_lower.split_whitespace().collect();
    
    for word in words {
        // Buscar formato DD-MM-YYYY o DD/MM/YYYY
        if word.contains('-') || word.contains('/') {
            let parts: Vec<&str> = if word.contains('-') {
                word.split('-').collect()
            } else {
                word.split('/').collect()
            };
            if parts.len() == 3 {
                if let (Ok(day), Ok(month), Ok(year)) = (
                    parts[0].parse::<u32>(),
                    parts[1].parse::<u32>(),
                    parts[2].parse::<u32>()
                ) {
                    if day >= 1 && day <= 31 && month >= 1 && month <= 12 && year >= 2000 && year <= 2100 {
                        // Asumir formato DD-MM-YYYY (más común en español)
                        if let Some(date) = chrono::NaiveDate::from_ymd_opt(year as i32, month, day) {
                            return Some(date);
                        }
                    }
                }
            }
        }
    }
    None
}

pub async fn get_events(db: &Database) -> Result<Vec<CalendarEvent>, String> {
    get_events_with_range(db, None, None).await
}

/// Obtener eventos del calendario con un rango de fechas opcional
/// Si se proporciona target_date, busca eventos en ese día específico
pub async fn get_events_with_range(
    db: &Database, 
    target_date: Option<chrono::NaiveDate>,
    days_range: Option<i64>
) -> Result<Vec<CalendarEvent>, String> {
    eprintln!("📅 Obteniendo eventos del calendario...");
    
    // Obtener token de la base de datos
    let token_data = db.get_token("google")
        .map_err(|e| format!("Error al obtener token: {}", e))?;
    
    let token = match token_data {
        Some(t) => {
            // Verificar si el token expiró
            let now = Utc::now().timestamp();
            if t.expires_at > 0 && t.expires_at < now {
                return Err("Token expirado. Por favor, vuelve a autenticarte.".to_string());
            }
            t.access_token
        },
        None => return Err("No hay token de autenticación. Por favor, conéctate primero.".to_string()),
    };
    
    // Calcular rango de búsqueda
    let now = Utc::now();
    let (time_min, time_max) = if let Some(date) = target_date {
        // Si hay una fecha objetivo, buscar en ese día específico (con margen de 1 día antes y después)
        // Convertir NaiveDate a DateTime<Utc> usando la zona horaria local
        let naive_dt = date.and_hms_opt(0, 0, 0).unwrap();
        // Usar la zona horaria de Argentina (UTC-3) como referencia
        let tz_offset = chrono::FixedOffset::west_opt(3 * 3600)
            .unwrap_or_else(|| chrono::FixedOffset::west_opt(0).unwrap());
        let target_datetime = tz_offset.from_local_datetime(&naive_dt)
            .single()
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| {
                // Si falla, usar UTC directamente
                Utc.from_utc_datetime(&naive_dt)
            });
        
        let search_start = target_datetime - Duration::days(1);
        let search_end = target_datetime + Duration::days(2); // +2 para incluir el día completo
        
        eprintln!("📅 Rango de búsqueda ajustado a fecha objetivo: {}", date.format("%Y-%m-%d"));
        eprintln!("   Desde: {} ({})", search_start.to_rfc3339(), search_start.format("%Y-%m-%d %H:%M:%S UTC"));
        eprintln!("   Hasta: {} ({})", search_end.to_rfc3339(), search_end.format("%Y-%m-%d %H:%M:%S UTC"));
        
        (search_start.to_rfc3339(), search_end.to_rfc3339())
    } else {
        // Rango por defecto: desde hace 7 días hasta 60 días en el futuro
        let days = days_range.unwrap_or(60);
        let time_min_dt = now - Duration::days(7);
        let time_max_dt = now + Duration::days(days);
        
        eprintln!("📅 Rango de búsqueda por defecto:");
        eprintln!("   Desde: {} ({})", time_min_dt.to_rfc3339(), time_min_dt.format("%Y-%m-%d %H:%M:%S UTC"));
        eprintln!("   Hasta: {} ({})", time_max_dt.to_rfc3339(), time_max_dt.format("%Y-%m-%d %H:%M:%S UTC"));
        
        (time_min_dt.to_rfc3339(), time_max_dt.to_rfc3339())
    };
    
    let url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/primary/events?timeMin={}&timeMax={}&singleEvents=true&orderBy=startTime&maxResults=500",
        urlencoding::encode(&time_min),
        urlencoding::encode(&time_max)
    );
    
    eprintln!("🔗 URL de la API: {}", url);
    eprintln!("🔗 Haciendo request a Google Calendar API...");
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Error al hacer request: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        eprintln!("❌ Error de API: {} - {}", status, error_text);
        
        if status == 401 {
            return Err("Token inválido o expirado. Por favor, vuelve a autenticarte.".to_string());
        }
        
        if status == 403 {
            return Err(
                "❌ Google Calendar API no está habilitada.\n\n".to_string() +
                "Por favor:\n" +
                "1. Ve a Google Cloud Console\n" +
                "2. Habilita 'Google Calendar API' en tu proyecto\n" +
                "3. Espera 2-5 minutos para que se propague\n" +
                "4. Vuelve a intentar\n\n" +
                "Ver: documentos/install/HABILITAR_CALENDAR_API.md"
            );
        }
        
        return Err(format!("Error de Google Calendar API: {} - {}", status, error_text));
    }
    
    // Leer respuesta como texto primero para debug
    let response_text = response.text().await
        .map_err(|e| format!("Error al leer respuesta: {}", e))?;
    
    eprintln!("📄 Respuesta de la API (primeros 2000 caracteres):");
    eprintln!("{}", response_text.chars().take(2000).collect::<String>());
    
    let calendar_response: GoogleCalendarResponse = serde_json::from_str(&response_text)
        .map_err(|e| format!("Error al parsear respuesta JSON: {}\nPrimeros 500 chars: {}", e, 
            response_text.chars().take(500).collect::<String>()))?;
    
    eprintln!("✅ Eventos recibidos de la API: {}", calendar_response.items.len());
    
    // Convertir eventos de Google a nuestro formato
    let events: Vec<CalendarEvent> = calendar_response.items
        .into_iter()
        .map(|event| {
            // Google puede usar date_time (con hora) o date (todo el día)
            // Para eventos de todo el día, date viene en formato "YYYY-MM-DD"
            // Para eventos con hora, date_time viene en formato RFC3339
            // Debug: mostrar qué campos tiene el evento ANTES de procesar
            eprintln!("🔍 Procesando evento: '{}' (ID: {})", event.summary, event.id);
            eprintln!("   start.date_time: {:?}", event.start.date_time);
            eprintln!("   start.date: {:?}", event.start.date);
            eprintln!("   end.date_time: {:?}", event.end.date_time);
            eprintln!("   end.date: {:?}", event.end.date);
            
            // Determinar si es un evento de todo el día
            let is_all_day = event.start.date_time.is_none() && event.start.date.is_some();
            
            let start = if let Some(ref dt) = event.start.date_time {
                eprintln!("   ✅ Usando date_time para start: {}", dt);
                // Evento con hora específica - usar tal cual (ya viene en RFC3339 con zona horaria)
                dt.clone()
            } else if let Some(ref d) = event.start.date {
                // Evento de todo el día: Google devuelve solo "YYYY-MM-DD"
                // Para eventos de todo el día, pasamos la fecha directamente sin hora
                // El frontend usará is_all_day para formatearla correctamente
                d.clone()
            } else {
                eprintln!("   ❌ ERROR: Evento sin fecha de inicio: {}", event.id);
                eprintln!("   ⚠️ Usando fecha actual como fallback");
                Utc::now().to_rfc3339()
            };
            
            let end = if let Some(ref dt) = event.end.date_time {
                // Evento con hora específica - usar tal cual (ya viene en RFC3339 con zona horaria)
                eprintln!("   ✅ Usando date_time para end: {}", dt);
                dt.clone()
            } else if let Some(ref d) = event.end.date {
                // Evento de todo el día: Google usa el día siguiente como "end"
                // Por ejemplo, un evento del 24/02 tiene end="2026-02-25"
                // Para eventos de todo el día, pasamos la fecha directamente
                // El frontend usará is_all_day para formatearla correctamente
                eprintln!("   ✅ Usando date para end (todo el día): {}", d);
                d.clone()
            } else {
                eprintln!("   ❌ ERROR: Evento sin fecha de fin: {}", event.id);
                eprintln!("   ⚠️ Usando fecha actual + 1 hora como fallback");
                (Utc::now() + Duration::hours(1)).to_rfc3339()
            };
            
            // Debug: mostrar qué campos tiene el evento
            eprintln!("🔍 Procesando evento: '{}' (ID: {})", event.summary, event.id);
            eprintln!("   start.date_time: {:?}", event.start.date_time);
            eprintln!("   start.date: {:?}", event.start.date);
            eprintln!("   end.date_time: {:?}", event.end.date_time);
            eprintln!("   end.date: {:?}", event.end.date);
            
            // Sanitizar descripción: remover HTML básico
            let description = event.description.map(|desc| {
                // Remover tags HTML básicos y decodificar entidades comunes
                let cleaned = desc
                    .replace("<html>", "")
                    .replace("</html>", "")
                    .replace("<head>", "")
                    .replace("</head>", "")
                    .replace("<body>", "")
                    .replace("</body>", "")
                    .replace("<meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\">", "")
                    .replace("<br>", "\n")
                    .replace("<br/>", "\n")
                    .replace("<br />", "\n")
                    .replace("&nbsp;", " ")
                    .replace("&amp;", "&")
                    .replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&quot;", "\"")
                    .trim()
                    .to_string();
                
                // Si después de limpiar queda vacío o solo espacios, retornar None
                if cleaned.is_empty() {
                    None
                } else {
                    Some(cleaned)
                }
            }).flatten();
            
            eprintln!("   📌 Evento: '{}' - Inicio: {} - Fin: {} - Todo el día: {}", 
                event.summary, 
                start, 
                end,
                event.start.date_time.is_none() && event.start.date.is_some());
            
            // Convertir participantes de Google a lista de emails
            let attendees = event.attendees.as_ref().map(|atts| {
                atts.iter()
                    .map(|a| a.email.clone())
                    .collect()
            });
            
            CalendarEvent {
                id: event.id,
                title: event.summary,
                start,
                end,
                description,
                location: event.location,
                is_all_day,
                attendees,
            }
        })
        .collect();
    
    eprintln!("✅ Eventos procesados: {}", events.len());
    Ok(events)
}

/// Obtener un evento específico por ID desde Google Calendar
/// Útil para buscar eventos que están fuera del rango de fechas de get_events
pub async fn get_event_by_id(db: &Database, event_id: &str) -> Result<CalendarEvent, String> {
    eprintln!("📅 Obteniendo evento específico por ID: {}", event_id);
    
    // Obtener token de la base de datos
    let token_data = db.get_token("google")
        .map_err(|e| format!("Error al obtener token: {}", e))?;
    
    let token = match token_data {
        Some(t) => {
            // Verificar si el token expiró
            let now = Utc::now().timestamp();
            if t.expires_at > 0 && t.expires_at < now {
                return Err("Token expirado. Por favor, vuelve a autenticarte.".to_string());
            }
            t.access_token
        },
        None => return Err("No hay token de autenticación. Por favor, conéctate primero.".to_string()),
    };
    
    let url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/primary/events/{}",
        urlencoding::encode(event_id)
    );
    
    eprintln!("🔗 URL de la API: {}", url);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Error al hacer request: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        eprintln!("❌ Error de API: {} - {}", status, error_text);
        
        if status == 401 {
            return Err("Token inválido o expirado. Por favor, vuelve a autenticarte.".to_string());
        }
        if status == 404 {
            return Err(format!("Evento con ID '{}' no encontrado. Puede que ya haya sido eliminado.", event_id));
        }
        
        return Err(format!("Error de Google Calendar API: {} - {}", status, error_text));
    }
    
    let google_event: GoogleCalendarEvent = response
        .json()
        .await
        .map_err(|e| format!("Error al parsear respuesta: {}", e))?;
    
    // Convertir el evento de Google a nuestro formato
    let is_all_day = google_event.start.date_time.is_none() && google_event.start.date.is_some();
    
    let start = if let Some(ref dt) = google_event.start.date_time {
        dt.clone()
    } else if let Some(ref d) = google_event.start.date {
        d.clone()
    } else {
        Utc::now().to_rfc3339()
    };
    
    let end = if let Some(ref dt) = google_event.end.date_time {
        dt.clone()
    } else if let Some(ref d) = google_event.end.date {
        // Para eventos de todo el día, Google devuelve el día siguiente como end
        // Pero nosotros queremos el mismo día
        d.clone()
    } else {
        (Utc::now() + chrono::Duration::hours(1)).to_rfc3339()
    };
    
    // Convertir participantes de Google a lista de emails
    let attendees = google_event.attendees.as_ref().map(|atts| {
        atts.iter()
            .map(|a| a.email.clone())
            .collect()
    });
    
    Ok(CalendarEvent {
        id: google_event.id,
        title: google_event.summary,
        start,
        end,
        description: google_event.description,
        location: google_event.location,
        is_all_day,
        attendees,
    })
}

/// Crear un nuevo evento en Google Calendar
pub async fn create_event(db: &Database, event: CreateEventRequest) -> Result<CalendarEvent, String> {
    eprintln!("📝 Creando nuevo evento: '{}'", event.title);
    
    // Obtener token de la base de datos
    let token_data = db.get_token("google")
        .map_err(|e| format!("Error al obtener token: {}", e))?;
    
    let token = match token_data {
        Some(t) => {
            // Verificar si el token expiró
            let now = Utc::now().timestamp();
            if t.expires_at > 0 && t.expires_at < now {
                eprintln!("❌ Token expirado. Se requiere reautenticación.");
                return Err("Token expirado. Por favor, vuelve a autenticarte.".to_string());
            }
            t.access_token
        },
        None => {
            eprintln!("❌ No hay token de autenticación.");
            return Err("No hay token de autenticación. Por favor, conéctate primero.".to_string());
        },
    };
    
    // Construir el cuerpo del request según si es evento de todo el día o con hora
    let start_obj = if event.is_all_day {
        // Para eventos de todo el día, Google espera solo "date" (YYYY-MM-DD)
        // El end debe ser el día siguiente
        json!({
            "date": event.start
        })
    } else {
        // Para eventos con hora, usar dateTime en formato RFC3339
        json!({
            "dateTime": event.start,
            "timeZone": "America/Argentina/Buenos_Aires"
        })
    };
    
    let end_obj = if event.is_all_day {
        // Para eventos de todo el día, end debe ser el día siguiente
        // Parsear la fecha y agregar un día
        if let Ok(date) = chrono::NaiveDate::parse_from_str(&event.end, "%Y-%m-%d") {
            let next_day = date + Duration::days(1);
            json!({
                "date": next_day.format("%Y-%m-%d").to_string()
            })
        } else {
            // Fallback: usar la fecha tal cual (aunque debería ser el día siguiente)
            json!({
                "date": event.end
            })
        }
    } else {
        json!({
            "dateTime": event.end,
            "timeZone": "America/Argentina/Buenos_Aires"
        })
    };
    
    // Construir el JSON del evento
    let mut event_json = json!({
        "summary": event.title,
        "start": start_obj,
        "end": end_obj
    });
    
    // Agregar campos opcionales si existen
    if let Some(ref desc) = event.description {
        if !desc.trim().is_empty() {
            event_json["description"] = json!(desc);
        }
    }
    
    if let Some(ref loc) = event.location {
        if !loc.trim().is_empty() {
            event_json["location"] = json!(loc);
        }
    }
    
    // Agregar participantes si existen
    if let Some(ref attendees) = event.attendees {
        if !attendees.is_empty() {
            let attendees_json: Vec<serde_json::Value> = attendees
                .iter()
                .filter(|email| !email.trim().is_empty())
                .map(|email| json!({ "email": email.trim() }))
                .collect();
            if !attendees_json.is_empty() {
                event_json["attendees"] = json!(attendees_json);
            }
        }
    }
    
    eprintln!("📤 Enviando request a Google Calendar API...");
    eprintln!("📄 JSON del evento: {}", serde_json::to_string_pretty(&event_json).unwrap_or_default());
    
    let client = reqwest::Client::new();
    let url = "https://www.googleapis.com/calendar/v3/calendars/primary/events";
    
    // Construir URL con parámetro sendUpdates para enviar notificaciones a los invitados
    let mut request_url = url.to_string();
    if event.attendees.is_some() && !event.attendees.as_ref().unwrap().is_empty() {
        request_url = format!("{}?sendUpdates=all", url);
    }
    
    let response = client
        .post(&request_url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&event_json)
        .send()
        .await
        .map_err(|e| format!("Error al hacer request: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        eprintln!("❌ Error de API: {} - {}", status, error_text);
        
        if status == 401 {
            return Err("Token inválido o expirado. Por favor, vuelve a autenticarte.".to_string());
        }
        if status == 403 {
            return Err(format!(
                "Error 403: Permiso denegado para crear eventos en Google Calendar.\n\nPosibles causas:\n1. La Google Calendar API no está habilitada en tu proyecto.\n2. Los permisos OAuth no incluyen escritura (necesitas 'calendar' scope, no solo 'calendar.readonly').\n3. Vuelve a autenticarte con los permisos correctos."
            ));
        }
        
        return Err(format!("Error de Google Calendar API: {} - {}", status, error_text));
    }
    
    let created_event: GoogleCalendarEvent = response
        .json()
        .await
        .map_err(|e| format!("Error al parsear respuesta: {}", e))?;
    
    eprintln!("✅ Evento creado exitosamente: {}", created_event.id);
    
    // Convertir el evento creado a nuestro formato
    let is_all_day = created_event.start.date_time.is_none() && created_event.start.date.is_some();
    
    let start = if let Some(ref dt) = created_event.start.date_time {
        dt.clone()
    } else if let Some(ref d) = created_event.start.date {
        d.clone()
    } else {
        Utc::now().to_rfc3339()
    };
    
    let end = if let Some(ref dt) = created_event.end.date_time {
        dt.clone()
    } else if let Some(ref d) = created_event.end.date {
        // Para eventos de todo el día, restar un día al end
        if let Ok(date) = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            let end_date = date - Duration::days(1);
            end_date.format("%Y-%m-%d").to_string()
        } else {
            d.clone()
        }
    } else {
        (Utc::now() + chrono::Duration::hours(1)).to_rfc3339()
    };
    
    // Convertir participantes de Google a lista de emails
    let attendees = created_event.attendees.as_ref().map(|atts| {
        atts.iter()
            .map(|a| a.email.clone())
            .collect()
    });
    
    Ok(CalendarEvent {
        id: created_event.id,
        title: created_event.summary,
        start,
        end,
        description: created_event.description,
        location: created_event.location,
        is_all_day,
        attendees,
    })
}

/// Eliminar un evento de Google Calendar
pub async fn delete_event(db: &Database, event_id: &str) -> Result<(), String> {
    eprintln!("🗑️ Eliminando evento: {}", event_id);
    
    // Obtener token de la base de datos
    let token_data = db.get_token("google")
        .map_err(|e| format!("Error al obtener token: {}", e))?;
    
    let token = match token_data {
        Some(t) => {
            // Verificar si el token expiró
            let now = Utc::now().timestamp();
            if t.expires_at > 0 && t.expires_at < now {
                eprintln!("❌ Token expirado. Se requiere reautenticación.");
                return Err("Token expirado. Por favor, vuelve a autenticarte.".to_string());
            }
            t.access_token
        },
        None => {
            eprintln!("❌ No hay token de autenticación.");
            return Err("No hay token de autenticación. Por favor, conéctate primero.".to_string());
        },
    };
    
    let url = format!("https://www.googleapis.com/calendar/v3/calendars/primary/events/{}", 
        urlencoding::encode(event_id));
    
    eprintln!("📤 Enviando DELETE request a: {}", url);
    
    let client = reqwest::Client::new();
    let response = client
        .delete(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| format!("Error al hacer request: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        eprintln!("❌ Error de API: {} - {}", status, error_text);
        
        if status == 401 {
            return Err("Token inválido o expirado. Por favor, vuelve a autenticarte.".to_string());
        }
        if status == 403 {
            return Err("Error 403: Permiso denegado para eliminar eventos en Google Calendar.".to_string());
        }
        if status == 404 {
            return Err("Evento no encontrado. Puede que ya haya sido eliminado.".to_string());
        }
        
        return Err(format!("Error de Google Calendar API: {} - {}", status, error_text));
    }
    
    eprintln!("✅ Evento eliminado exitosamente");
    Ok(())
}

/// Actualizar un evento existente en Google Calendar
pub async fn update_event(db: &Database, event_id: &str, event: CreateEventRequest) -> Result<CalendarEvent, String> {
    eprintln!("✏️ Actualizando evento: {} - '{}'", event_id, event.title);
    
    // Obtener token de la base de datos
    let token_data = db.get_token("google")
        .map_err(|e| format!("Error al obtener token: {}", e))?;
    
    let token = match token_data {
        Some(t) => {
            // Verificar si el token expiró
            let now = Utc::now().timestamp();
            if t.expires_at > 0 && t.expires_at < now {
                eprintln!("❌ Token expirado. Se requiere reautenticación.");
                return Err("Token expirado. Por favor, vuelve a autenticarte.".to_string());
            }
            t.access_token
        },
        None => {
            eprintln!("❌ No hay token de autenticación.");
            return Err("No hay token de autenticación. Por favor, conéctate primero.".to_string());
        },
    };
    
    // Construir el cuerpo del request (similar a create_event)
    let start_obj = if event.is_all_day {
        json!({
            "date": event.start
        })
    } else {
        json!({
            "dateTime": event.start,
            "timeZone": "America/Argentina/Buenos_Aires"
        })
    };
    
    let end_obj = if event.is_all_day {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(&event.end, "%Y-%m-%d") {
            let next_day = date + Duration::days(1);
            json!({
                "date": next_day.format("%Y-%m-%d").to_string()
            })
        } else {
            json!({
                "date": event.end
            })
        }
    } else {
        json!({
            "dateTime": event.end,
            "timeZone": "America/Argentina/Buenos_Aires"
        })
    };
    
    // Construir el JSON del evento
    let mut event_json = json!({
        "summary": event.title,
        "start": start_obj,
        "end": end_obj
    });
    
    // Agregar campos opcionales si existen
    if let Some(ref desc) = event.description {
        if !desc.trim().is_empty() {
            event_json["description"] = json!(desc);
        }
    }
    
    if let Some(ref loc) = event.location {
        if !loc.trim().is_empty() {
            event_json["location"] = json!(loc);
        }
    }
    
    // Manejar participantes: 
    // - Si attendees es Some([]), enviar array vacío para eliminar todos los participantes
    // - Si attendees es Some([...]), enviar la lista de participantes
    // - Si attendees es None, no enviar el campo (mantener participantes existentes)
    if let Some(ref attendees) = event.attendees {
        let attendees_json: Vec<serde_json::Value> = attendees
            .iter()
            .filter(|email| !email.trim().is_empty())
            .map(|email| json!({ "email": email.trim() }))
            .collect();
        // Siempre enviar el campo attendees, incluso si está vacío (para eliminar participantes)
        event_json["attendees"] = json!(attendees_json);
    }
    // Si attendees es None, no agregamos el campo, manteniendo los participantes existentes
    
    eprintln!("📤 Enviando PATCH request a Google Calendar API...");
    eprintln!("📄 JSON del evento: {}", serde_json::to_string_pretty(&event_json).unwrap_or_default());
    
    let mut url = format!("https://www.googleapis.com/calendar/v3/calendars/primary/events/{}", 
        urlencoding::encode(event_id));
    
    // Agregar parámetro sendUpdates si hay participantes
    if event.attendees.is_some() && !event.attendees.as_ref().unwrap().is_empty() {
        url = format!("{}?sendUpdates=all", url);
    }
    
    let client = reqwest::Client::new();
    let response = client
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .json(&event_json)
        .send()
        .await
        .map_err(|e| format!("Error al hacer request: {}", e))?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        eprintln!("❌ Error de API: {} - {}", status, error_text);
        
        if status == 401 {
            return Err("Token inválido o expirado. Por favor, vuelve a autenticarte.".to_string());
        }
        if status == 403 {
            return Err("Error 403: Permiso denegado para editar eventos en Google Calendar.".to_string());
        }
        if status == 404 {
            return Err("Evento no encontrado. Puede que ya haya sido eliminado.".to_string());
        }
        
        return Err(format!("Error de Google Calendar API: {} - {}", status, error_text));
    }
    
    let updated_event: GoogleCalendarEvent = response
        .json()
        .await
        .map_err(|e| format!("Error al parsear respuesta: {}", e))?;
    
    eprintln!("✅ Evento actualizado exitosamente: {}", updated_event.id);
    
    // Convertir el evento actualizado a nuestro formato
    let is_all_day = updated_event.start.date_time.is_none() && updated_event.start.date.is_some();
    
    let start = if let Some(ref dt) = updated_event.start.date_time {
        dt.clone()
    } else if let Some(ref d) = updated_event.start.date {
        d.clone()
    } else {
        Utc::now().to_rfc3339()
    };
    
    let end = if let Some(ref dt) = updated_event.end.date_time {
        dt.clone()
    } else if let Some(ref d) = updated_event.end.date {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d") {
            let end_date = date - Duration::days(1);
            end_date.format("%Y-%m-%d").to_string()
        } else {
            d.clone()
        }
    } else {
        (Utc::now() + chrono::Duration::hours(1)).to_rfc3339()
    };
    
    // Convertir participantes de Google a lista de emails
    let attendees = updated_event.attendees.as_ref().map(|atts| {
        atts.iter()
            .map(|a| a.email.clone())
            .collect()
    });
    
    Ok(CalendarEvent {
        id: updated_event.id,
        title: updated_event.summary,
        start,
        end,
        description: updated_event.description,
        location: updated_event.location,
        is_all_day,
        attendees,
    })
}
