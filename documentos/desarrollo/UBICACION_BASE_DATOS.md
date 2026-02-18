# 📍 Ubicación de la Base de Datos

## Ruta Completa

La base de datos SQLite se crea automáticamente en:

```
D:\Proyectos\Agente ofimática\Code\data\database.db
```

## Cómo se Determina la Ruta

El código en `src-tauri/src/main.rs` determina la ubicación de la siguiente manera:

1. **En desarrollo** (`npm run dev`):
   - El ejecutable está en: `src-tauri/target/debug/agente-ofimatica.exe`
   - El código sube 3 niveles para llegar a `Code/`
   - Crea `Code/data/database.db`

2. **En producción** (ejecutable compilado):
   - Depende de dónde esté el ejecutable
   - Busca el directorio que contiene `src-tauri` o tiene "Code" en el nombre
   - Crea `data/database.db` relativo a ese directorio

## Estructura de Directorios

```
Code/
├── data/                    # Se crea automáticamente
│   └── database.db          # Base de datos SQLite
├── src-tauri/
│   └── target/
│       └── debug/           # En desarrollo
│       └── release/          # En producción
└── ...
```

## Verificar la Ubicación

Para verificar dónde se creó realmente la base de datos:

1. **Ejecuta la aplicación:**
   ```bash
   npm run dev
   ```

2. **Busca el archivo:**
   - La base de datos se crea la primera vez que se ejecuta la app
   - Busca en: `D:\Proyectos\Agente ofimática\Code\data\database.db`

3. **Si no está ahí:**
   - Busca en el directorio donde está el ejecutable
   - O busca archivos `*.db` en el sistema de archivos

## Notas

- La base de datos es **portable**: está en el directorio del proyecto
- Se crea automáticamente la primera vez que se ejecuta la app
- Los tokens OAuth se guardan en la tabla `oauth_tokens`
- Los eventos de calendario se guardan en la tabla `calendar_events`

## Acceder a la Base de Datos

Puedes usar cualquier cliente SQLite para ver el contenido:

- **DB Browser for SQLite**: https://sqlitebrowser.org/
- **SQLite CLI**: `sqlite3 data/database.db`
- **Extensiones de VS Code**: SQLite Viewer, SQLite

## Backup

Para hacer backup de la base de datos, simplemente copia:
```
data/database.db
```

Para restaurar, reemplaza el archivo en la misma ubicación.
