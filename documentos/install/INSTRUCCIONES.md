# 🚀 Instrucciones de Inicio Rápido

## Paso 1: Verificar Requisitos

Abre PowerShell o CMD y verifica que tengas instalado:

```bash
node --version    # Debe ser v18 o superior
npm --version
rustc --version   # Si no está, instala desde https://rustup.rs/
cargo --version
```

## Paso 2: Instalar Dependencias

```bash
cd "D:\Proyectos\Agente ofimática\Code"
npm install
```

Esto instalará todas las dependencias de Node.js necesarias.

## Paso 3: Configurar Credenciales de Google

**IMPORTANTE:** Necesitas crear credenciales OAuth2 en Google Cloud Console.

### 3.1. Crear Proyecto en Google Cloud Console

1. Ve a https://console.cloud.google.com/
2. Crea un nuevo proyecto o selecciona uno existente
3. Nombra el proyecto (ej: "Agente Ofimático")

### 3.2. Habilitar APIs

Ve a "APIs y Servicios" > "Biblioteca" y habilita:
- ✅ Google Calendar API
- ✅ Gmail API  
- ✅ Google Drive API

### 3.3. Crear Credenciales OAuth 2.0

1. Ve a "APIs y Servicios" > "Credenciales"
2. Click en "Crear credenciales" > "ID de cliente OAuth 2.0"
3. Tipo de aplicación: **Aplicación de escritorio**
4. Nombre: "Agente Ofimático Desktop"
5. URI de redirección autorizados: `http://localhost:1420/oauth/callback`
6. Click en "Crear"
7. **Copia el Client ID y Client Secret**

### 3.4. Configurar en el Proyecto

Edita el archivo `src-tauri/src/google_auth.rs` y reemplaza:

```rust
const CLIENT_ID: &str = "TU_CLIENT_ID_AQUI";
const CLIENT_SECRET: &str = "TU_CLIENT_SECRET_AQUI";
```

Con tus credenciales reales.

**⚠️ IMPORTANTE:** Nunca commitees estas credenciales. Están en `.gitignore` pero ten cuidado.

## Paso 4: Ejecutar en Desarrollo

```bash
npm run dev
```

La primera vez puede tardar varios minutos mientras Rust compila todas las dependencias.

Deberías ver:
- Una ventana de la aplicación Tauri
- El servidor de desarrollo de Vite corriendo

## Paso 5: Probar la Conexión

1. En la app, click en "Conectar con Google"
2. Se abrirá tu navegador
3. Autoriza los permisos
4. **NOTA:** Por ahora, el callback OAuth aún no está implementado completamente (ver TODO en código)

## 🐛 Problemas Comunes

### Error: "No se puede compilar Rust"
- Instala [Microsoft Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Ejecuta: `rustup update`

### Error: "Puerto 1420 en uso"
- Cierra otras aplicaciones que usen ese puerto
- O cambia el puerto en `vite.config.ts` y `tauri.conf.json`

### Error: "No se encuentra cargo"
- Asegúrate de tener Rust instalado: https://rustup.rs/
- Reinicia la terminal después de instalar

## 📝 Próximos Pasos

Una vez que tengas la app corriendo:

1. **Completar OAuth2:** Implementar el servidor local para recibir el callback
2. **Conectar Google Calendar:** Hacer requests reales a la API
3. **Agregar más funcionalidades:** Gmail, Drive, etc.

## 💡 Notas

- La base de datos SQLite se creará automáticamente en `data/database.db`
- Los tokens se guardarán en la base de datos (aún no implementado completamente)
- El proyecto está configurado para funcionar offline
