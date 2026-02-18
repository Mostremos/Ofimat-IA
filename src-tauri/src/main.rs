// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod google_auth;
mod calendar;
mod ai; // Reemplazamos gemini.rs con ai.rs para usar Groq/Ollama
mod memory; // Sistema de memoria persistente condensada
mod memory_condenser; // Funciones para condensar conversaciones y extraer conocimiento

use database::Database;
use tauri::Manager;
use serde_json::json;

fn main() {
    // Cargar variables de entorno desde .env (si existe)
    // En producción, las variables deben estar en el sistema
    dotenv::dotenv().ok();
    
    tauri::Builder::default()
        .setup(|app| {
            // Inicializar base de datos en el directorio del proyecto
            // Esto permite que sea portable
            // Intentar obtener el directorio del proyecto desde el ejecutable
            let db_path = match std::env::current_exe() {
                Ok(exe_path) => {
                // En desarrollo: exe está en src-tauri/target/debug/
                // En producción: exe está en la raíz del proyecto o en una subcarpeta
                let mut project_dir = exe_path.parent().unwrap();
                
                // Si estamos en target/debug o target/release, subir 3 niveles
                if project_dir.ends_with("debug") || project_dir.ends_with("release") {
                    project_dir = project_dir.parent().unwrap(); // target/
                    project_dir = project_dir.parent().unwrap(); // src-tauri/
                    project_dir = project_dir.parent().unwrap(); // Code/
                }
                // Si estamos en otra ubicación, intentar encontrar el directorio Code
                else {
                    // Buscar hacia arriba hasta encontrar un directorio con "Code" o "src-tauri"
                    let mut current = project_dir;
                    while let Some(parent) = current.parent() {
                        if parent.join("src-tauri").exists() || parent.file_name().and_then(|n| n.to_str()).map(|s| s.contains("Code")).unwrap_or(false) {
                            project_dir = parent;
                            break;
                        }
                        current = parent;
                    }
                }
                
                    let data_dir = project_dir.join("data");
                    std::fs::create_dir_all(&data_dir)
                        .map_err(|e| format!("Error al crear directorio data: {}", e))?;
                    data_dir.join("database.db")
                }
                Err(e) => {
                    eprintln!("Error al obtener ruta del ejecutable: {}", e);
                    // Fallback: usar directorio actual
                    let data_dir = std::path::Path::new("data");
                    std::fs::create_dir_all(&data_dir)
                        .map_err(|e| format!("Error al crear directorio data: {}", e))?;
                    data_dir.join("database.db")
                }
            };
            
            eprintln!("Inicializando base de datos en: {:?}", db_path);
            let db = Database::new(&db_path)
                .map_err(|e| format!("Error al crear base de datos: {}", e))?;
            app.manage(db);
            eprintln!("Base de datos inicializada correctamente");
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            is_google_connected,
            connect_google,
            connect_google_manual,
            disconnect_google,
            get_calendar_events,
            create_calendar_event,
            update_calendar_event,
            delete_calendar_event,
            create_event_from_natural_language,
            google_auth::oauth_callback,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn is_google_connected(db: tauri::State<'_, Database>) -> Result<bool, String> {
    database::has_valid_token(&db).await
}

#[tauri::command]
async fn connect_google(
    app: tauri::AppHandle,
    db: tauri::State<'_, Database>,
) -> Result<bool, String> {
    google_auth::authenticate(&app, &db).await
}

#[tauri::command]
async fn connect_google_manual(
    db: tauri::State<'_, Database>,
    auth_code: String,
) -> Result<bool, String> {
    google_auth::authenticate_with_code(&db, &auth_code).await
}

#[tauri::command]
async fn disconnect_google(db: tauri::State<'_, Database>) -> Result<bool, String> {
    db.delete_token("google")
        .map_err(|e| format!("Error al eliminar token: {}", e))?;
    Ok(true)
}

#[tauri::command]
async fn get_calendar_events(
    db: tauri::State<'_, Database>,
) -> Result<Vec<calendar::CalendarEvent>, String> {
    calendar::get_events(&db).await
}

#[tauri::command]
async fn create_calendar_event(
    db: tauri::State<'_, Database>,
    event: calendar::CreateEventRequest,
) -> Result<calendar::CalendarEvent, String> {
    calendar::create_event(&db, event).await
}

#[tauri::command]
async fn update_calendar_event(
    db: tauri::State<'_, Database>,
    event_id: String,
    event: calendar::CreateEventRequest,
) -> Result<calendar::CalendarEvent, String> {
    calendar::update_event(&db, &event_id, event).await
}

#[tauri::command]
async fn delete_calendar_event(
    db: tauri::State<'_, Database>,
    event_id: String,
) -> Result<(), String> {
    calendar::delete_event(&db, &event_id).await
}

#[tauri::command]
async fn create_event_from_natural_language(
    db: tauri::State<'_, Database>,
    instruction: String,
) -> Result<calendar::CalendarEvent, String> {
    // Obtener fecha actual para pasarla al modelo y validaciones
    let now = chrono::Utc::now();
    let tz_offset = chrono::FixedOffset::west_opt(3 * 3600)
        .unwrap_or_else(|| chrono::FixedOffset::west_opt(0).unwrap());
    let local_now = now.with_timezone(&tz_offset);
    let current_date_str = local_now.format("%Y-%m-%d").to_string();
    
    // Guardar la instrucción en minúsculas para usar en la lógica de preservación de campos
    let instruction_lower = instruction.to_lowercase();
    
    // ⚠️ CRÍTICO: Extraer fecha del prompt ANTES de buscar eventos
    // Esto permite ajustar el rango de búsqueda de Google Calendar
    let target_date = calendar::extract_date_from_prompt(&instruction);
    let target_original_date = target_date.map(|d| d.format("%Y-%m-%d").to_string());
    
    if let Some(ref date) = target_original_date {
        eprintln!("📅 Fecha original detectada en instrucción: {}", date);
        eprintln!("🔍 Ajustando rango de búsqueda de Google Calendar para incluir esta fecha...");
    }
    
    // Obtener eventos con rango ajustado si hay una fecha objetivo
    let recent_events = calendar::get_events_with_range(&db, target_date, None).await.ok();
    
    // Validar que se encontraron eventos si se mencionó una fecha
    if let Some(ref date) = target_original_date {
        if let Some(ref events) = recent_events {
            if events.is_empty() {
                return Err(format!(
                    "⚠️ No encontré ningún evento el {} en tu calendario. Por favor, verifica la fecha o menciona el título del evento.",
                    date
                ));
            }
            eprintln!("✅ Se encontraron {} eventos en la fecha objetivo", events.len());
        }
    }
    
    let recent_events_json = if let Some(events) = &recent_events {
        // Filtrar eventos si hay una fecha original mencionada
        let filtered_events: Vec<_> = if let Some(ref target_date) = target_original_date {
            eprintln!("🔍 Filtrando eventos por fecha original: {}", target_date);
            events.iter()
                .filter(|e| {
                    let event_date = if e.is_all_day {
                        e.start.clone()
                    } else {
                        chrono::DateTime::parse_from_rfc3339(&e.start)
                            .ok()
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or_default()
                    };
                    event_date == *target_date
                })
                .collect()
        } else {
            // Si no hay fecha original, tomar los últimos 20 eventos
            events.iter().take(20).collect()
        };
        
        eprintln!("📋 Pasando {} eventos a la IA (filtrados por fecha original si aplica)", filtered_events.len());
        
        // Crear JSON con información detallada para mejor identificación
        let events_summary: Vec<serde_json::Value> = filtered_events
            .iter()
            .map(|e| {
                // Extraer fecha legible del campo start
                let date_str = if e.is_all_day {
                    e.start.clone() // Ya está en formato YYYY-MM-DD
                } else {
                    // Parsear RFC3339 y extraer solo la fecha
                    chrono::DateTime::parse_from_rfc3339(&e.start)
                        .ok()
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| e.start.clone())
                };
                
                // Limpiar ID de eventos recurrentes (quitar sufijo _timestamp)
                let clean_id = if e.id.contains('_') {
                    e.id.split('_').next().unwrap_or(&e.id).to_string()
                } else {
                    e.id.clone()
                };
                
                json!({
                    "id": clean_id, // ID limpio sin sufijo de instancia
                    "original_id": e.id, // ID completo para referencia
                    "title": e.title,
                    "start": e.start,
                    "end": e.end, // Incluir end para calcular duración
                    "date": date_str, // Fecha en formato YYYY-MM-DD para fácil comparación
                    "is_all_day": e.is_all_day
                })
            })
            .collect();
        serde_json::to_string(&events_summary).unwrap_or_default()
    } else {
        String::new()
    };
    
    // Procesar la instrucción con IA (Groq/Llama)
    let parsed_event = ai::process_natural_language(
        &db, 
        &instruction, 
        &current_date_str,
        if recent_events_json.is_empty() { None } else { Some(&recent_events_json) }
    ).await?;
    
    // Manejar acciones diferentes
    // ⚠️ CRÍTICO: Si hay original_date pero action es None o "create", asumir "edit"
    // Pero si action es "delete" o "edit", respetar lo que dice la IA
    let action = match parsed_event.action.as_deref() {
        Some("delete") => "delete", // Respetar delete explícito
        Some("edit") => "edit",     // Respetar edit explícito
        Some("error") => "error",   // Respetar error
        _ => {
            // Si action es None o "create" pero hay original_date, es una edición
            if parsed_event.original_date.is_some() {
                "edit"
            } else {
                "create"
            }
        }
    };
    
    // Si la IA devolvió un error, retornarlo directamente
    if action == "error" {
        return Err(format!("⚠️ La IA necesita más información: {}", 
            parsed_event.title // Usamos title como mensaje de error en este caso
        ));
    }
    
    eprintln!("🎯 Acción determinada: {} (original_date: {:?}, event_id: {:?})", 
        action, parsed_event.original_date, parsed_event.event_id);
    
    match action {
        "edit" => {
            // Editar evento existente
            // Si hay original_date, buscar eventos en esa fecha específica
            let target_date_for_search = parsed_event.original_date.as_ref()
                .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());
            
            let current_events = if let Some(date) = target_date_for_search {
                eprintln!("🔍 Buscando eventos en fecha específica: {}", date.format("%Y-%m-%d"));
                calendar::get_events_with_range(&db, Some(date), None).await?
            } else {
                calendar::get_events(&db).await?
            };
            
            // Si hay original_date, buscar el evento por fecha primero (más confiable)
            let current_event = if let Some(ref original_date) = parsed_event.original_date {
                eprintln!("🔍 Buscando evento por fecha original: {}", original_date);
                eprintln!("📅 Recorriendo {} eventos del calendario...", current_events.len());
                
                // Normalizar la fecha original a formato YYYY-MM-DD si viene en otro formato
                let normalized_original_date = if original_date.len() == 10 && original_date.contains('-') {
                    original_date.clone()
                } else if original_date.len() == 10 && original_date.contains('/') {
                    // Formato DD/MM/YYYY o MM/DD/YYYY - intentar parsear
                    if let Ok(date) = chrono::NaiveDate::parse_from_str(original_date, "%d/%m/%Y") {
                        date.format("%Y-%m-%d").to_string()
                    } else if let Ok(date) = chrono::NaiveDate::parse_from_str(original_date, "%m/%d/%Y") {
                        date.format("%Y-%m-%d").to_string()
                    } else {
                        original_date.clone()
                    }
                } else {
                    original_date.clone()
                };
                
                eprintln!("📅 Fecha original normalizada: {}", normalized_original_date);
                
                // Buscar evento que coincida EXACTAMENTE con la fecha original
                let matching_events: Vec<_> = current_events.iter()
                    .filter(|e| {
                        let event_date = if e.is_all_day {
                            e.start.clone()
                        } else {
                            chrono::DateTime::parse_from_rfc3339(&e.start)
                                .ok()
                                .map(|dt| dt.format("%Y-%m-%d").to_string())
                                .unwrap_or_default()
                        };
                        let matches = event_date == normalized_original_date;
                        if matches {
                            eprintln!("   ✅ Coincidencia encontrada: '{}' ({}) - fecha: {}", e.title, e.id, event_date);
                        }
                        matches
                    })
                    .collect();
                
                if matching_events.is_empty() {
                    eprintln!("⚠️ No se encontró ningún evento con fecha exacta {}", normalized_original_date);
                    // Si hay event_id, intentar obtener el evento directamente por ID desde la API
                    // Esto es necesario porque el evento puede estar fuera del rango de fechas de get_events
                    if let Some(event_id) = &parsed_event.event_id {
                        // Limpiar ID de eventos recurrentes (quitar sufijo _timestamp si existe)
                        let clean_event_id = if event_id.contains('_') && event_id.matches('_').count() >= 2 {
                            // Si tiene múltiples guiones bajos, podría ser un ID de instancia recurrente
                            // Intentar obtener el ID base (antes del primer _timestamp)
                            event_id.split('_').take(2).collect::<Vec<_>>().join("_")
                        } else {
                            event_id.clone()
                        };
                        eprintln!("⚠️ Intentando buscar evento por ID: {} (limpio: {})", event_id, clean_event_id);
                        // Intentar primero con el ID limpio, luego con el original si falla
                        let event_by_id = match calendar::get_event_by_id(&db, &clean_event_id).await {
                            Ok(event) => event,
                            Err(_) => {
                                eprintln!("⚠️ No se encontró con ID limpio, intentando con ID original...");
                                calendar::get_event_by_id(&db, event_id).await?
                            }
                        };
                        // Verificar que el evento obtenido por ID tenga la fecha original
                        let event_date_str = if event_by_id.is_all_day {
                            event_by_id.start.clone()
                        } else {
                            chrono::DateTime::parse_from_rfc3339(&event_by_id.start)
                                .ok()
                                .map(|dt| dt.format("%Y-%m-%d").to_string())
                                .unwrap_or_default()
                        };
                        if event_date_str != normalized_original_date {
                            return Err(format!(
                                "⚠️ El evento con ID {} tiene fecha {}, pero se buscaba un evento con fecha {}. Por favor, verifica la fecha original mencionada.",
                                event_id, event_date_str, normalized_original_date
                            ));
                        }
                        eprintln!("✅ Evento encontrado por ID y validado por fecha: {} ({})", event_by_id.title, event_by_id.id);
                        event_by_id
                    } else {
                        return Err(format!("No se encontró ningún evento con fecha original {}. Por favor, verifica la fecha o menciona el título del evento.", normalized_original_date));
                    }
                } else if matching_events.len() == 1 {
                    let event_ref = &matching_events[0];
                    eprintln!("✅ Evento encontrado por fecha original: {} ({})", event_ref.title, event_ref.id);
                    (*event_ref).clone()
                } else {
                    // Múltiples eventos con la misma fecha
                    eprintln!("⚠️ Se encontraron {} eventos con fecha {}", matching_events.len(), normalized_original_date);
                    // Si hay un título mencionado, buscar el que coincida
                    if parsed_event.title.is_empty() {
                        return Err(format!(
                            "Se encontraron {} eventos con fecha {}. Por favor, menciona también el título del evento para identificarlo correctamente.",
                            matching_events.len(), normalized_original_date
                        ));
                    }
                    // Buscar evento que coincida con el título
                    let found_event_ref = matching_events.iter()
                        .find(|e| e.title == parsed_event.title)
                        .ok_or_else(|| format!(
                            "Se encontraron {} eventos con fecha {}, pero ninguno tiene el título '{}'. Por favor, menciona el título exacto del evento que quieres editar.",
                            matching_events.len(), normalized_original_date, parsed_event.title
                        ))?;
                    eprintln!("✅ Evento encontrado por fecha Y título: {} ({})", found_event_ref.title, found_event_ref.id);
                    (*found_event_ref).clone()
                }
            } else if let Some(event_id) = &parsed_event.event_id {
                // Si no hay original_date, buscar por event_id (primero en la lista, luego directamente en la API)
                eprintln!("✏️ Editando evento por ID: {}", event_id);
                let found_event = if let Some(event) = current_events.iter().find(|e| e.id == *event_id) {
                    event.clone()
                } else {
                    // Si no está en la lista actual, buscarlo directamente en la API
                    eprintln!("⚠️ Evento no encontrado en la lista actual, buscando directamente en la API...");
                    calendar::get_event_by_id(&db, event_id).await
                        .map_err(|e| format!("No se encontró el evento con ID {}: {}", event_id, e))?
                };
                
                // VALIDACIÓN CRÍTICA: Si la instrucción menciona "evento de fecha X" pero no hay original_date,
                // es muy probable que la IA identificó el evento incorrecto. Rechazar la operación.
                let instruction_lower = instruction.to_lowercase();
                if (instruction_lower.contains("evento de fecha") || instruction_lower.contains("evento el día") || instruction_lower.contains("evento del")) 
                    && parsed_event.original_date.is_none() 
                    && parsed_event.start_date.is_some() {
                    // La instrucción claramente menciona una fecha original pero la IA no la extrajo
                    // Esto es peligroso porque podría editar el evento incorrecto
                    return Err(format!(
                        "⚠️ La instrucción menciona 'evento de fecha' pero no se pudo identificar la fecha original del evento. Por favor, reformula la instrucción mencionando explícitamente 'evento de fecha [fecha original] moviéndolo a [fecha nueva]' o menciona el título exacto del evento."
                    ));
                }
                
                found_event
            } else {
                return Err("Para editar un evento, necesito el ID del evento o la fecha original del evento. Por favor, sé más específico.".to_string());
            };
            
            // VALIDACIÓN: Verificar que el evento identificado coincide con los criterios mencionados
            // Si se mencionó una fecha ORIGINAL del evento, verificar que el evento tenga esa fecha
            if let Some(ref original_date) = parsed_event.original_date {
                // La IA mencionó una fecha ORIGINAL, verificar que el evento tenga esa fecha
                let event_date_str = if current_event.is_all_day {
                    current_event.start.clone()
                } else {
                    chrono::DateTime::parse_from_rfc3339(&current_event.start)
                        .ok()
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_default()
                };
                
                if event_date_str != *original_date {
                    // El evento identificado NO tiene la fecha original mencionada
                    return Err(format!(
                        "⚠️ El evento identificado ('{}') tiene fecha {}, pero mencionaste que el evento original tenía fecha {}. Por favor, sé más específico mencionando el título del evento o verifica que la fecha sea correcta.",
                        current_event.title,
                        event_date_str,
                        original_date
                    ));
                }
            } else if let Some(ref mentioned_date) = parsed_event.start_date {
                // Si no hay original_date pero hay start_date, podría ser que la instrucción mencionó solo la fecha nueva
                // En este caso, intentar buscar eventos que coincidan con alguna otra pista (título, etc.)
                // Pero si no hay más información, es mejor pedir clarificación
                eprintln!("⚠️ Se mencionó una fecha nueva ({}) pero no se especificó la fecha original del evento. Verificando si el evento identificado es correcto...", mentioned_date);
            }
            
            // Si se mencionó un título específico, verificar que coincida
            // ⚠️ IMPORTANTE: Si encontramos el evento por fecha original, IGNORAR el título que la IA devolvió
            // porque la fecha es más confiable que el título (la IA puede confundirse con títulos)
            if !parsed_event.title.is_empty() && parsed_event.title != current_event.title {
                // Si encontramos el evento por fecha original, el título de la IA puede ser incorrecto
                // En ese caso, usar el título del evento encontrado y continuar
                if parsed_event.original_date.is_some() {
                    eprintln!("⚠️ La IA mencionó título '{}' pero el evento encontrado por fecha tiene título '{}'. Usando el título del evento encontrado (la fecha es más confiable).", 
                        parsed_event.title, current_event.title);
                    // Continuar con el evento encontrado, ignorando el título de la IA
                } else {
                    // Si NO encontramos por fecha original, el título debe coincidir
                    return Err(format!(
                        "⚠️ El evento identificado tiene título '{}', pero mencionaste '{}'. Por favor, verifica que sea el evento correcto o menciona el título exacto del evento que quieres editar.",
                        current_event.title,
                        parsed_event.title
                    ));
                }
            }
            
            eprintln!("✅ Evento identificado para editar: '{}' (ID: {})", current_event.title, current_event.id);
            eprintln!("📝 Datos de la IA: start_date={:?}, end_date={:?}, is_all_day={}", 
                parsed_event.start_date, parsed_event.end_date, parsed_event.is_all_day);
            
            // Construir el request de actualización combinando datos existentes con los nuevos
            let start_date = if let Some(ref date_str) = parsed_event.start_date {
                date_str.clone()
            } else {
                // Extraer fecha del evento actual
                if current_event.is_all_day {
                    current_event.start.clone()
                } else {
                    // Parsear RFC3339 y extraer fecha
                    chrono::DateTime::parse_from_rfc3339(&current_event.start)
                        .ok()
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| current_date_str.clone())
                }
            };
            
            let start_time = parsed_event.start_time.clone();
            let end_date = if let Some(ref date_str) = parsed_event.end_date {
                date_str.clone()
            } else {
                if current_event.is_all_day {
                    current_event.end.clone()
                } else {
                    chrono::DateTime::parse_from_rfc3339(&current_event.end)
                        .ok()
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| start_date.clone())
                }
            };
            
            let end_time = parsed_event.end_time.clone();
            
            // Determinar si es todo el día
            // Si la instrucción menciona "con los mismos horarios", preservar el tipo del evento original
            let preserve_timing = instruction_lower.contains("mismo") && (instruction_lower.contains("horario") || instruction_lower.contains("hora"));
            
            let is_all_day = if preserve_timing {
                // Preservar el tipo del evento original - IGNORAR lo que la IA devolvió
                eprintln!("⚠️ Preservando is_all_day del evento original: {}", current_event.is_all_day);
                current_event.is_all_day
            } else if start_time.is_none() && end_time.is_none() {
                // Si no se mencionaron horarios, usar el tipo del evento original
                current_event.is_all_day
            } else {
                // Si se mencionaron horarios, no es todo el día
                false
            };
            
            // ⚠️ CRÍTICO: Calcular y preservar la duración original del evento
            // Esto evita que el evento se expanda a "varias semanas" cuando solo se mueve la fecha
            let (start, end) = if is_all_day {
                // Para eventos de todo el día, end SIEMPRE debe ser el día siguiente a start_date
                // Ignorar end_date del evento anterior cuando se cambia la fecha de inicio
                if let Ok(start_date_parsed) = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d") {
                    // Calcular duración original del evento
                    let original_start = if current_event.is_all_day {
                        chrono::NaiveDate::parse_from_str(&current_event.start, "%Y-%m-%d").ok()
                    } else {
                        chrono::DateTime::parse_from_rfc3339(&current_event.start)
                            .ok()
                            .map(|dt| dt.date_naive())
                    };
                    let original_end = if current_event.is_all_day {
                        chrono::NaiveDate::parse_from_str(&current_event.end, "%Y-%m-%d").ok()
                    } else {
                        chrono::DateTime::parse_from_rfc3339(&current_event.end)
                            .ok()
                            .map(|dt| dt.date_naive())
                    };
                    
                    let calculated_end = if let (Some(orig_start), Some(orig_end)) = (original_start, original_end) {
                        // Calcular duración original
                        let duration = orig_end.signed_duration_since(orig_start);
                        // Aplicar la misma duración a la nueva fecha
                        let new_end = start_date_parsed + duration;
                        new_end.format("%Y-%m-%d").to_string()
                    } else {
                        // Fallback: día siguiente (duración de 1 día)
                        let next_day = start_date_parsed + chrono::Duration::days(1);
                        next_day.format("%Y-%m-%d").to_string()
                    };
                    
                    eprintln!("📏 Preservando duración: evento de todo el día, end calculado: {}", calculated_end);
                    (start_date.clone(), calculated_end)
                } else {
                    // Si no se puede parsear start_date, usar end_date solo si se especificó explícitamente
                    let calculated_end = parsed_event.end_date.as_ref()
                        .map(|d| d.clone())
                        .unwrap_or_else(|| {
                            // Calcular día siguiente desde start_date como último recurso
                            chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
                                .ok()
                                .map(|d| (d + chrono::Duration::days(1)).format("%Y-%m-%d").to_string())
                                .unwrap_or_else(|| start_date.clone())
                        });
                    (start_date.clone(), calculated_end)
                }
            } else {
                // Eventos con hora específica
                // ⚠️ CRÍTICO: Preservar la duración original del evento
                // Guardar si start_time y end_time están vacíos antes de consumirlos
                let start_time_is_none = start_time.is_none();
                let end_time_is_none = end_time.is_none();
                
                let start_time_str = start_time.unwrap_or_else(|| {
                    if !current_event.is_all_day {
                        chrono::DateTime::parse_from_rfc3339(&current_event.start)
                            .ok()
                            .map(|dt| dt.format("%H:%M").to_string())
                            .unwrap_or_else(|| "09:00".to_string())
                    } else {
                        "09:00".to_string()
                    }
                });
                
                // Calcular duración original del evento
                let original_duration = if let (Ok(orig_start), Ok(orig_end)) = (
                    chrono::DateTime::parse_from_rfc3339(&current_event.start),
                    chrono::DateTime::parse_from_rfc3339(&current_event.end)
                ) {
                    orig_end.signed_duration_since(orig_start)
                } else {
                    // Fallback: 1 hora de duración
                    chrono::Duration::hours(1)
                };
                
                eprintln!("📏 Duración original del evento: {} minutos", original_duration.num_minutes());
                
                // Construir nueva fecha/hora de inicio
                let start_datetime = chrono::NaiveDateTime::parse_from_str(
                    &format!("{} {}", start_date, start_time_str),
                    "%Y-%m-%d %H:%M"
                ).map_err(|e| format!("Error al parsear fecha/hora de inicio: {}", e))?;
                
                // Calcular nueva fecha/hora de fin aplicando la duración original
                let end_datetime = if preserve_timing || (start_time_is_none && end_time_is_none) {
                    // Preservar duración exacta: nueva_fin = nueva_inicio + duración_original
                    start_datetime + original_duration
                } else {
                    // Si se especificó un end_time explícito, usarlo
                    let end_time_str = end_time.unwrap_or_else(|| {
                        // Si no se especificó end_time, calcularlo desde la duración original
                        let calculated_end = start_datetime + original_duration;
                        calculated_end.format("%H:%M").to_string()
                    });
                    chrono::NaiveDateTime::parse_from_str(
                        &format!("{} {}", end_date, end_time_str),
                        "%Y-%m-%d %H:%M"
                    ).map_err(|e| format!("Error al parsear fecha/hora de fin: {}", e))?
                };
                
                use chrono::TimeZone;
                let start_local = tz_offset.from_local_datetime(&start_datetime)
                    .single()
                    .ok_or_else(|| "Fecha/hora de inicio inválida".to_string())?;
                
                let end_local = tz_offset.from_local_datetime(&end_datetime)
                    .single()
                    .ok_or_else(|| "Fecha/hora de fin inválida".to_string())?;
                
                eprintln!("📏 Nueva fecha/hora calculada preservando duración: {} → {}", 
                    start_local.format("%Y-%m-%d %H:%M"), 
                    end_local.format("%Y-%m-%d %H:%M"));
                
                (start_local.with_timezone(&chrono::Utc).to_rfc3339(), 
                 end_local.with_timezone(&chrono::Utc).to_rfc3339())
            };
            
            // Si el título es null o vacío, usar el título del evento actual
            // ⚠️ CRÍTICO: Si encontramos el evento por fecha original, normalmente usar el título del evento encontrado
            // PERO si la instrucción menciona explícitamente "cambiar el título" o "cambiar título a/por", usar el título de la IA
            let instruction_lower_for_title = instruction.to_lowercase();
            // Detectar si la instrucción menciona explícitamente cambiar el título
            // Buscar variantes: "cambiar/cambia el título/titulo", "título/titulo a/por", etc.
            // También buscar si hay "cambiar/cambia" Y "título/titulo" en la misma instrucción
            let has_cambiar = instruction_lower_for_title.contains("cambiar") || instruction_lower_for_title.contains("cambia");
            let has_titulo = instruction_lower_for_title.contains("título") || instruction_lower_for_title.contains("titulo");
            let has_por = instruction_lower_for_title.contains(" por ");
            let has_a = instruction_lower_for_title.contains(" a ");
            
            let explicitly_changing_title = 
                instruction_lower_for_title.contains("cambiar el título") 
                || instruction_lower_for_title.contains("cambiar título")
                || instruction_lower_for_title.contains("cambiar titulo")
                || instruction_lower_for_title.contains("cambia el título")
                || instruction_lower_for_title.contains("cambia título")
                || instruction_lower_for_title.contains("cambia titulo")
                || (has_cambiar && has_titulo) // Si tiene "cambiar/cambia" Y "título", es cambio de título
                || (has_titulo && (has_por || has_a)); // Si tiene "título" Y "por" o "a", es cambio de título
            
            eprintln!("🔍 Detección de cambio de título: explicitly_changing_title={}", explicitly_changing_title);
            eprintln!("   - tiene 'cambiar/cambia': {}, tiene 'título': {}, tiene 'por': {}, tiene 'a': {}", 
                has_cambiar, has_titulo, has_por, has_a);
            
            let title = if parsed_event.original_date.is_some() && !explicitly_changing_title {
                // Si encontramos por fecha original Y NO se menciona explícitamente cambiar el título,
                // usar el título del evento encontrado (más confiable)
                eprintln!("📌 Usando título del evento encontrado por fecha: '{}' (ignorando título de la IA: '{}')", 
                    current_event.title, parsed_event.title);
                current_event.title.clone()
            } else if parsed_event.title.is_empty() {
                current_event.title.clone()
            } else {
                // Si se menciona explícitamente cambiar el título, o si no hay fecha original, usar el título de la IA
                if explicitly_changing_title {
                    eprintln!("📌 Usando título de la IA porque se mencionó explícitamente cambiar el título: '{}'", parsed_event.title);
                }
                parsed_event.title.clone()
            };
            
            // Preservar horarios si no se mencionaron explícitamente o si se dice "mismos horarios"
            // Si el evento original tenía horarios y no se mencionaron nuevos, mantenerlos
            let (final_start, final_end, final_is_all_day) = if preserve_timing && !current_event.is_all_day {
                // La instrucción dice "mismos horarios" y el evento original NO es de todo el día
                // Preservar los horarios originales pero con la nueva fecha
                eprintln!("⚠️ Preservando horarios originales con nueva fecha");
                // Extraer hora del evento original
                if let Ok(start_dt) = chrono::DateTime::parse_from_rfc3339(&current_event.start) {
                    if let Ok(end_dt) = chrono::DateTime::parse_from_rfc3339(&current_event.end) {
                        // Calcular duración original
                        let duration = end_dt.signed_duration_since(start_dt);
                        
                        // Construir nueva fecha/hora de inicio con la fecha nueva pero horario original
                        let start_time = start_dt.format("%H:%M:%S").to_string();
                        let new_start_datetime = chrono::NaiveDateTime::parse_from_str(
                            &format!("{} {}", start_date, start_time),
                            "%Y-%m-%d %H:%M:%S"
                        );
                        
                        if let Ok(start_naive) = new_start_datetime {
                            // Calcular nueva fecha/hora de fin: nueva_inicio + duración_original
                            let end_naive = start_naive + duration;
                            
                            use chrono::TimeZone;
                            if let (Some(start_local), Some(end_local)) = (
                                tz_offset.from_local_datetime(&start_naive).single(),
                                tz_offset.from_local_datetime(&end_naive).single()
                            ) {
                                eprintln!("📏 Horarios preservados: {} → {} (duración: {} minutos)", 
                                    start_local.format("%Y-%m-%d %H:%M"), 
                                    end_local.format("%Y-%m-%d %H:%M"),
                                    duration.num_minutes());
                                (start_local.with_timezone(&chrono::Utc).to_rfc3339(), 
                                 end_local.with_timezone(&chrono::Utc).to_rfc3339(), false)
                            } else {
                                // Si falla el parseo de timezone, usar los horarios calculados anteriormente
                                (start, end, false)
                            }
                        } else {
                            // Si falla el parseo, usar los horarios calculados anteriormente
                            (start, end, false)
                        }
                    } else {
                        // Si falla el parseo del end, usar los horarios calculados anteriormente
                        (start, end, false)
                    }
                } else {
                    // Si falla el parseo del start, usar los horarios calculados anteriormente
                    (start, end, false)
                }
            } else if !current_event.is_all_day && is_all_day && parsed_event.start_time.is_none() && parsed_event.end_time.is_none() {
                // El evento original tenía horarios pero la IA no los mencionó, mantener los horarios originales
                eprintln!("⚠️ Evento original tenía horarios pero no se mencionaron nuevos, manteniendo horarios originales");
                (current_event.start.clone(), current_event.end.clone(), false)
            } else {
                (start, end, is_all_day)
            };
            
            // Preservar participantes si no se mencionaron explícitamente
            // Si la instrucción menciona "con los mismos participantes" o similar, mantener los participantes originales
            let final_attendees = if instruction.to_lowercase().contains("mismo") && instruction.to_lowercase().contains("participante") 
                || instruction.to_lowercase().contains("misma") && instruction.to_lowercase().contains("participante")
                || instruction.to_lowercase().contains("mismos") && instruction.to_lowercase().contains("participante") {
                // La instrucción menciona mantener participantes, usar los originales
                current_event.attendees.clone()
            } else if parsed_event.attendees.is_some() {
                // Se mencionaron participantes nuevos, usar los nuevos
                parsed_event.attendees.clone()
            } else {
                // No se mencionaron participantes, mantener los originales
                current_event.attendees.clone()
            };
            
            let update_request = calendar::CreateEventRequest {
                title,
                start: final_start,
                end: final_end,
                description: parsed_event.description.or(current_event.description.clone()),
                location: parsed_event.location.or(current_event.location.clone()),
                is_all_day: final_is_all_day,
                attendees: final_attendees,
            };
                
            calendar::update_event(&db, &current_event.id, update_request).await
        },
        "delete" => {
            // Eliminar evento
            if let Some(event_id) = &parsed_event.event_id {
                eprintln!("🗑️ Eliminando evento: {}", event_id);
                calendar::delete_event(&db, event_id).await?;
                return Err("Evento eliminado exitosamente".to_string()); // Retornamos error porque no hay evento para devolver
            } else {
                Err("Para eliminar un evento, se requiere el event_id. Por favor, menciona qué evento quieres eliminar.".to_string())
            }
        },
        _ => {
            // Crear nuevo evento (comportamiento por defecto)
            eprintln!("➕ Creando nuevo evento");
            
            // Validar que la fecha no sea pasada
            if let Some(ref start_date) = parsed_event.start_date {
                if let Ok(event_date) = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d") {
                    let current_date = local_now.date_naive();
                    if event_date < current_date {
                        return Err(format!(
                            "⚠️ La fecha del evento ({}) es anterior a la fecha actual ({}). Por favor, verifica la fecha o confirma si realmente quieres crear un evento en el pasado.",
                            start_date, current_date_str
                        ));
                    }
                }
            }
            
            // Determinar fecha de inicio
            let start_date = if let Some(date_str) = &parsed_event.start_date {
                date_str.clone()
            } else {
                // Si no hay fecha, usar hoy
                current_date_str.clone()
            };
            
            // Determinar fecha de fin (usar la misma si no se especifica)
            let end_date = parsed_event.end_date.unwrap_or_else(|| start_date.clone());
            
            // Construir start y end según si es todo el día o tiene hora
            let (start, end) = if parsed_event.is_all_day {
                (start_date, end_date)
            } else {
                // Para eventos con hora, construir RFC3339
                let start_time = parsed_event.start_time.unwrap_or_else(|| "09:00".to_string());
                let end_time = parsed_event.end_time.unwrap_or_else(|| "10:00".to_string());
                
                // Parsear fecha y hora como NaiveDateTime
                let start_datetime = chrono::NaiveDateTime::parse_from_str(
                    &format!("{} {}", start_date, start_time),
                    "%Y-%m-%d %H:%M"
                ).map_err(|e| format!("Error al parsear fecha/hora de inicio: {}", e))?;
                
                let end_datetime = chrono::NaiveDateTime::parse_from_str(
                    &format!("{} {}", end_date, end_time),
                    "%Y-%m-%d %H:%M"
                ).map_err(|e| format!("Error al parsear fecha/hora de fin: {}", e))?;
                
                // Convertir a la zona horaria local y luego a UTC para RFC3339
                use chrono::TimeZone;
                let start_local = tz_offset.from_local_datetime(&start_datetime)
                    .single()
                    .ok_or_else(|| "Fecha/hora de inicio inválida".to_string())?;
                
                let end_local = tz_offset.from_local_datetime(&end_datetime)
                    .single()
                    .ok_or_else(|| "Fecha/hora de fin inválida".to_string())?;
                
                // Convertir a UTC para RFC3339
                let start_utc = start_local.with_timezone(&chrono::Utc);
                let end_utc = end_local.with_timezone(&chrono::Utc);
                
                (start_utc.to_rfc3339(), end_utc.to_rfc3339())
            };
            
            // Crear el evento usando la función existente
            let create_request = calendar::CreateEventRequest {
                title: parsed_event.title,
                start,
                end,
                description: parsed_event.description,
                location: parsed_event.location,
                is_all_day: parsed_event.is_all_day,
                attendees: parsed_event.attendees,
            };
            
            calendar::create_event(&db, create_request).await
        }
    }
}
