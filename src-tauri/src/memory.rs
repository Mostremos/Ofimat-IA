#![allow(dead_code)] // Algunos métodos se reservan para uso futuro

use rusqlite::{Connection, Result as SqlResult, params};
use std::sync::Mutex;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Tipos de conocimiento que se pueden almacenar
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KnowledgeType {
    Decision,      // Decisiones de arquitectura/diseño
    BugFix,        // Bugs resueltos y soluciones
    Pattern,       // Patrones descubiertos
    Configuration, // Configuraciones importantes
    Context,       // Contexto general del proyecto
    Summary,       // Resumen de conversación
}

impl KnowledgeType {
    fn as_str(&self) -> &'static str {
        match self {
            KnowledgeType::Decision => "decision",
            KnowledgeType::BugFix => "bugfix",
            KnowledgeType::Pattern => "pattern",
            KnowledgeType::Configuration => "config",
            KnowledgeType::Context => "context",
            KnowledgeType::Summary => "summary",
        }
    }
}

/// Entrada de conocimiento almacenada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: i64,
    pub knowledge_type: String,
    pub title: String,
    pub content: String,
    pub summary: String,        // Resumen compacto (~100 tokens)
    pub tags: Vec<String>,
    pub project_path: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub relevance_score: Option<f64>, // Para búsquedas
}

/// Timeline de una entrada (eventos relacionados)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub id: i64,
    pub knowledge_id: i64,
    pub event_type: String, // "created", "updated", "referenced"
    pub description: String,
    pub timestamp: i64,
}

pub struct Memory {
    conn: Mutex<Connection>,
}

impl Memory {
    pub fn new(conn: Connection) -> SqlResult<Self> {
        let memory = Memory {
            conn: Mutex::new(conn),
        };
        memory.init_schema()?;
        Ok(memory)
    }

    fn init_schema(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        
        // Tabla principal de conocimiento con FTS5 para búsqueda full-text
        conn.execute(
            "CREATE TABLE IF NOT EXISTS knowledge (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                knowledge_type TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                summary TEXT NOT NULL,
                tags TEXT, -- JSON array
                project_path TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Tabla FTS5 para búsqueda rápida (solo índices, no duplica datos)
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS knowledge_fts USING fts5(
                title,
                content,
                summary,
                tags,
                content_rowid=id,
                content='knowledge'
            )",
            [],
        )?;

        // Tabla de timeline para Progressive Disclosure
        conn.execute(
            "CREATE TABLE IF NOT EXISTS knowledge_timeline (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                knowledge_id INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                description TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (knowledge_id) REFERENCES knowledge(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Índices para búsquedas rápidas
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_knowledge_type ON knowledge(knowledge_type)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_knowledge_project ON knowledge(project_path)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_knowledge_created ON knowledge(created_at DESC)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timeline_knowledge ON knowledge_timeline(knowledge_id, timestamp DESC)",
            [],
        )?;

        // Triggers para mantener FTS5 sincronizado
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS knowledge_fts_insert AFTER INSERT ON knowledge BEGIN
                INSERT INTO knowledge_fts(rowid, title, content, summary, tags)
                VALUES (new.id, new.title, new.content, new.summary, new.tags);
            END",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS knowledge_fts_update AFTER UPDATE ON knowledge BEGIN
                UPDATE knowledge_fts SET
                    title = new.title,
                    content = new.content,
                    summary = new.summary,
                    tags = new.tags
                WHERE rowid = new.id;
            END",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS knowledge_fts_delete AFTER DELETE ON knowledge BEGIN
                DELETE FROM knowledge_fts WHERE rowid = old.id;
            END",
            [],
        )?;

        Ok(())
    }

    /// Guarda una entrada de conocimiento
    pub fn save_knowledge(
        &self,
        knowledge_type: KnowledgeType,
        title: &str,
        content: &str,
        summary: &str,
        tags: &[String],
        project_path: Option<&str>,
    ) -> SqlResult<i64> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();
        let tags_json = serde_json::to_string(tags).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT INTO knowledge 
            (knowledge_type, title, content, summary, tags, project_path, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                knowledge_type.as_str(),
                title,
                content,
                summary,
                tags_json,
                project_path,
                now,
                now
            ],
        )?;

        let id = conn.last_insert_rowid();
        
        // Crear entrada inicial en timeline
        conn.execute(
            "INSERT INTO knowledge_timeline (knowledge_id, event_type, description, timestamp)
            VALUES (?1, 'created', 'Entrada creada', ?2)",
            params![id, now],
        )?;

        Ok(id)
    }

    /// Búsqueda compacta (Capa 1: Progressive Disclosure)
    /// Retorna solo resúmenes (~100 tokens cada uno) para ahorrar contexto
    pub fn search_compact(&self, query: &str, limit: i32) -> SqlResult<Vec<KnowledgeEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut results = Vec::new();

        // Búsqueda FTS5 con ranking
        let mut stmt = conn.prepare(
            "SELECT k.id, k.knowledge_type, k.title, k.summary, k.tags, k.project_path, 
                    k.created_at, k.updated_at,
                    bm25(knowledge_fts) as score
             FROM knowledge_fts
             JOIN knowledge k ON knowledge_fts.rowid = k.id
             WHERE knowledge_fts MATCH ?1
             ORDER BY score
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![query, limit], |row| {
            let tags_json: String = row.get(4)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            
            Ok(KnowledgeEntry {
                id: row.get(0)?,
                knowledge_type: row.get(1)?,
                title: row.get(2)?,
                content: String::new(), // No incluimos contenido en búsqueda compacta
                summary: row.get(3)?,
                tags,
                project_path: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
                relevance_score: Some(row.get(8)?),
            })
        })?;

        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Obtiene el timeline de una entrada (Capa 2: Progressive Disclosure)
    pub fn get_timeline(&self, knowledge_id: i64) -> SqlResult<Vec<TimelineEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut results = Vec::new();

        let mut stmt = conn.prepare(
            "SELECT id, knowledge_id, event_type, description, timestamp
             FROM knowledge_timeline
             WHERE knowledge_id = ?1
             ORDER BY timestamp DESC"
        )?;

        let rows = stmt.query_map(params![knowledge_id], |row| {
            Ok(TimelineEntry {
                id: row.get(0)?,
                knowledge_id: row.get(1)?,
                event_type: row.get(2)?,
                description: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?;

        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Obtiene el detalle completo de una entrada (Capa 3: Progressive Disclosure)
    pub fn get_detail(&self, knowledge_id: i64) -> SqlResult<Option<KnowledgeEntry>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, knowledge_type, title, content, summary, tags, project_path, 
                    created_at, updated_at
             FROM knowledge
             WHERE id = ?1"
        )?;

        let mut rows = stmt.query_map(params![knowledge_id], |row| {
            let tags_json: String = row.get(5)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            
            Ok(KnowledgeEntry {
                id: row.get(0)?,
                knowledge_type: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                summary: row.get(4)?,
                tags,
                project_path: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                relevance_score: None,
            })
        })?;

        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }

    /// Actualiza una entrada de conocimiento
    pub fn update_knowledge(
        &self,
        knowledge_id: i64,
        title: Option<&str>,
        content: Option<&str>,
        summary: Option<&str>,
        tags: Option<&[String]>,
    ) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();

        // Construir query dinámicamente
        let mut updates = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(t) = title {
            updates.push("title = ?");
            params_vec.push(Box::new(t));
        }
        if let Some(c) = content {
            updates.push("content = ?");
            params_vec.push(Box::new(c));
        }
        if let Some(s) = summary {
            updates.push("summary = ?");
            params_vec.push(Box::new(s));
        }
        if let Some(tags_vec) = tags {
            let tags_json = serde_json::to_string(tags_vec).unwrap_or_else(|_| "[]".to_string());
            updates.push("tags = ?");
            params_vec.push(Box::new(tags_json));
        }

        updates.push("updated_at = ?");
        params_vec.push(Box::new(now));

        if !updates.is_empty() {
            let query = format!("UPDATE knowledge SET {} WHERE id = ?", updates.join(", "));
            params_vec.push(Box::new(knowledge_id));
            
            let mut stmt = conn.prepare(&query)?;
            let params: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
            stmt.execute(params.as_slice())?;

            // Registrar en timeline
            conn.execute(
                "INSERT INTO knowledge_timeline (knowledge_id, event_type, description, timestamp)
                VALUES (?1, 'updated', 'Entrada actualizada', ?2)",
                params![knowledge_id, now],
            )?;
        }

        Ok(())
    }

    /// Elimina una entrada de conocimiento
    pub fn delete_knowledge(&self, knowledge_id: i64) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM knowledge WHERE id = ?1", params![knowledge_id])?;
        Ok(())
    }

    /// Busca conocimiento por tipo
    pub fn get_by_type(&self, knowledge_type: KnowledgeType, limit: i32) -> SqlResult<Vec<KnowledgeEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut results = Vec::new();

        let mut stmt = conn.prepare(
            "SELECT id, knowledge_type, title, content, summary, tags, project_path, 
                    created_at, updated_at
             FROM knowledge
             WHERE knowledge_type = ?1
             ORDER BY created_at DESC
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![knowledge_type.as_str(), limit], |row| {
            let tags_json: String = row.get(5)?;
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            
            Ok(KnowledgeEntry {
                id: row.get(0)?,
                knowledge_type: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                summary: row.get(4)?,
                tags,
                project_path: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
                relevance_score: None,
            })
        })?;

        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }
}
