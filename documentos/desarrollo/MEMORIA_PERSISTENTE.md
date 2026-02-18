# Sistema de Memoria Persistente Condensada

## ¿Qué es y por qué lo necesitamos?

El sistema de memoria persistente permite que la IA "recuerde" información importante entre sesiones sin consumir tokens excesivos. En lugar de guardar conversaciones completas (que ocupan mucho contexto), guardamos:

1. **Resúmenes condensados** (~100 tokens cada uno)
2. **Conocimiento estructurado**: decisiones, bugs resueltos, patrones descubiertos
3. **Búsqueda eficiente** con Progressive Disclosure (3 capas)

## Cómo Funciona

### Arquitectura

```
┌─────────────────────────────────────────┐
│  Conversación Completa (muchos tokens) │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│  Condensador de Memoria                 │
│  - Extrae resúmenes (~100 tokens)       │
│  - Identifica conocimiento importante   │
│  - Genera tags automáticos              │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│  Base de Datos SQLite + FTS5            │
│  - knowledge (entradas estructuradas)   │
│  - knowledge_fts (índice de búsqueda)  │
│  - knowledge_timeline (historial)       │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│  Búsqueda con Progressive Disclosure    │
│  1. Resumen compacto (~100 tokens)      │
│  2. Timeline (eventos relacionados)     │
│  3. Detalle completo (solo si necesario)│
└─────────────────────────────────────────┘
```

### Tipos de Conocimiento

El sistema almacena diferentes tipos de conocimiento:

- **Decision**: Decisiones de arquitectura o diseño importantes
- **BugFix**: Bugs resueltos y sus soluciones
- **Pattern**: Patrones descubiertos durante el desarrollo
- **Configuration**: Configuraciones importantes del proyecto
- **Context**: Contexto general del proyecto
- **Summary**: Resumen de conversaciones completas

### Progressive Disclosure (3 Capas)

Para ahorrar tokens, el sistema usa un enfoque de "revelación progresiva":

#### Capa 1: Búsqueda Compacta
- Retorna solo resúmenes (~100 tokens cada uno)
- Incluye título, resumen, tags y score de relevancia
- **Uso**: Búsqueda inicial para encontrar información relevante

#### Capa 2: Timeline
- Muestra eventos relacionados (creación, actualizaciones, referencias)
- Permite entender el contexto temporal
- **Uso**: Cuando necesitas ver la historia de una entrada

#### Capa 3: Detalle Completo
- Contenido completo de la entrada
- Solo se carga cuando realmente se necesita
- **Uso**: Cuando necesitas todos los detalles

## Uso Práctico

### Guardar Conocimiento Automáticamente

El sistema puede detectar automáticamente conocimiento importante:

```rust
// Después de una conversación importante, se puede llamar:
memory_condenser::condense_conversation(
    &db,
    &conversation_text,
    Some(&project_path)
).await?;

// Esto extrae automáticamente:
// - Resumen de la conversación
// - Decisiones de arquitectura
// - Bugs resueltos
// - Patrones descubiertos
```

### Búsqueda de Conocimiento

```rust
// Búsqueda compacta (Capa 1)
let results = memory_condenser::search_relevant_knowledge(
    &db,
    "cómo manejar eventos recurrentes en Google Calendar",
    5  // límite de resultados
)?;

// Cada resultado tiene:
// - title: Título descriptivo
// - summary: Resumen compacto (~100 tokens)
// - tags: Tags relevantes
// - relevance_score: Score de relevancia

// Si necesitas más detalle (Capa 3):
let detail = memory.get_detail(result.id)?;
```

### Integración con la IA

Cuando la IA necesita contexto, puede buscar conocimiento relevante:

```rust
// Antes de procesar una instrucción, buscar contexto relevante
let context = memory_condenser::search_relevant_knowledge(
    &db,
    &instruction,
    3  // Solo los 3 más relevantes
)?;

// Construir contexto compacto para la IA
let context_text = context.iter()
    .map(|k| format!("[{}] {}\n{}", k.knowledge_type, k.title, k.summary))
    .collect::<Vec<_>>()
    .join("\n\n");

// Usar en el prompt de la IA (solo ~300 tokens en lugar de miles)
```

## Ventajas

1. **Ahorro de Tokens**: En lugar de cargar conversaciones completas (miles de tokens), solo cargas resúmenes (~100 tokens cada uno)

2. **Búsqueda Eficiente**: FTS5 permite búsquedas rápidas y relevantes

3. **Memoria Persistente**: El conocimiento se mantiene entre sesiones

4. **Progressive Disclosure**: Solo cargas lo que necesitas, cuando lo necesitas

5. **Estructuración**: El conocimiento está organizado por tipo, facilitando la búsqueda

6. **Sin Dependencias Externas**: Todo funciona con SQLite (ya incluido en el proyecto)

## Próximos Pasos

1. **Integración Automática**: Hacer que el sistema guarde automáticamente conocimiento importante después de cada conversación relevante

2. **Mejora de Extracción**: Usar la IA (Groq) para generar mejores resúmenes y extraer conocimiento más preciso

3. **Sincronización Git**: Implementar sincronización de conocimiento entre proyectos (similar a Engram)

4. **Interfaz de Usuario**: Crear una UI para visualizar y gestionar el conocimiento almacenado

5. **Filtrado por Proyecto**: Mejorar el filtrado automático por proyecto usando el `project_path`

## Referencias

Este sistema está inspirado en [Engram](https://github.com/Gentleman-Programming/engram), pero adaptado para nuestro proyecto específico usando Rust y SQLite.

## Estructura de la Base de Datos

```sql
-- Tabla principal de conocimiento
CREATE TABLE knowledge (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    knowledge_type TEXT NOT NULL,  -- "decision", "bugfix", "pattern", etc.
    title TEXT NOT NULL,
    content TEXT NOT NULL,         -- Contenido completo
    summary TEXT NOT NULL,          -- Resumen compacto (~100 tokens)
    tags TEXT,                      -- JSON array de tags
    project_path TEXT,              -- Ruta del proyecto (opcional)
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Tabla FTS5 para búsqueda rápida
CREATE VIRTUAL TABLE knowledge_fts USING fts5(
    title,
    content,
    summary,
    tags,
    content_rowid=id,
    content='knowledge'
);

-- Tabla de timeline para Progressive Disclosure
CREATE TABLE knowledge_timeline (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    knowledge_id INTEGER NOT NULL,
    event_type TEXT NOT NULL,      -- "created", "updated", "referenced"
    description TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    FOREIGN KEY (knowledge_id) REFERENCES knowledge(id) ON DELETE CASCADE
);
```

## Ejemplo de Uso Completo

```rust
use crate::memory_condenser;
use crate::database::Database;

// 1. Después de una conversación importante
let conversation = "Usuario: Necesito cambiar la fecha de un evento...
AI: Para cambiar la fecha de un evento, necesitas...
[conversación completa]";

// 2. Condensar y guardar
let summary_id = memory_condenser::condense_conversation(
    &db,
    &conversation,
    Some("/ruta/al/proyecto")
).await?;

// 3. Extraer conocimiento específico
let knowledge_ids = memory_condenser::extract_knowledge(
    &db,
    &conversation,
    Some("/ruta/al/proyecto")
).await?;

// 4. Más tarde, buscar contexto relevante
let relevant = memory_condenser::search_relevant_knowledge(
    &db,
    "cómo cambiar fecha de evento",
    3
)?;

// 5. Usar en contexto de la IA
for entry in relevant {
    println!("[{}] {}", entry.knowledge_type, entry.title);
    println!("{}", entry.summary);  // Solo ~100 tokens
}
```
