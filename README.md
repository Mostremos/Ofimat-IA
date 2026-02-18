# Ofimat-IA

Aplicación desktop de automatización de oficina con asistencia de IA y memoria persistente.
En las primeras versiones, funciona mediante API de Google Calendar, Drive, y Gmail.
Desarrollado en Rust, Tauri, y SQLite.

Asistente ofimático con IA para automatización de tareas con Google Calendar, Gmail, Google Drive, Outlook y WhatsApp.

## 🚀 Características

- ✅ Integración con Google Calendar
- 🔄 Funcionamiento offline con sincronización
- 💾 Base de datos SQLite local
- 🔐 Autenticación OAuth2 segura
- 📱 Interfaz de escritorio con Tauri

## 📋 Requisitos Previos

### Windows
- [Node.js](https://nodejs.org/) (v18 o superior)
- [Rust](https://www.rust-lang.org/tools/install) (última versión estable)
- [Microsoft Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (para compilar Rust en Windows)

### Verificar instalación
```bash
node --version
npm --version
rustc --version
cargo --version
```

## 🛠️ Instalación

1. **Instalar dependencias de Node.js:**
```bash
npm install
```

2. **Configurar credenciales de Google OAuth2:**
   - Ve a [Google Cloud Console](https://console.cloud.google.com/)
   - Crea un nuevo proyecto o selecciona uno existente
   - Habilita las APIs necesarias:
     - Google Calendar API
     - Gmail API
     - Google Drive API
   - Crea credenciales OAuth 2.0:
     - Tipo: Aplicación de escritorio
     - URI de redirección: `http://127.0.0.1:1420/oauth/callback`
   - Copia el Client ID y Client Secret

3. **Configurar variables de entorno:**
   - Copia el archivo `.env.example` a `.env`:
     ```bash
     cp .env.example .env
     ```
   - Edita el archivo `.env` y reemplaza los valores:
     - `GOOGLE_CLIENT_ID`: Tu Client ID de Google
     - `GOOGLE_CLIENT_SECRET`: Tu Client Secret de Google
     - `GROQ_API_KEY`: (Opcional) Tu API key de Groq para procesamiento de lenguaje natural
   - ⚠️ **IMPORTANTE**: Nunca subas el archivo `.env` a Git. Está en `.gitignore` por seguridad.

## 🏃 Ejecutar en Desarrollo

```bash
npm run dev
```

Esto iniciará:
- El servidor de desarrollo de Vite (frontend)
- La aplicación Tauri (desktop)

## 📦 Compilar para Producción

```bash
npm run build
```

El ejecutable se generará en `src-tauri/target/release/`

## 📁 Estructura del Proyecto

```
Code/
├── src/                    # Frontend React/TypeScript
│   ├── App.tsx            # Componente principal
│   └── main.tsx           # Punto de entrada
├── src-tauri/             # Backend Rust
│   ├── src/
│   │   ├── main.rs        # Punto de entrada Tauri
│   │   ├── database.rs    # Gestión de SQLite
│   │   ├── google_auth.rs # Autenticación OAuth2
│   │   └── calendar.rs    # Integración Google Calendar
│   └── Cargo.toml         # Dependencias Rust
├── data/                   # Base de datos SQLite (se crea automáticamente)
├── package.json
└── README.md
```

## 🔧 Próximos Pasos

### Fase 1: Completar OAuth2
- [ ] Implementar servidor HTTP local para recibir callback OAuth
- [ ] Guardar tokens de forma segura
- [ ] Implementar refresh token automático

### Fase 2: Google Calendar
- [ ] Conectar con Google Calendar API
- [ ] Leer eventos reales
- [ ] Crear eventos
- [ ] Sincronización offline

### Fase 3: Otras Integraciones
- [ ] Gmail
- [ ] Google Drive
- [ ] Outlook
- [ ] WhatsApp

## 🐛 Solución de Problemas

### Error: "No se puede compilar Rust"
- Asegúrate de tener Visual C++ Build Tools instalado
- Ejecuta: `rustup update`

### Error: "Puerto 1420 en uso"
- Cambia el puerto en `vite.config.ts` y `tauri.conf.json`

### Error: "No se puede conectar con Google"
- Verifica que las credenciales OAuth2 estén correctas
- Asegúrate de que las APIs estén habilitadas en Google Cloud Console

## 📚 Documentación

Toda la documentación está organizada en el directorio [`documentos/`](documentos/):

- **Instalación y Configuración:** [`documentos/install/`](documentos/install/)
- **Manuales de Usuario:** [`documentos/manuales/`](documentos/manuales/)
- **Desarrollo:** [`documentos/desarrollo/`](documentos/desarrollo/)
- **Ideas y Roadmap:** [`documentos/ideas/`](documentos/ideas/)
- **Control de Versiones:** [`documentos/versiones/`](documentos/versiones/)

## 📝 Notas

- La base de datos SQLite se crea automáticamente en `data/database.db` (portable)
- Los tokens OAuth se almacenan en la base de datos
- El proyecto está configurado para funcionar offline y sincronizar cuando vuelva la conexión

## 🤝 Contribuir

Este es un proyecto en desarrollo activo. Las contribuciones son bienvenidas.

## 📄 Licencia

MIT License - Ver [LICENSE](LICENSE) para más detalles.
