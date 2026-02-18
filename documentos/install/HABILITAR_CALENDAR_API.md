# 📅 Habilitar Google Calendar API

## Problema

Si ves este error:
```
403 Forbidden - Google Calendar API has not been used in project ... before or it is disabled
```

Significa que necesitas habilitar la Google Calendar API en tu proyecto de Google Cloud Console.

## Solución

### Paso 1: Ir a Google Cloud Console

1. Abre tu navegador y ve a:
   ```
   https://console.cloud.google.com/apis/api/calendar-json.googleapis.com/overview?project=969339938999
   ```
   
   O alternativamente:
   - Ve a https://console.cloud.google.com/
   - Selecciona tu proyecto (el que tiene el ID `969339938999`)
   - Ve a "APIs y Servicios" > "Biblioteca"
   - Busca "Google Calendar API"
   - Haz clic en el resultado

### Paso 2: Habilitar la API

1. En la página de Google Calendar API, haz clic en el botón **"HABILITAR"** (o "ENABLE")
2. Espera unos segundos mientras Google habilita la API

### Paso 3: Verificar

1. Deberías ver un mensaje de confirmación
2. La página debería mostrar "API habilitada" o "API enabled"

### Paso 4: Esperar propagación

- Google puede tardar **1-5 minutos** en propagar los cambios
- Si intentas usar la API inmediatamente después de habilitarla, puede que aún dé error
- Espera unos minutos y vuelve a intentar

## Verificar que está habilitada

1. Ve a "APIs y Servicios" > "APIs habilitadas"
2. Deberías ver "Google Calendar API" en la lista
3. Si no aparece, vuelve a habilitarla

## Probar en la aplicación

1. Una vez habilitada la API, vuelve a la aplicación Tauri
2. Haz clic en "Actualizar" para recargar los eventos
3. Deberías ver tus eventos del calendario

## Nota

Si habilitaste la API recientemente y aún ves el error 403:
- Espera 2-5 minutos
- Vuelve a intentar
- Si persiste, verifica que estás usando el proyecto correcto en Google Cloud Console
