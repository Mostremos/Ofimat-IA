# 🛠️ Documentación de Desarrollo

Documentación técnica para desarrolladores del proyecto.

## Contenido

- Arquitectura del sistema
- Guías de desarrollo
- Referencias de API
- Estándares de código
- Procesos de build y deploy

## Estructura del Proyecto

```
Code/
├── src/                  # Frontend React/TypeScript
├── src-tauri/           # Backend Rust
│   ├── src/
│   │   ├── main.rs      # Punto de entrada
│   │   ├── database.rs  # Gestión SQLite
│   │   ├── google_auth.rs # OAuth2
│   │   └── calendar.rs  # Google Calendar API
├── data/                # Base de datos SQLite (portable)
└── documentos/          # Esta documentación
```

## Tecnologías

- **Frontend:** React + TypeScript + Vite
- **Backend:** Rust + Tauri
- **Base de datos:** SQLite
- **APIs:** Google Calendar, Gmail, Drive, Outlook, WhatsApp
