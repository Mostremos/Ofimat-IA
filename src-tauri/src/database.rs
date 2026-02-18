#![allow(dead_code)] // Algunos métodos y campos se reservan para uso futuro

use rusqlite::{Connection, Result as SqlResult};
use std::sync::Mutex;
use chrono::Utc;

use crate::memory::Memory;

pub struct Database {
    conn: Mutex<Connection>,
    memory: Mutex<Option<Memory>>,
}

impl Database {
    pub fn new(path: &std::path::Path) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        let conn_for_memory = Connection::open(path)?; // SQLite permite múltiples conexiones
        
        // Inicializar módulo de memoria
        let memory = match Memory::new(conn_for_memory) {
            Ok(m) => Some(m),
            Err(e) => {
                eprintln!("Advertencia: No se pudo inicializar módulo de memoria: {}", e);
                None
            }
        };
        
        let db = Database {
            conn: Mutex::new(conn),
            memory: Mutex::new(memory),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        
        // Tabla para tokens OAuth
        conn.execute(
            "CREATE TABLE IF NOT EXISTS oauth_tokens (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                service TEXT NOT NULL UNIQUE,
                access_token TEXT NOT NULL,
                refresh_token TEXT,
                expires_at INTEGER,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Tabla para eventos de calendario (cache local)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS calendar_events (
                id TEXT PRIMARY KEY,
                service TEXT NOT NULL,
                title TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                description TEXT,
                synced INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Tabla para cola de sincronización offline
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                service TEXT NOT NULL,
                action TEXT NOT NULL,
                data TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                retry_count INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Tabla para conversaciones con IA (memoria)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_message TEXT NOT NULL,
                ai_response TEXT NOT NULL,
                context TEXT,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Tabla para configuración de la app
        conn.execute(
            "CREATE TABLE IF NOT EXISTS app_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    pub fn save_token(
        &self,
        service: &str,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_at: Option<i64>,
    ) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO oauth_tokens 
            (service, access_token, refresh_token, expires_at, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            [service, access_token, refresh_token.unwrap_or(""), &expires_at.unwrap_or(0).to_string(), &now.to_string()],
        )?;

        Ok(())
    }

    pub fn get_token(&self, service: &str) -> SqlResult<Option<TokenData>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT access_token, refresh_token, expires_at FROM oauth_tokens WHERE service = ?1"
        )?;
        
        let token_iter = stmt.query_map([service], |row| {
            Ok(TokenData {
                access_token: row.get(0)?,
                refresh_token: row.get(1)?,
                expires_at: row.get(2)?,
            })
        })?;

        for token in token_iter {
            return Ok(Some(token?));
        }
        Ok(None)
    }

    pub fn delete_token(&self, service: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM oauth_tokens WHERE service = ?1",
            [service],
        )?;
        Ok(())
    }

    // Reservado para uso futuro - permite acceso directo a la conexión si es necesario
    #[allow(dead_code)]
    pub fn get_connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }

    /// Obtiene el módulo de memoria persistente
    pub fn get_memory(&self) -> Option<std::sync::MutexGuard<'_, Option<Memory>>> {
        Some(self.memory.lock().unwrap())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Los campos se usan en calendar.rs y google_auth.rs, pero el linter no los detecta
pub struct TokenData {
    pub access_token: String, // Se usa en calendar.rs línea 64 y google_auth.rs líneas 41, 301
    pub refresh_token: String, // Se usa en google_auth.rs líneas 42, 302 para refresh tokens
    pub expires_at: i64,
}

pub async fn has_valid_token(db: &Database) -> Result<bool, String> {
    match db.get_token("google") {
        Ok(Some(token)) => {
            // Verificar si el token no ha expirado
            let now = Utc::now().timestamp();
            Ok(token.expires_at == 0 || token.expires_at > now)
        }
        Ok(None) => Ok(false),
        Err(e) => Err(format!("Error al verificar token: {}", e)),
    }
}
