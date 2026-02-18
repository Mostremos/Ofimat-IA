#![allow(dead_code)] // TokenRequest fue removido pero el linter puede seguir reportándolo

use crate::database::Database;
use tauri::{AppHandle, Manager, WindowBuilder};
use oauth2::{
    basic::BasicClient,
    reqwest::async_http_client,
    AuthorizationCode,
    AuthUrl,
    ClientId,
    ClientSecret,
    RedirectUrl,
    Scope,
    TokenResponse,
    TokenUrl,
};
use std::sync::Mutex;
use tokio::net::TcpListener;

// Canal compartido para recibir el código OAuth desde el webview
static OAUTH_CODE_SENDER: Mutex<Option<tokio::sync::oneshot::Sender<Result<String, String>>>> = Mutex::new(None);

// Función helper para obtener credenciales desde variables de entorno
// Las credenciales deben estar en un archivo .env o como variables de entorno del sistema
fn get_client_id() -> String {
    std::env::var("GOOGLE_CLIENT_ID")
        .unwrap_or_else(|_| {
            eprintln!("⚠️ GOOGLE_CLIENT_ID no encontrada. Usando placeholder.");
            "TU_CLIENT_ID_AQUI".to_string()
        })
}

fn get_client_secret() -> String {
    std::env::var("GOOGLE_CLIENT_SECRET")
        .unwrap_or_else(|_| {
            eprintln!("⚠️ GOOGLE_CLIENT_SECRET no encontrada. Usando placeholder.");
            "TU_CLIENT_SECRET_AQUI".to_string()
        })
}
// IMPORTANTE: Para apps de escritorio, Google requiere usar 127.0.0.1 (no localhost)
// Esto permite que el navegador entregue automáticamente el código al servidor local
const REDIRECT_URI: &str = "http://127.0.0.1:1420/oauth/callback";
const CALLBACK_PORT: u16 = 1420;

// TokenResponse ya no es necesario - la biblioteca oauth2 maneja esto


/// Comando Tauri para recibir el código OAuth desde el webview
#[tauri::command]
pub async fn oauth_callback(code: String) -> Result<(), String> {
    eprintln!("✅ Código OAuth recibido desde webview: {}...", &code[..10.min(code.len())]);
    
    // Enviar código al canal si existe
    if let Ok(mut sender_opt) = OAUTH_CODE_SENDER.lock() {
        if let Some(sender) = sender_opt.take() {
            let _ = sender.send(Ok(code));
            eprintln!("✅ Código enviado al canal");
        } else {
            eprintln!("⚠️ No hay receptor esperando el código OAuth");
        }
    }
    
    Ok(())
}

/// Función alternativa para autenticar usando un código de autorización manual
/// Útil cuando el servidor HTTP local no funciona correctamente
pub async fn authenticate_with_code(db: &Database, auth_code: &str) -> Result<bool, String> {
    eprintln!("🔑 Usando código de autorización manual...");
    
    // Crear cliente OAuth2
    let client_id = get_client_id();
    let client_secret = get_client_secret();
    
    // Verificar que las credenciales estén configuradas
    if client_id == "TU_CLIENT_ID_AQUI" || client_secret == "TU_CLIENT_SECRET_AQUI" {
        return Err("Por favor configura GOOGLE_CLIENT_ID y GOOGLE_CLIENT_SECRET en un archivo .env o como variables de entorno. Ver documentos/install/CONFIGURAR_GOOGLE_OAUTH.md".to_string());
    }
    
    let client = BasicClient::new(
        ClientId::new(client_id.clone()),
        Some(ClientSecret::new(client_secret.clone())),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .map_err(|e| format!("Error al crear AuthUrl: {}", e))?,
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
            .map_err(|e| format!("Error al crear TokenUrl: {}", e))?),
    )
    .set_redirect_uri(RedirectUrl::new(REDIRECT_URI.to_string())
        .map_err(|e| format!("Error al crear RedirectUrl: {}", e))?);
    
    // Intercambiar código por token usando la biblioteca oauth2
    eprintln!("🔄 Intercambiando código por token...");
    let token: oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType> = client
        .exchange_code(AuthorizationCode::new(auth_code.to_string()))
        .request_async(async_http_client)
        .await
        .map_err(|e| format!("Error al obtener token: {}", e))?;
    
    eprintln!("✅ Token recibido exitosamente");
    
    // Guardar token en la base de datos
    let access_token: String = token.access_token().secret().clone();
    let refresh_token: Option<String> = token.refresh_token().map(|rt| rt.secret().clone());
    let expires_at: i64 = token.expires_in()
        .and_then(|expires| expires.as_secs().checked_add(chrono::Utc::now().timestamp() as u64))
        .map(|secs| secs as i64)
        .unwrap_or(0);
    
    eprintln!("💾 Guardando token en base de datos...");
    db.save_token(
        "google",
        &access_token,
        refresh_token.as_deref(),
        Some(expires_at),
    )
    .map_err(|e| format!("Error al guardar token: {}", e))?;
    
    eprintln!("✅ Autenticación completada exitosamente!");
    Ok(true)
}

pub async fn authenticate(app: &AppHandle, db: &Database) -> Result<bool, String> {
    // Obtener credenciales desde variables de entorno
    let client_id = get_client_id();
    let client_secret = get_client_secret();
    
    // Verificar que las credenciales estén configuradas
    if client_id == "TU_CLIENT_ID_AQUI" || client_secret == "TU_CLIENT_SECRET_AQUI" {
        return Err("Por favor configura GOOGLE_CLIENT_ID y GOOGLE_CLIENT_SECRET en un archivo .env o como variables de entorno. Ver documentos/install/CONFIGURAR_GOOGLE_OAUTH.md".to_string());
    }
    
    eprintln!("Usando CLIENT_ID: {}...", &client_id[..20.min(client_id.len())]);
    eprintln!("Usando REDIRECT_URI: {}", REDIRECT_URI);

    // Crear cliente OAuth2 usando la biblioteca oauth2
    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
            .map_err(|e| format!("Error al crear AuthUrl: {}", e))?,
        Some(TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
            .map_err(|e| format!("Error al crear TokenUrl: {}", e))?),
    )
    .set_redirect_uri(RedirectUrl::new(REDIRECT_URI.to_string())
        .map_err(|e| format!("Error al crear RedirectUrl: {}", e))?);

    // Definir scopes
    let scopes = vec![
        Scope::new("https://www.googleapis.com/auth/calendar".to_string()),
        Scope::new("https://www.googleapis.com/auth/gmail.readonly".to_string()),
        Scope::new("https://www.googleapis.com/auth/drive.readonly".to_string()),
    ];

    // PRIMERO: Iniciar el servidor HTTP ANTES de abrir el navegador
    eprintln!("🔵 Iniciando servidor HTTP local en puerto {} para recibir callback...", CALLBACK_PORT);

    // Crear servidor HTTP local para recibir el callback
    let (tx, rx) = tokio::sync::oneshot::channel::<Result<AuthorizationCode, String>>();
    let (server_ready_tx, server_ready_rx) = tokio::sync::oneshot::channel::<()>();
    
    use axum::{
        extract::Query,
        response::Html,
        routing::get,
        Router,
    };
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex as TokioMutex;
    
    let tx_for_callback = Arc::new(TokioMutex::new(Some(tx)));
    let tx_clone = tx_for_callback.clone();
    let router = Router::new()
        .route("/oauth/callback", get(move |Query(params): Query<HashMap<String, String>>| async move {
            eprintln!("✅ Request recibido en /oauth/callback");
            eprintln!("📋 Parámetros recibidos: {:?}", params);
            
            if let Some(code) = params.get("code") {
                eprintln!("✅ Código de autorización recibido: {}...", &code[..10.min(code.len())]);
                let mut tx_opt = tx_clone.lock().await;
                if let Some(tx) = tx_opt.take() {
                    let _ = tx.send(Ok(AuthorizationCode::new(code.clone())));
                    eprintln!("✅ Código enviado al canal");
                } else {
                    eprintln!("⚠️ No hay receptor esperando el código");
                }
                Html(SUCCESS_HTML.to_string())
            } else if let Some(error) = params.get("error") {
                eprintln!("❌ Error en request: {}", error);
                let mut tx_opt = tx_clone.lock().await;
                if let Some(tx) = tx_opt.take() {
                    let _ = tx.send(Err(format!("Error de autorización: {}", error)));
                }
                Html(ERROR_HTML.to_string())
            } else {
                eprintln!("⚠️ Request sin código ni error");
                Html(ERROR_HTML.to_string())
            }
        }))
        .route("/", get(|| async { 
            eprintln!("📄 Request recibido en / (root)");
            Html("OAuth callback server".to_string()) 
        }));

    // Iniciar servidor Axum
    let server_handle = tokio::spawn(async move {
        // IMPORTANTE: Para apps de escritorio, Google requiere usar 127.0.0.1 (no localhost ni 0.0.0.0)
        let listener = TcpListener::bind(format!("127.0.0.1:{}", CALLBACK_PORT)).await
            .map_err(|e| format!("Error: No se pudo escuchar en 127.0.0.1:{}: {}", CALLBACK_PORT, e))?;
        
        eprintln!("✅ Servidor HTTP escuchando en 127.0.0.1:{} (IPv4)", CALLBACK_PORT);
        
        // Notificar que el servidor está listo
        let _ = server_ready_tx.send(());
        
        eprintln!("⏳ Esperando callback de Google...");
        eprintln!("🔍 Servidor listo para aceptar conexiones en el puerto {}", CALLBACK_PORT);
        eprintln!("🌐 URL de callback esperada: {}", REDIRECT_URI);
        eprintln!("🚀 Iniciando servidor Axum...");

        // Axum 0.7 acepta directamente un TcpListener
        axum::serve(listener, router.into_make_service())
            .await
            .map_err(|e| {
                eprintln!("❌ Error en servidor Axum: {}", e);
                format!("Error en servidor HTTP: {}", e)
            })?;
        
        Ok::<(), String>(())
    });

    // Esperar a que el servidor esté completamente listo
    eprintln!("⏳ Esperando a que el servidor esté listo...");
    server_ready_rx.await
        .map_err(|_| "Error al esperar servidor".to_string())?;
    
    // Pequeño delay adicional para asegurar que el servidor está completamente listo
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    eprintln!("✅ Servidor listo, generando URL de autorización...");

    // SEGUNDO: Generar URL de autorización y abrir navegador DESPUÉS de que el servidor esté listo
    let (auth_url, _csrf_token) = client
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scopes(scopes.into_iter())
        .url();

    eprintln!("🔵 URL de autorización generada");
    eprintln!("🪟 Abriendo ventana de autorización en webview de Tauri...");
    
    // Crear ventana webview de Tauri para la autorización
    let _auth_window = WindowBuilder::new(
        app,
        "oauth_auth",
        tauri::WindowUrl::External(auth_url.clone())
    )
    .title("Autorización de Google")
    .inner_size(600.0, 700.0)
    .resizable(true)
    .center()
    .build()
    .map_err(|e| format!("Error al crear ventana de autorización: {}", e))?;
    
    eprintln!("✅ Ventana de autorización abierta en webview de Tauri");
    eprintln!("⏳ Esperando autorización en el webview...");

    // Esperar el código de autorización
    eprintln!("⏳ Esperando código de autorización (timeout: 5 minutos)...");
    let auth_code_result = match tokio::time::timeout(
        std::time::Duration::from_secs(300),
        rx
    ).await {
        Ok(Ok(code_result)) => code_result,
        Ok(Err(e)) => {
            eprintln!("❌ Error en canal: {}", e);
            server_handle.abort();
            return Err(format!("Error al recibir callback: {}", e));
        },
        Err(_) => {
            eprintln!("❌ Timeout esperando callback");
            server_handle.abort();
            return Err("Timeout: No se recibió respuesta de Google en 5 minutos. Por favor, intenta de nuevo.".to_string());
        },
    };
    
    let auth_code = match auth_code_result {
        Ok(code) => {
            eprintln!("✅ Código recibido exitosamente");
            code
        },
        Err(e) => {
            eprintln!("❌ Error en callback: {}", e);
            // Cerrar ventana de autorización si existe
            if let Some(window) = app.get_window("oauth_auth") {
                let _ = window.close();
            }
            server_handle.abort();
            return Err(format!("Error de autorización: {}", e));
        },
    };

    // Cerrar ventana de autorización
    if let Some(window) = app.get_window("oauth_auth") {
        let _ = window.close();
        eprintln!("✅ Ventana de autorización cerrada");
    }
    
    // Cancelar el servidor
    server_handle.abort();

    // Intercambiar código por token usando la biblioteca oauth2
    eprintln!("🔄 Intercambiando código por token...");
    let token_result = client
        .exchange_code(auth_code)
        .request_async(async_http_client)
        .await;

    let token: oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType> = match token_result {
        Ok(token) => {
            eprintln!("✅ Token recibido exitosamente");
            token
        },
        Err(e) => {
            eprintln!("❌ Error al intercambiar código por token: {}", e);
            return Err(format!("Error al obtener token: {}", e));
        }
    };

    // Guardar token en la base de datos
    let access_token = token.access_token().secret().clone();
    let refresh_token = token.refresh_token().map(|rt| rt.secret().clone());
    let expires_at = token.expires_in()
        .and_then(|expires| expires.as_secs().checked_add(chrono::Utc::now().timestamp() as u64))
        .map(|secs| secs as i64)
        .unwrap_or(0);

    eprintln!("💾 Guardando token en base de datos...");
    db.save_token(
        "google",
        &access_token,
        refresh_token.as_deref(),
        Some(expires_at),
    )
    .map_err(|e| format!("Error al guardar token: {}", e))?;

    eprintln!("✅ Autenticación completada exitosamente!");
    Ok(true)
}

fn extract_code_from_request(request: &str) -> Option<String> {
    eprintln!("🔍 Extrayendo código del request...");
    // Buscar línea GET que contiene el código
    for line in request.lines() {
        if line.starts_with("GET") {
            eprintln!("📝 Línea GET encontrada: {}", line);
            if let Some(query_start) = line.find('?') {
                let query = &line[query_start + 1..];
                eprintln!("📝 Query string: {}", query);
                if let Some(code_param) = query.find("code=") {
                    let code_start = code_param + 5;
                    let code_end = query[code_start..]
                        .find('&')
                        .map(|i| code_start + i)
                        .unwrap_or_else(|| query.len());
                    let code = &query[code_start..code_end];
                    eprintln!("✅ Código extraído: {}...", &code[..10.min(code.len())]);
                    return Some(code.to_string());
                } else {
                    eprintln!("❌ No se encontró 'code=' en el query string");
                }
            } else {
                eprintln!("❌ No se encontró '?' en la línea GET");
            }
        }
    }
    eprintln!("❌ No se encontró línea GET con código");
    None
}

fn extract_error_from_request(request: &str) -> Option<String> {
    for line in request.lines() {
        if line.starts_with("GET") {
            if let Some(query_start) = line.find('?') {
                let query = &line[query_start + 1..];
                if let Some(error_param) = query.find("error=") {
                    let error_start = error_param + 7;
                    let error_end = query[error_start..]
                        .find('&')
                        .map(|i| error_start + i)
                        .unwrap_or_else(|| line.len() - query_start - 1);
                    let error = &query[error_start..error_end];
                    return Some(error.to_string());
                }
            }
        }
    }
    None
}

fn create_success_response() -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        Content-Length: {}\r\n\
        \r\n\
        {}",
        SUCCESS_HTML.len(),
        SUCCESS_HTML
    )
}

fn create_error_response() -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
        Content-Type: text/html; charset=utf-8\r\n\
        Content-Length: {}\r\n\
        \r\n\
        {}",
        ERROR_HTML.len(),
        ERROR_HTML
    )
}

const SUCCESS_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Autorización Exitosa</title>
    <meta charset="UTF-8">
    <style>
        body {
            font-family: Arial, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        }
        .container {
            background: white;
            padding: 40px;
            border-radius: 10px;
            box-shadow: 0 10px 25px rgba(0,0,0,0.2);
            text-align: center;
            max-width: 500px;
        }
        h1 { color: #4caf50; margin-bottom: 20px; }
        p { color: #666; margin: 10px 0; }
        .checkmark {
            font-size: 60px;
            color: #4caf50;
            margin-bottom: 20px;
        }
        .info-box {
            background: #e8f5e9;
            border: 1px solid #4caf50;
            border-radius: 5px;
            padding: 15px;
            margin: 20px 0;
            text-align: left;
        }
        .info-box p {
            margin: 5px 0;
            font-size: 14px;
        }
        .close-button {
            background: #4caf50;
            color: white;
            border: none;
            padding: 12px 30px;
            border-radius: 5px;
            font-size: 16px;
            cursor: pointer;
            margin-top: 20px;
        }
        .close-button:hover {
            background: #45a049;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="checkmark">✅</div>
        <h1>Autorización Exitosa</h1>
        <p>Tu cuenta de Google ha sido conectada correctamente.</p>
        <div class="info-box">
            <p><strong>✅ Todo listo:</strong></p>
            <p>• La aplicación ya tiene acceso a tu calendario</p>
            <p>• Puedes crear, editar y eliminar eventos</p>
            <p>• Puedes cerrar esta ventana ahora</p>
        </div>
        <button class="close-button" onclick="window.close()">Cerrar Ventana</button>
        <p style="font-size: 12px; color: #999; margin-top: 15px;">
            Si el botón no funciona, cierra esta ventana manualmente.
        </p>
    </div>
    <script>
        // Intentar cerrar automáticamente después de 3 segundos
        setTimeout(() => {
            try {
                window.close();
            } catch(e) {
                // Si no se puede cerrar, mostrar mensaje más visible
                console.log('No se pudo cerrar automáticamente. Por favor, cierra manualmente.');
            }
        }, 3000);
    </script>
</body>
</html>
"#;

const ERROR_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Error de Autorización</title>
    <meta charset="UTF-8">
    <style>
        body {
            font-family: Arial, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #ff7e5f 0%, #feb47b 100%);
        }
        .container {
            background: white;
            padding: 40px;
            border-radius: 10px;
            box-shadow: 0 10px 25px rgba(0,0,0,0.2);
            text-align: center;
            max-width: 500px;
        }
        h1 { color: #f44336; margin-bottom: 20px; }
        p { color: #666; margin: 10px 0; }
        .error-icon {
            font-size: 60px;
            color: #f44336;
            margin-bottom: 20px;
        }
        .info-box {
            background: #ffebee;
            border: 1px solid #f44336;
            border-radius: 5px;
            padding: 15px;
            margin: 20px 0;
            text-align: left;
        }
        .info-box p {
            margin: 5px 0;
            font-size: 14px;
        }
        .close-button {
            background: #f44336;
            color: white;
            border: none;
            padding: 12px 30px;
            border-radius: 5px;
            font-size: 16px;
            cursor: pointer;
            margin-top: 20px;
        }
        .close-button:hover {
            background: #d32f2f;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="error-icon">❌</div>
        <h1>Error de Autorización</h1>
        <p>Hubo un problema durante la autorización.</p>
        <div class="info-box">
            <p><strong>¿Qué hacer?</strong></p>
            <p>• Vuelve a la aplicación</p>
            <p>• Intenta conectarte nuevamente</p>
            <p>• Si el problema persiste, revisa la consola de la aplicación</p>
        </div>
        <button class="close-button" onclick="window.close()">Cerrar Ventana</button>
    </div>
    <script>
        // Intentar cerrar automáticamente después de 5 segundos
        setTimeout(() => {
            try {
                window.close();
            } catch(e) {
                console.log('No se pudo cerrar automáticamente. Por favor, cierra manualmente.');
            }
        }, 5000);
    </script>
</body>
</html>
"#;

// exchange_code_for_token ya no es necesario - la biblioteca oauth2 lo maneja
