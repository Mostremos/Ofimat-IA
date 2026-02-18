//! Módulo para condensar conversaciones y extraer conocimiento importante
//! 
//! Este módulo usa la IA para generar resúmenes automáticos de conversaciones
//! y extraer conocimiento valioso (decisiones, bugs resueltos, patrones) que
//! se almacena en la base de datos de memoria persistente.

#![allow(dead_code)] // Funciones preparadas para uso futuro

use crate::database::Database;
use crate::memory::KnowledgeType;

/// Condensa una conversación completa en un resumen compacto usando IA
/// 
/// El resumen se guarda como entrada de tipo `Summary` en la memoria persistente.
/// Esto permite recuperar contexto relevante sin cargar toda la conversación.
pub async fn condense_conversation(
    db: &Database,
    conversation_text: &str,
    project_path: Option<&str>,
) -> Result<i64, String> {
    // Usar la IA para generar un resumen compacto (~100 tokens)
    let summary_prompt = format!(
        "Resume la siguiente conversación en máximo 100 palabras, destacando:\n\
        1. El problema o tarea principal\n\
        2. La solución implementada\n\
        3. Decisiones técnicas importantes\n\
        4. Cualquier bug resuelto o patrón descubierto\n\n\
        Conversación:\n{}\n\n\
        Resumen:",
        conversation_text
    );

    // Llamar a la IA para generar el resumen
    // Nota: Esto requiere una función helper que use Groq para generar texto libre
    // Por ahora, usamos un resumen simple basado en longitud
    let summary = generate_summary_with_ai(&summary_prompt).await
        .unwrap_or_else(|| generate_simple_summary(conversation_text));

    // Extraer tags automáticamente
    let tags = extract_tags(conversation_text);

    // Guardar en memoria persistente
    let memory_guard = db.get_memory()
        .ok_or("No se pudo acceder al módulo de memoria")?;
    
    let memory = memory_guard.as_ref()
        .ok_or("Módulo de memoria no inicializado")?;

    let title = extract_title(conversation_text);
    
    memory.save_knowledge(
        KnowledgeType::Summary,
        &title,
        conversation_text,
        &summary,
        &tags,
        project_path,
    ).map_err(|e| format!("Error al guardar resumen: {}", e))
}

/// Extrae conocimiento importante de una conversación (decisiones, bugs, patrones)
/// 
/// Analiza la conversación y crea entradas separadas para cada tipo de conocimiento
/// encontrado, facilitando la búsqueda posterior.
pub async fn extract_knowledge(
    db: &Database,
    conversation_text: &str,
    project_path: Option<&str>,
) -> Result<Vec<i64>, String> {
    let memory_guard = db.get_memory()
        .ok_or("No se pudo acceder al módulo de memoria")?;
    
    let memory = memory_guard.as_ref()
        .ok_or("Módulo de memoria no inicializado")?;

    let mut knowledge_ids = Vec::new();

    // Detectar decisiones de arquitectura
    if conversation_text.contains("decidimos") || 
       conversation_text.contains("arquitectura") ||
       conversation_text.contains("diseño") ||
       conversation_text.contains("implementamos") {
        if let Some((title, content, summary)) = extract_knowledge_entry(
            conversation_text,
            "decisión de arquitectura",
            "decision"
        ).await {
            let tags = vec!["arquitectura".to_string(), "decisión".to_string()];
            if let Ok(id) = memory.save_knowledge(
                KnowledgeType::Decision,
                &title,
                &content,
                &summary,
                &tags,
                project_path,
            ) {
                knowledge_ids.push(id);
            }
        }
    }

    // Detectar bugs resueltos
    if conversation_text.contains("bug") ||
       conversation_text.contains("error") ||
       conversation_text.contains("solucionamos") ||
       conversation_text.contains("corregimos") {
        if let Some((title, content, summary)) = extract_knowledge_entry(
            conversation_text,
            "bug resuelto",
            "bugfix"
        ).await {
            let tags = vec!["bug".to_string(), "solución".to_string()];
            if let Ok(id) = memory.save_knowledge(
                KnowledgeType::BugFix,
                &title,
                &content,
                &summary,
                &tags,
                project_path,
            ) {
                knowledge_ids.push(id);
            }
        }
    }

    // Detectar patrones
    if conversation_text.contains("patrón") ||
       conversation_text.contains("pattern") ||
       conversation_text.contains("enfoque") {
        if let Some((title, content, summary)) = extract_knowledge_entry(
            conversation_text,
            "patrón descubierto",
            "pattern"
        ).await {
            let tags = vec!["patrón".to_string()];
            if let Ok(id) = memory.save_knowledge(
                KnowledgeType::Pattern,
                &title,
                &content,
                &summary,
                &tags,
                project_path,
            ) {
                knowledge_ids.push(id);
            }
        }
    }

    Ok(knowledge_ids)
}

/// Genera un resumen usando IA (requiere implementación con Groq)
async fn generate_summary_with_ai(_prompt: &str) -> Option<String> {
    // TODO: Implementar llamada a Groq API para generar resumen
    // Por ahora retornamos None para usar el resumen simple
    None
}

/// Genera un resumen simple basado en las primeras líneas
fn generate_simple_summary(text: &str) -> String {
    let lines: Vec<&str> = text.lines().take(3).collect();
    let summary = lines.join(" ");
    if summary.len() > 200 {
        format!("{}...", &summary[..200])
    } else {
        summary
    }
}

/// Extrae tags relevantes de una conversación
fn extract_tags(text: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let text_lower = text.to_lowercase();
    
    // Tags comunes basados en palabras clave
    if text_lower.contains("calendar") || text_lower.contains("calendario") {
        tags.push("calendario".to_string());
    }
    if text_lower.contains("evento") || text_lower.contains("event") {
        tags.push("eventos".to_string());
    }
    if text_lower.contains("oauth") || text_lower.contains("autenticación") {
        tags.push("oauth".to_string());
    }
    if text_lower.contains("rust") {
        tags.push("rust".to_string());
    }
    if text_lower.contains("sqlite") {
        tags.push("sqlite".to_string());
    }
    if text_lower.contains("tauri") {
        tags.push("tauri".to_string());
    }
    
    tags
}

/// Extrae un título descriptivo de la conversación
fn extract_title(text: &str) -> String {
    // Buscar la primera línea que parezca un título o pregunta
    for line in text.lines().take(5) {
        let line = line.trim();
        if !line.is_empty() && line.len() < 100 {
            // Si parece un título o pregunta, usarlo
            if line.ends_with('?') || 
               (line.len() > 10 && line.len() < 80 && !line.starts_with("User:") && !line.starts_with("AI:")) {
                return line.to_string();
            }
        }
    }
    
    // Fallback: usar las primeras palabras
    let words: Vec<&str> = text.split_whitespace().take(5).collect();
    format!("{}...", words.join(" "))
}

/// Extrae una entrada de conocimiento específica usando IA
async fn extract_knowledge_entry(
    text: &str,
    knowledge_type: &str,
    _category: &str,
) -> Option<(String, String, String)> {
    // Por ahora, extracción simple
    // TODO: Usar IA para extraer mejor el conocimiento específico
    
    let title = format!("{}: {}", knowledge_type, extract_title(text));
    let content = text.to_string();
    let summary = generate_simple_summary(text);
    
    Some((title, content, summary))
}

/// Busca conocimiento relevante para una consulta
/// 
/// Usa Progressive Disclosure: primero busca resúmenes compactos,
/// luego permite profundizar si es necesario.
pub fn search_relevant_knowledge(
    db: &Database,
    query: &str,
    limit: i32,
) -> Result<Vec<crate::memory::KnowledgeEntry>, String> {
    let memory_guard = db.get_memory()
        .ok_or("No se pudo acceder al módulo de memoria")?;
    
    let memory = memory_guard.as_ref()
        .ok_or("Módulo de memoria no inicializado")?;

    memory.search_compact(query, limit)
        .map_err(|e| format!("Error en búsqueda: {}", e))
}
