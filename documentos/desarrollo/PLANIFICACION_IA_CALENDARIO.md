# Planificación: Integración de IA con Calendario

## Objetivo
Permitir que una IA (Gemini, GPT, etc.) pueda leer, analizar y gestionar eventos del calendario de forma inteligente, incluyendo:
- Detectar vencimientos y fechas importantes
- Crear eventos automáticamente desde documentación
- Procesar instrucciones del usuario en lenguaje natural

## Funcionalidades Propuestas

### 1. Lectura y Análisis del Calendario

#### 1.1 Detección de Vencimientos
- **Objetivo**: La IA debe poder identificar eventos que representan vencimientos o deadlines
- **Implementación**:
  - Agregar campo `event_type` o `tags` a los eventos (ej: "vencimiento", "deadline", "recordatorio")
  - La IA puede analizar títulos y descripciones para detectar patrones
  - Crear comando Tauri: `analyze_calendar_for_deadlines(days_ahead: i32) -> Vec<DeadlineEvent>`
  - La IA puede usar este comando para obtener eventos próximos y analizarlos

#### 1.2 Análisis de Eventos Recurrentes
- Detectar patrones en eventos recurrentes
- Identificar conflictos de horarios
- Sugerir optimizaciones de agenda

### 2. Creación Automática de Eventos

#### 2.1 Desde Google Drive
- **Objetivo**: La IA lee documentos en Drive y extrae fechas/eventos
- **Implementación**:
  - Integrar Google Drive API (ya tenemos OAuth)
  - Comando: `search_drive_for_dates(query: String) -> Vec<ExtractedDate>`
  - La IA puede procesar documentos y crear eventos automáticamente
  - Ejemplo: "Encuentra todos los documentos que mencionen fechas de vencimiento"

#### 2.2 Desde Emails (Gmail)
- **Objetivo**: Extraer eventos de emails (confirmaciones, invitaciones, etc.)
- **Implementación**:
  - Integrar Gmail API (ya tenemos OAuth con scope `gmail.readonly`)
  - Comando: `extract_events_from_emails(days: i32) -> Vec<EmailEvent>`
  - La IA puede leer emails y crear eventos automáticamente

#### 2.3 Desde Instrucciones del Usuario
- **Objetivo**: Procesar instrucciones en lenguaje natural
- **Implementación**:
  - Comando: `create_event_from_natural_language(instruction: String) -> CalendarEvent`
  - La IA procesa la instrucción y crea el evento
  - Ejemplo: "Agrega una reunión con Juan el próximo martes a las 3pm"

### 3. Notificaciones Inteligentes

#### 3.1 Recordatorios Proactivos
- La IA puede analizar el calendario y enviar recordatorios personalizados
- Considerar patrones de comportamiento del usuario
- Ajustar recordatorios según la importancia del evento

#### 3.2 Análisis de Conflictos
- Detectar conflictos de horarios
- Sugerir alternativas
- Alertar sobre sobrecarga de eventos

## Arquitectura Propuesta

### Capa de Comandos Tauri (Backend Rust)

```rust
// Nuevos comandos a agregar
#[tauri::command]
async fn analyze_calendar_deadlines(
    db: tauri::State<'_, Database>,
    days_ahead: i32
) -> Result<Vec<DeadlineEvent>, String>

#[tauri::command]
async fn search_drive_for_dates(
    db: tauri::State<'_, Database>,
    query: String
) -> Result<Vec<ExtractedDate>, String>

#[tauri::command]
async fn extract_events_from_emails(
    db: tauri::State<'_, Database>,
    days: i32
) -> Result<Vec<EmailEvent>, String>

#[tauri::command]
async fn create_event_from_natural_language(
    db: tauri::State<'_, Database>,
    instruction: String,
    ai_provider: String // "gemini", "openai", etc.
) -> Result<CalendarEvent, String>
```

### Capa de IA (Abstracción)

```rust
// src-tauri/src/ai/mod.rs
pub trait AIProvider {
    async fn process_calendar_instruction(&self, instruction: String) -> Result<CalendarEvent, String>;
    async fn analyze_deadlines(&self, events: Vec<CalendarEvent>) -> Result<Vec<DeadlineEvent>, String>;
    async fn extract_dates_from_text(&self, text: String) -> Result<Vec<ExtractedDate>, String>;
}

// Implementaciones
pub struct GeminiProvider { ... }
pub struct OpenAIProvider { ... }
```

### Flujo de Trabajo

1. **Usuario da instrucción** → Frontend captura
2. **Frontend invoca comando Tauri** → `create_event_from_natural_language`
3. **Backend procesa con IA** → Gemini/GPT analiza la instrucción
4. **IA extrae información** → Fecha, hora, participantes, descripción
5. **Backend crea evento** → Usa `create_calendar_event` existente
6. **Frontend actualiza** → Muestra el nuevo evento

## Integración con Google Drive

### Pasos para Implementar

1. **Verificar scopes OAuth**:
   - Ya tenemos: `calendar`, `gmail.readonly`, `drive.readonly`
   - Necesitamos: `drive.readonly` (ya está) o `drive` para crear archivos

2. **Implementar módulo Drive**:
   ```rust
   // src-tauri/src/drive.rs
   pub async fn search_files(db: &Database, query: String) -> Result<Vec<DriveFile>, String>
   pub async fn read_file_content(db: &Database, file_id: String) -> Result<String, String>
   pub async fn extract_dates_from_document(db: &Database, file_id: String) -> Result<Vec<ExtractedDate>, String>
   ```

3. **Procesamiento de Documentos**:
   - PDFs: Usar librería para extraer texto
   - Google Docs: Usar Google Docs API
   - Word/Excel: Convertir a texto y procesar

## Integración con Gmail

### Pasos para Implementar

1. **Verificar scopes**:
   - Ya tenemos: `gmail.readonly`

2. **Implementar módulo Gmail**:
   ```rust
   // src-tauri/src/gmail.rs
   pub async fn search_emails(db: &Database, query: String) -> Result<Vec<Email>, String>
   pub async fn extract_events_from_email(db: &Database, email_id: String) -> Result<Vec<EmailEvent>, String>
   ```

3. **Procesamiento de Emails**:
   - Buscar emails con palabras clave: "reunión", "evento", "cita", etc.
   - Extraer fechas y horas del contenido
   - Crear eventos automáticamente

## Procesamiento de Lenguaje Natural

### Ejemplos de Instrucciones

- "Agrega una reunión con María el próximo martes a las 3pm"
- "Crea un evento de todo el día para el cumpleaños de Juan el 15 de marzo"
- "Programa una llamada con el equipo el viernes a las 10am, invita a todos"
- "Revisa mi calendario y dime qué tengo esta semana"
- "¿Cuándo es mi próximo vencimiento?"

### Implementación

1. **Prompt Engineering**:
   - Crear prompts estructurados para la IA
   - Incluir contexto del calendario actual
   - Formato de respuesta estructurado (JSON)

2. **Validación**:
   - Verificar que la IA extrajo información válida
   - Confirmar con el usuario antes de crear eventos críticos

## Base de Datos - Nuevas Tablas

```sql
-- Eventos creados por IA
CREATE TABLE ai_created_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT NOT NULL, -- ID del evento en Google Calendar
    ai_provider TEXT NOT NULL, -- "gemini", "openai", etc.
    instruction TEXT NOT NULL, -- Instrucción original del usuario
    created_at INTEGER NOT NULL,
    FOREIGN KEY (event_id) REFERENCES calendar_events(id)
);

-- Fechas extraídas de documentos
CREATE TABLE extracted_dates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_type TEXT NOT NULL, -- "drive", "email", "user_instruction"
    source_id TEXT, -- ID del documento/email
    extracted_date TEXT NOT NULL, -- Fecha extraída
    context TEXT, -- Contexto donde se encontró
    created_at INTEGER NOT NULL
);

-- Análisis de vencimientos
CREATE TABLE deadline_analysis (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT NOT NULL,
    deadline_type TEXT, -- "documento", "pago", "tarea", etc.
    priority INTEGER, -- 1-5
    analyzed_at INTEGER NOT NULL,
    FOREIGN KEY (event_id) REFERENCES calendar_events(id)
);
```

## Prioridades de Implementación

### Fase 1: Fundamentos (Actual)
- ✅ OAuth con Google
- ✅ CRUD de eventos en Calendar
- ✅ Invitados a eventos
- ✅ Notificaciones por email

### Fase 2: Lectura Inteligente
- [ ] Comando para analizar calendario (vencimientos)
- [ ] Integración básica con IA (Gemini)
- [ ] Procesamiento de lenguaje natural simple

### Fase 3: Extracción Automática
- [ ] Integración con Google Drive
- [ ] Extracción de fechas de documentos
- [ ] Integración con Gmail
- [ ] Extracción de eventos de emails

### Fase 4: Automatización Avanzada
- [ ] Creación automática de eventos desde documentos
- [ ] Análisis predictivo de vencimientos
- [ ] Sugerencias inteligentes de horarios
- [ ] Recordatorios proactivos

## Consideraciones Técnicas

### Límites de API
- Google Calendar API: 1,000,000 requests/día
- Google Drive API: 1,000 requests/100 segundos/usuario
- Gmail API: 1,000,000 requests/día
- Gemini API: Ver límites de la suscripción

### Costos
- Gemini: Incluido en Google Workspace Pro
- OpenAI: Pay-per-use
- Considerar caché de respuestas de IA

### Privacidad
- Todos los datos se procesan localmente cuando es posible
- Tokens OAuth almacenados localmente en SQLite
- Considerar encriptación de datos sensibles

## Próximos Pasos

1. **Implementar notificaciones por email** (en progreso)
2. **Agregar comando básico de análisis de calendario**
3. **Integrar Gemini para procesamiento de lenguaje natural**
4. **Crear abstracción de proveedores de IA**
5. **Implementar integración con Google Drive**
