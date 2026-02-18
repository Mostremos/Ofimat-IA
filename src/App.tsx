import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

interface CalendarEvent {
  id: string;
  title: string;
  start: string;
  end: string;
  description?: string;
  location?: string;
  is_all_day: boolean;
  attendees?: string[]; // Lista de emails de participantes
}

function App() {
  const [events, setEvents] = useState<CalendarEvent[]>([]);
  const [isConnected, setIsConnected] = useState(false);
  const [loading, setLoading] = useState(false);
  const [showManualInput, setShowManualInput] = useState(false);
  const [manualCode, setManualCode] = useState("");
  const [isTauri, setIsTauri] = useState(false);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [showAIChat, setShowAIChat] = useState(false);
  const [aiInstruction, setAiInstruction] = useState("");
  const [aiLoading, setAiLoading] = useState(false);
  const [editingEvent, setEditingEvent] = useState<CalendarEvent | null>(null);
  const formRef = useRef<HTMLDivElement>(null);
  const [newEvent, setNewEvent] = useState({
    title: "",
    startDate: "",
    startTime: "",
    endDate: "",
    endTime: "",
    description: "",
    location: "",
    isAllDay: false,
    attendees: "", // Lista de emails separados por comas o saltos de línea
  });

  // Hacer scroll al formulario cuando se abre
  useEffect(() => {
    if (showCreateForm && formRef.current) {
      // Pequeño delay para asegurar que el DOM se haya actualizado
      setTimeout(() => {
        formRef.current?.scrollIntoView({ 
          behavior: 'smooth', 
          block: 'start',
          inline: 'nearest'
        });
      }, 100);
    }
  }, [showCreateForm]);

  useEffect(() => {
    // Verificar si estamos en Tauri - múltiples métodos para asegurar detección
    const isTauriEnv = typeof window !== 'undefined' && (
      (window as any).__TAURI_IPC__ !== undefined ||
      (window as any).__TAURI__ !== undefined ||
      (window as any).__TAURI_INTERNALS__ !== undefined
    );
    
    console.log("🔍 Detectando entorno Tauri:", {
      hasWindow: typeof window !== 'undefined',
      hasTAURI_IPC: typeof window !== 'undefined' && (window as any).__TAURI_IPC__ !== undefined,
      hasTAURI: typeof window !== 'undefined' && (window as any).__TAURI__ !== undefined,
      hasTAURI_INTERNALS: typeof window !== 'undefined' && (window as any).__TAURI_INTERNALS__ !== undefined,
      isTauriEnv
    });
    
    if (isTauriEnv) {
      setIsTauri(true);
      console.log("✅ Detectado: Aplicación Tauri");
      // Verificar estado de conexión al cargar - usar isTauriEnv directamente para evitar race condition
      checkConnectionDirect();
    } else {
      setIsTauri(false);
      console.warn("⚠️ No estás en la aplicación Tauri. Abre la aplicación de escritorio, no el navegador.");
      console.warn("💡 Para usar la aplicación:");
      console.warn("   1. Cierra esta ventana del navegador");
      console.warn("   2. Busca la ventana 'Agente Ofimático' (debería haberse abierto automáticamente)");
      console.warn("   3. Si no se abrió, ejecuta 'npm run dev' en la terminal");
    }
  }, []);

  // Función para verificar conexión que no depende del estado isTauri (para evitar race conditions)
  const checkConnectionDirect = async () => {
    try {
      console.log("🔍 Verificando conexión con Google...");
      const connected = await invoke<boolean>("is_google_connected");
      console.log("📊 Estado de conexión:", connected);
      setIsConnected(connected);
      if (connected) {
        console.log("✅ Conectado, cargando eventos...");
        loadEvents();
      } else {
        console.log("❌ No conectado, mostrando botones de conexión");
      }
    } catch (error) {
      console.error("❌ Error checking connection:", error);
    }
  };

  const checkConnection = async () => {
    if (!isTauri) {
      console.log("⏭️ Saltando checkConnection: no estamos en Tauri");
      return;
    }
    
    try {
      console.log("🔍 Verificando conexión con Google...");
      const connected = await invoke<boolean>("is_google_connected");
      console.log("📊 Estado de conexión:", connected);
      setIsConnected(connected);
      if (connected) {
        console.log("✅ Conectado, cargando eventos...");
        loadEvents();
      } else {
        console.log("❌ No conectado, mostrando botones de conexión");
      }
    } catch (error) {
      console.error("❌ Error checking connection:", error);
    }
  };

  const connectGoogle = async () => {
    if (!isTauri) {
      alert("⚠️ Esta función solo funciona en la aplicación de escritorio.\n\nPor favor, cierra esta ventana del navegador y abre la aplicación Tauri.");
      return;
    }

    setLoading(true);
    try {
      console.log("Iniciando conexión con Google...");
      
      // Agregar timeout para evitar que se quede bloqueado indefinidamente
      const timeoutPromise = new Promise<never>((_, reject) => {
        setTimeout(() => {
          reject(new Error("Timeout: El servidor HTTP no recibió la conexión en 2 minutos. Usa 'Ingresar código manualmente' para continuar."));
        }, 120000); // 2 minutos
      });
      
      const connectPromise = invoke<boolean>("connect_google");
      
      const success = await Promise.race([connectPromise, timeoutPromise]);
      console.log("Resultado de conexión:", success);
      if (success) {
        setIsConnected(true);
        await loadEvents();
      }
    } catch (error: any) {
      console.error("Error connecting to Google:", error);
      const errorMessage = error?.toString() || "Error desconocido";
      
      // Si es un timeout, sugerir usar el método manual
      if (errorMessage.includes("Timeout")) {
        alert(
          `⏱️ ${errorMessage}\n\n` +
          `💡 Solución:\n` +
          `1. Haz clic en "Ingresar código manualmente"\n` +
          `2. Copia el código de la URL del navegador (después de "code=")\n` +
          `3. Pega el código en el prompt`
        );
      } else {
        alert(`Error al conectar con Google:\n\n${errorMessage}\n\nRevisa la consola para más detalles.`);
      }
    } finally {
      setLoading(false);
      console.log("✅ Loading reseteado");
    }
  };

  const connectGoogleManual = async () => {
    if (!isTauri) {
      alert("⚠️ Esta función solo funciona en la aplicación de escritorio.\n\nPor favor, cierra esta ventana del navegador y abre la aplicación Tauri.");
      return;
    }

    // Usar prompt nativo del navegador para pedir el código
    const code = prompt(
      "Ingresa el código de autorización de Google.\n\n" +
      "Copia solo la parte después de 'code=' de la URL del navegador.\n\n" +
      "Ejemplo: 4/0AfrIep...",
      ""
    );

    if (!code || !code.trim()) {
      return; // Usuario canceló o no ingresó código
    }

    setLoading(true);
    try {
      console.log("Conectando con código manual...");
      const success = await invoke<boolean>("connect_google_manual", { authCode: code.trim() });
      console.log("Resultado de conexión:", success);
      if (success) {
        setIsConnected(true);
        setShowManualInput(false);
        setManualCode("");
        await loadEvents();
      }
    } catch (error: any) {
      console.error("Error connecting to Google:", error);
      const errorMessage = error?.toString() || "Error desconocido";
      alert(`Error al conectar con Google:\n\n${errorMessage}\n\nRevisa la consola para más detalles.`);
    } finally {
      setLoading(false);
    }
  };

  const loadEvents = async () => {
    if (!isTauri) return;
    
    try {
      const calendarEvents = await invoke<CalendarEvent[]>("get_calendar_events");
      setEvents(calendarEvents);
    } catch (error) {
      console.error("Error loading events:", error);
    }
  };

  const createEvent = async () => {
    if (!isTauri) return;
    
    // Validar campos requeridos
    if (!newEvent.title.trim()) {
      alert("Por favor, ingresa un título para el evento.");
      return;
    }
    
    if (!newEvent.startDate) {
      alert("Por favor, selecciona una fecha de inicio.");
      return;
    }
    
    if (!newEvent.isAllDay && !newEvent.startTime) {
      alert("Por favor, selecciona una hora de inicio.");
      return;
    }
    
    if (!newEvent.endDate) {
      alert("Por favor, selecciona una fecha de fin.");
      return;
    }
    
    if (!newEvent.isAllDay && !newEvent.endTime) {
      alert("Por favor, selecciona una hora de fin.");
      return;
    }
    
    setLoading(true);
    try {
      // Construir fechas según si es todo el día o con hora
      let start: string;
      let end: string;
      
      if (newEvent.isAllDay) {
        // Para eventos de todo el día, usar formato YYYY-MM-DD
        start = newEvent.startDate;
        end = newEvent.endDate;
      } else {
        // Para eventos con hora, usar formato RFC3339
        // Combinar fecha y hora en la zona horaria local
        const startLocal = new Date(`${newEvent.startDate}T${newEvent.startTime}`);
        const endLocal = new Date(`${newEvent.endDate}T${newEvent.endTime}`);
        
        // Validar que la fecha de fin sea posterior a la de inicio
        if (endLocal <= startLocal) {
          alert("La fecha/hora de fin debe ser posterior a la fecha/hora de inicio.");
          return;
        }
        
        // Convertir a RFC3339 (ISO 8601) - esto convierte a UTC pero mantiene el tiempo correcto
        start = startLocal.toISOString();
        end = endLocal.toISOString();
      }
      
      console.log("📝 Creando evento:", {
        title: newEvent.title,
        start,
        end,
        isAllDay: newEvent.isAllDay,
      });
      
      // Procesar participantes: separar por comas o saltos de línea, limpiar espacios
      const attendeesList = newEvent.attendees
        .split(/[,\n]/)
        .map(email => email.trim())
        .filter(email => email.length > 0 && email.includes('@'));
      
      const createdEvent = await invoke<CalendarEvent>("create_calendar_event", {
        event: {
          title: newEvent.title.trim(),
          start,
          end,
          description: newEvent.description.trim() || null,
          location: newEvent.location.trim() || null,
          is_all_day: newEvent.isAllDay,
          attendees: attendeesList.length > 0 ? attendeesList : null,
        },
      });
      
      console.log("✅ Evento creado:", createdEvent);
      
      // Limpiar formulario
      setNewEvent({
        title: "",
        startDate: "",
        startTime: "",
        endDate: "",
        endTime: "",
        description: "",
        location: "",
        isAllDay: false,
        attendees: "",
      });
      setShowCreateForm(false);
      
      // Recargar eventos - agregar un pequeño delay para asegurar que Google haya procesado el evento
      await new Promise(resolve => setTimeout(resolve, 1000)); // Esperar 1 segundo
      await loadEvents();
      
      alert("✅ Evento creado exitosamente!");
    } catch (error: any) {
      console.error("❌ Error creando evento:", error);
      const errorMessage = error?.toString() || "Error desconocido";
      alert(`Error al crear evento:\n\n${errorMessage}\n\nRevisa la consola para más detalles.`);
    } finally {
      setLoading(false);
    }
  };

  const createEventWithAI = async () => {
    if (!isTauri || !aiInstruction.trim()) {
      alert("Por favor, ingresa una instrucción para crear el evento.");
      return;
    }

    setAiLoading(true);
    try {
      console.log("🤖 Creando evento con IA:", aiInstruction);
      const createdEvent = await invoke<CalendarEvent>("create_event_from_natural_language", {
        instruction: aiInstruction.trim(),
      });
      console.log("✅ Evento creado con IA:", createdEvent);
      
      // Limpiar el campo de instrucción
      setAiInstruction("");
      setShowAIChat(false);
      
      // Recargar eventos
      await loadEvents();
      
      alert(`✅ Evento creado exitosamente!\n\n"${createdEvent.title}"`);
    } catch (error: any) {
      console.error("❌ Error creando evento con IA:", error);
      alert(`Error al crear evento con IA:\n\n${error}`);
    } finally {
      setAiLoading(false);
    }
  };

  const deleteEvent = async (eventId: string, eventTitle: string) => {
    if (!isTauri) return;
    
    if (!confirm(`¿Estás seguro de que quieres eliminar el evento "${eventTitle}"?`)) {
      return;
    }
    
    setLoading(true);
    try {
      console.log("🗑️ Eliminando evento:", eventId);
      await invoke("delete_calendar_event", { eventId });
      console.log("✅ Evento eliminado");
      
      // Recargar eventos
      await loadEvents();
      
      alert("✅ Evento eliminado exitosamente!");
    } catch (error: any) {
      console.error("❌ Error eliminando evento:", error);
      const errorMessage = error?.toString() || "Error desconocido";
      alert(`Error al eliminar evento:\n\n${errorMessage}\n\nRevisa la consola para más detalles.`);
    } finally {
      setLoading(false);
    }
  };

  const startEditEvent = (event: CalendarEvent) => {
    // Parsear fechas del evento para llenar el formulario
    let startDate = "";
    let startTime = "";
    let endDate = "";
    let endTime = "";
    
    if (event.is_all_day) {
      // Para eventos de todo el día, la fecha viene como YYYY-MM-DD
      startDate = event.start;
      endDate = event.end;
    } else {
      // Para eventos con hora, parsear RFC3339
      // Usar la fecha local del navegador para mantener la zona horaria correcta
      const start = new Date(event.start);
      const end = new Date(event.end);
      
      // Obtener año, mes y día en la zona horaria local
      const startYear = start.getFullYear();
      const startMonth = String(start.getMonth() + 1).padStart(2, '0');
      const startDay = String(start.getDate()).padStart(2, '0');
      startDate = `${startYear}-${startMonth}-${startDay}`;
      
      const endYear = end.getFullYear();
      const endMonth = String(end.getMonth() + 1).padStart(2, '0');
      const endDay = String(end.getDate()).padStart(2, '0');
      endDate = `${endYear}-${endMonth}-${endDay}`;
      
      // Obtener hora y minuto en la zona horaria local
      startTime = String(start.getHours()).padStart(2, '0') + ':' + String(start.getMinutes()).padStart(2, '0');
      endTime = String(end.getHours()).padStart(2, '0') + ':' + String(end.getMinutes()).padStart(2, '0');
    }
    
    setNewEvent({
      title: event.title,
      startDate,
      startTime,
      endDate,
      endTime,
      description: event.description || "",
      location: event.location || "",
      isAllDay: event.is_all_day,
      attendees: event.attendees ? event.attendees.join(', ') : "",
    });
    
    setEditingEvent(event);
    setShowCreateForm(true);
  };

  const updateEvent = async () => {
    if (!isTauri || !editingEvent) return;
    
    // Validar campos (mismo código que createEvent)
    if (!newEvent.title.trim()) {
      alert("Por favor, ingresa un título para el evento.");
      return;
    }
    
    if (!newEvent.startDate) {
      alert("Por favor, selecciona una fecha de inicio.");
      return;
    }
    
    if (!newEvent.isAllDay && !newEvent.startTime) {
      alert("Por favor, selecciona una hora de inicio.");
      return;
    }
    
    if (!newEvent.endDate) {
      alert("Por favor, selecciona una fecha de fin.");
      return;
    }
    
    if (!newEvent.isAllDay && !newEvent.endTime) {
      alert("Por favor, selecciona una hora de fin.");
      return;
    }
    
    setLoading(true);
    try {
      // Construir fechas (mismo código que createEvent)
      let start: string;
      let end: string;
      
      if (newEvent.isAllDay) {
        start = newEvent.startDate;
        end = newEvent.endDate;
      } else {
        // Combinar fecha y hora en la zona horaria local
        const startLocal = new Date(`${newEvent.startDate}T${newEvent.startTime}`);
        const endLocal = new Date(`${newEvent.endDate}T${newEvent.endTime}`);
        
        // Validar que la fecha de fin sea posterior a la de inicio
        if (endLocal <= startLocal) {
          alert("La fecha/hora de fin debe ser posterior a la fecha/hora de inicio.");
          return;
        }
        
        // Convertir a RFC3339 (ISO 8601)
        start = startLocal.toISOString();
        end = endLocal.toISOString();
      }
      
      console.log("✏️ Actualizando evento:", editingEvent.id);
      
      // Procesar participantes: separar por comas o saltos de línea, limpiar espacios
      // Si el campo está vacío, enviar array vacío [] para eliminar todos los participantes
      // Si tiene contenido, enviar la lista procesada de emails válidos
      let attendeesToSend: string[] | null = null;
      if (newEvent.attendees.trim() === "") {
        // Campo vacío: enviar array vacío para eliminar todos los participantes
        attendeesToSend = [];
      } else {
        // Procesar y filtrar emails válidos
        const attendeesList = newEvent.attendees
          .split(/[,\n]/)
          .map(email => email.trim())
          .filter(email => email.length > 0 && email.includes('@'));
        attendeesToSend = attendeesList.length > 0 ? attendeesList : [];
      }
      
      const updatedEvent = await invoke<CalendarEvent>("update_calendar_event", {
        eventId: editingEvent.id,
        event: {
          title: newEvent.title.trim(),
          start,
          end,
          description: newEvent.description.trim() || null,
          location: newEvent.location.trim() || null,
          is_all_day: newEvent.isAllDay,
          attendees: attendeesToSend, // Array vacío [] para eliminar, o lista de emails
        },
      });
      
      console.log("✅ Evento actualizado:", updatedEvent);
      
      // Limpiar formulario
      setNewEvent({
        title: "",
        startDate: "",
        startTime: "",
        endDate: "",
        endTime: "",
        description: "",
        location: "",
        isAllDay: false,
        attendees: "",
      });
      setEditingEvent(null);
      setShowCreateForm(false);
      
      // Recargar eventos
      await new Promise(resolve => setTimeout(resolve, 1000));
      await loadEvents();
      
      alert("✅ Evento actualizado exitosamente!");
    } catch (error: any) {
      console.error("❌ Error actualizando evento:", error);
      const errorMessage = error?.toString() || "Error desconocido";
      alert(`Error al actualizar evento:\n\n${errorMessage}\n\nRevisa la consola para más detalles.`);
    } finally {
      setLoading(false);
    }
  };

  // Función para formatear fechas de eventos
  const formatEventDate = (dateString: string, isAllDay: boolean): string => {
    try {
      if (isAllDay) {
        // Para eventos de todo el día, la fecha viene como "YYYY-MM-DD" directamente
        // Parsear solo la fecha sin considerar zona horaria
        const [year, month, day] = dateString.split('-');
        // Mostrar solo la fecha en formato DD/MM/YYYY
        return `${day}/${month}/${year}`;
      } else {
        // Para eventos con hora, parsear con zona horaria
        const date = new Date(dateString);
        return date.toLocaleString("es-ES", {
          year: "numeric",
          month: "2-digit",
          day: "2-digit",
          hour: "2-digit",
          minute: "2-digit"
        });
      }
    } catch (error) {
      console.error("Error formateando fecha:", error, dateString);
      return dateString;
    }
  };

  // Debug: verificar que el componente se renderiza
  console.log("🔄 App render:", {
    isConnected,
    showManualInput,
    isTauri,
    loading,
    shouldShowConnectButtons: !isConnected && isTauri,
    shouldShowManualButton: !isConnected && isTauri
  });

  return (
    <div className="container">
      <h1>Ofimat-IA</h1>
      
      {!isTauri && (
        <div style={{
          padding: "20px",
          backgroundColor: "#fff3cd",
          border: "2px solid #ffc107",
          borderRadius: "5px",
          marginBottom: "20px",
          textAlign: "center"
        }}>
            <h2 style={{ color: "#856404", marginTop: "0" }}>⚠️ Estás en el navegador web</h2>
          <p style={{ color: "#856404", fontSize: "16px" }}>
            Esta aplicación está diseñada para ejecutarse como una aplicación de escritorio.
          </p>
          <p style={{ color: "#856404", fontSize: "14px" }}>
            <strong>Por favor:</strong>
            <br />
            1. Cierra esta ventana del navegador
            <br />
            2. Busca la ventana de la aplicación "Ofimat-IA" (debería haberse abierto automáticamente)
            <br />
            3. Si no se abrió, ejecuta <code>npm run dev</code> en la terminal
          </p>
        </div>
      )}
      
      <div className="status">
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <p style={{ margin: 0 }}>
            Estado: <span className={isConnected ? "connected" : "disconnected"}>
              {isConnected ? "Conectado" : "Desconectado"}
            </span>
            {isTauri && <span style={{ marginLeft: "10px", fontSize: "12px", color: "#666" }}>(Aplicación Tauri)</span>}
          </p>
          {isConnected && isTauri && (
            <button
              onClick={async () => {
                if (confirm("¿Desconectar de Google? Necesitarás volver a autenticarte para usar la aplicación.")) {
                  try {
                    await invoke<boolean>("disconnect_google");
                    setIsConnected(false);
                    setEvents([]);
                    alert("✅ Desconectado. Vuelve a conectar para obtener los nuevos permisos (incluye escritura en calendario).");
                  } catch (error) {
                    console.error("Error al desconectar:", error);
                    alert(`Error al desconectar: ${error}`);
                  }
                }
              }}
              style={{
                padding: "5px 15px",
                fontSize: "12px",
                border: "1px solid #ccc",
                borderRadius: "4px",
                backgroundColor: "#f5f5f5",
                cursor: "pointer"
              }}
            >
              Desconectar
            </button>
          )}
        </div>
      </div>

      {!isConnected && isTauri && (
        <div className="connect-section">
          <button 
            onClick={connectGoogle} 
            disabled={loading}
            className="connect-button"
          >
            {loading ? "Conectando..." : "Conectar con Google"}
          </button>
          <div style={{ marginTop: "15px" }}>
            <button 
              onClick={connectGoogleManual}
              disabled={false}
              className="connect-button"
              style={{ 
                backgroundColor: "#666",
                display: "block",
                width: "100%",
                maxWidth: "300px",
                margin: "0 auto",
                cursor: "pointer",
                opacity: 1
              }}
            >
              Ingresar código manualmente
            </button>
            <p style={{ 
              marginTop: "10px", 
              fontSize: "12px", 
              color: "#666", 
              textAlign: "center",
              maxWidth: "500px",
              marginLeft: "auto",
              marginRight: "auto"
            }}>
              Si el servidor HTTP no funciona, puedes copiar el código de autorización de la URL del navegador después de hacer clic en "Conectar con Google".
            </p>
          </div>
        </div>
      )}

      {isConnected && (
        <div className="events-section">
          <div className="events-header">
            <h2>Eventos del Calendario</h2>
            <div style={{ display: "flex", gap: "10px" }}>
              <button
                onClick={() => {
                  setShowAIChat(!showAIChat);
                  setShowCreateForm(false);
                }}
                className="refresh-button"
                style={{ backgroundColor: "#9c27b0" }}
              >
                🤖 Crear con IA
              </button>
              <button 
                onClick={() => {
                  setShowCreateForm(!showCreateForm);
                  setShowAIChat(false);
                }}
                className="refresh-button"
                style={{ backgroundColor: "#4caf50" }}
              >
                {showCreateForm ? "Cancelar" : "+ Nuevo Evento"}
              </button>
              <button onClick={loadEvents} className="refresh-button">
                Actualizar
              </button>
            </div>
          </div>
          
          {showAIChat && (
            <div 
              ref={formRef}
              style={{
                marginTop: "20px",
                padding: "20px",
                backgroundColor: "#f3e5f5",
                borderRadius: "8px",
                border: "1px solid #9c27b0"
              }}>
              <h3 style={{ marginTop: "0", color: "#7b1fa2" }}>🤖 Crear Evento con IA</h3>
              <p style={{ fontSize: "14px", color: "#666", marginBottom: "15px" }}>
                Describe el evento en lenguaje natural. Ejemplo: "Reunión con Juan el próximo martes a las 3pm"
              </p>
              <textarea
                value={aiInstruction}
                onChange={(e) => setAiInstruction(e.target.value)}
                placeholder="Ej: Reunión con el equipo el viernes a las 10am, en la oficina principal"
                style={{
                  width: "100%",
                  padding: "12px",
                  fontSize: "14px",
                  border: "1px solid #9c27b0",
                  borderRadius: "4px",
                  minHeight: "100px",
                  resize: "vertical",
                  fontFamily: "inherit"
                }}
                disabled={aiLoading}
              />
              <div style={{ display: "flex", gap: "10px", marginTop: "15px", justifyContent: "flex-end" }}>
                <button
                  onClick={() => {
                    setShowAIChat(false);
                    setAiInstruction("");
                  }}
                  style={{
                    padding: "10px 20px",
                    fontSize: "14px",
                    border: "1px solid #ccc",
                    borderRadius: "4px",
                    backgroundColor: "#f5f5f5",
                    cursor: "pointer"
                  }}
                  disabled={aiLoading}
                >
                  Cancelar
                </button>
                <button
                  onClick={createEventWithAI}
                  disabled={aiLoading || !aiInstruction.trim()}
                  style={{
                    padding: "10px 20px",
                    fontSize: "14px",
                    border: "none",
                    borderRadius: "4px",
                    backgroundColor: aiLoading || !aiInstruction.trim() ? "#ccc" : "#9c27b0",
                    color: "white",
                    cursor: aiLoading || !aiInstruction.trim() ? "not-allowed" : "pointer"
                  }}
                >
                  {aiLoading ? "🤖 Procesando..." : "✨ Crear Evento"}
                </button>
              </div>
            </div>
          )}

          {showCreateForm && (
            <div 
              ref={formRef}
              style={{
                marginTop: "20px",
                padding: "20px",
                backgroundColor: "#f5f5f5",
                borderRadius: "8px",
                border: "1px solid #ddd"
              }}>
              <h3 style={{ marginTop: "0" }}>{editingEvent ? "Editar Evento" : "Crear Nuevo Evento"}</h3>
              
              <div style={{ marginBottom: "15px" }}>
                <label style={{ display: "block", marginBottom: "5px", fontWeight: "bold" }}>
                  Título *:
                </label>
                <input
                  type="text"
                  value={newEvent.title}
                  onChange={(e) => setNewEvent({ ...newEvent, title: e.target.value })}
                  style={{
                    width: "100%",
                    padding: "8px",
                    fontSize: "14px",
                    border: "1px solid #ccc",
                    borderRadius: "4px"
                  }}
                  placeholder="Ej: Reunión de equipo"
                />
              </div>
              
              <div style={{ marginBottom: "15px" }}>
                <label style={{ display: "flex", alignItems: "center", gap: "8px" }}>
                  <input
                    type="checkbox"
                    checked={newEvent.isAllDay}
                    onChange={(e) => setNewEvent({ ...newEvent, isAllDay: e.target.checked })}
                  />
                  <span>Evento de todo el día</span>
                </label>
              </div>
              
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "15px", marginBottom: "15px" }}>
                <div>
                  <label style={{ display: "block", marginBottom: "5px", fontWeight: "bold" }}>
                    Fecha de inicio *:
                  </label>
                  <input
                    type="date"
                    value={newEvent.startDate}
                    onChange={(e) => setNewEvent({ ...newEvent, startDate: e.target.value })}
                    style={{
                      width: "100%",
                      padding: "8px",
                      fontSize: "14px",
                      border: "1px solid #ccc",
                      borderRadius: "4px"
                    }}
                  />
                </div>
                
                {!newEvent.isAllDay && (
                  <div>
                    <label style={{ display: "block", marginBottom: "5px", fontWeight: "bold" }}>
                      Hora de inicio *:
                    </label>
                    <input
                      type="time"
                      value={newEvent.startTime}
                      onChange={(e) => setNewEvent({ ...newEvent, startTime: e.target.value })}
                      style={{
                        width: "100%",
                        padding: "8px",
                        fontSize: "14px",
                        border: "1px solid #ccc",
                        borderRadius: "4px"
                      }}
                    />
                  </div>
                )}
              </div>
              
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "15px", marginBottom: "15px" }}>
                <div>
                  <label style={{ display: "block", marginBottom: "5px", fontWeight: "bold" }}>
                    Fecha de fin *:
                  </label>
                  <input
                    type="date"
                    value={newEvent.endDate}
                    onChange={(e) => setNewEvent({ ...newEvent, endDate: e.target.value })}
                    style={{
                      width: "100%",
                      padding: "8px",
                      fontSize: "14px",
                      border: "1px solid #ccc",
                      borderRadius: "4px"
                    }}
                  />
                </div>
                
                {!newEvent.isAllDay && (
                  <div>
                    <label style={{ display: "block", marginBottom: "5px", fontWeight: "bold" }}>
                      Hora de fin *:
                    </label>
                    <input
                      type="time"
                      value={newEvent.endTime}
                      onChange={(e) => setNewEvent({ ...newEvent, endTime: e.target.value })}
                      style={{
                        width: "100%",
                        padding: "8px",
                        fontSize: "14px",
                        border: "1px solid #ccc",
                        borderRadius: "4px"
                      }}
                    />
                  </div>
                )}
              </div>
              
              <div style={{ marginBottom: "15px" }}>
                <label style={{ display: "block", marginBottom: "5px", fontWeight: "bold" }}>
                  Ubicación:
                </label>
                <input
                  type="text"
                  value={newEvent.location}
                  onChange={(e) => setNewEvent({ ...newEvent, location: e.target.value })}
                  style={{
                    width: "100%",
                    padding: "8px",
                    fontSize: "14px",
                    border: "1px solid #ccc",
                    borderRadius: "4px"
                  }}
                  placeholder="Ej: Oficina principal"
                />
              </div>
              
              <div style={{ marginBottom: "15px" }}>
                <label style={{ display: "block", marginBottom: "5px", fontWeight: "bold" }}>
                  Descripción:
                </label>
                <textarea
                  value={newEvent.description}
                  onChange={(e) => setNewEvent({ ...newEvent, description: e.target.value })}
                  style={{
                    width: "100%",
                    padding: "8px",
                    fontSize: "14px",
                    border: "1px solid #ccc",
                    borderRadius: "4px",
                    minHeight: "80px",
                    resize: "vertical"
                  }}
                  placeholder="Descripción del evento..."
                />
              </div>
              
              <div style={{ marginBottom: "15px" }}>
                <label style={{ display: "block", marginBottom: "5px", fontWeight: "bold" }}>
                  Participantes (emails):
                </label>
                <textarea
                  value={newEvent.attendees}
                  onChange={(e) => setNewEvent({ ...newEvent, attendees: e.target.value })}
                  style={{
                    width: "100%",
                    padding: "8px",
                    fontSize: "14px",
                    border: "1px solid #ccc",
                    borderRadius: "4px",
                    minHeight: "60px",
                    resize: "vertical"
                  }}
                  placeholder="email1@ejemplo.com, email2@ejemplo.com&#10;O uno por línea"
                />
                <p style={{ fontSize: "12px", color: "#666", marginTop: "5px" }}>
                  Separa los emails por comas o uno por línea
                </p>
              </div>
              
              <div style={{ display: "flex", gap: "10px", justifyContent: "flex-end" }}>
                <button
                  onClick={() => {
                    setShowCreateForm(false);
                    setEditingEvent(null);
                    setNewEvent({
                      title: "",
                      startDate: "",
                      startTime: "",
                      endDate: "",
                      endTime: "",
                      description: "",
                      location: "",
                      isAllDay: false,
                      attendees: "",
                    });
                  }}
                  style={{
                    padding: "10px 20px",
                    fontSize: "14px",
                    border: "1px solid #ccc",
                    borderRadius: "4px",
                    backgroundColor: "#f5f5f5",
                    cursor: "pointer"
                  }}
                >
                  Cancelar
                </button>
                <button
                  onClick={editingEvent ? updateEvent : createEvent}
                  disabled={loading}
                  style={{
                    padding: "10px 20px",
                    fontSize: "14px",
                    border: "none",
                    borderRadius: "4px",
                    backgroundColor: loading ? "#ccc" : "#4caf50",
                    color: "white",
                    cursor: loading ? "not-allowed" : "pointer",
                    fontWeight: "bold"
                  }}
                >
                  {loading 
                    ? (editingEvent ? "Actualizando..." : "Creando...") 
                    : (editingEvent ? "Actualizar Evento" : "Crear Evento")}
                </button>
              </div>
            </div>
          )}
          
          {events.length === 0 ? (
            <p>No hay eventos próximos (próximos 60 días)</p>
          ) : (
            <ul className="events-list">
              {events.map((event) => (
                <li key={event.id} className="event-item">
                  <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
                    <div style={{ flex: 1 }}>
                      <strong>{event.title}</strong>
                      <div className="event-time">
                        <span style={{ fontWeight: "bold" }}>Inicio:</span> {formatEventDate(event.start, event.is_all_day)}
                        <br />
                        <span style={{ fontWeight: "bold" }}>Fin:</span> {formatEventDate(event.end, event.is_all_day)}
                      </div>
                      {event.location && (
                        <div className="event-location" style={{ marginTop: "5px", color: "#666", fontSize: "0.9em" }}>
                          📍 {event.location}
                        </div>
                      )}
                      {event.description && (
                        <div 
                          className="event-description" 
                          style={{ 
                            marginTop: "5px", 
                            color: "#666", 
                            fontSize: "0.85em",
                            whiteSpace: "pre-wrap",
                            wordWrap: "break-word"
                          }}
                        >
                          {event.description.length > 200 
                            ? event.description.substring(0, 200) + "..." 
                            : event.description}
                        </div>
                      )}
                      {event.attendees && event.attendees.length > 0 && (
                        <div className="event-attendees" style={{ marginTop: "8px", fontSize: "0.85em" }}>
                          <span style={{ fontWeight: "bold", color: "#666" }}>👥 Participantes:</span>
                          <div style={{ marginTop: "3px", color: "#666" }}>
                            {event.attendees.map((email, idx) => (
                              <span key={idx} style={{ display: "inline-block", marginRight: "8px", marginBottom: "3px" }}>
                                {email}
                              </span>
                            ))}
                          </div>
                        </div>
                      )}
                    </div>
                    <div style={{ display: "flex", gap: "5px", marginLeft: "10px" }}>
                      <button
                        onClick={() => startEditEvent(event)}
                        disabled={loading}
                        style={{
                          padding: "5px 10px",
                          fontSize: "12px",
                          border: "1px solid #4285f4",
                          borderRadius: "4px",
                          backgroundColor: "#fff",
                          color: "#4285f4",
                          cursor: loading ? "not-allowed" : "pointer"
                        }}
                        title="Editar evento"
                      >
                        ✏️
                      </button>
                      <button
                        onClick={() => deleteEvent(event.id, event.title)}
                        disabled={loading}
                        style={{
                          padding: "5px 10px",
                          fontSize: "12px",
                          border: "1px solid #f44336",
                          borderRadius: "4px",
                          backgroundColor: "#fff",
                          color: "#f44336",
                          cursor: loading ? "not-allowed" : "pointer"
                        }}
                        title="Eliminar evento"
                      >
                        🗑️
                      </button>
                    </div>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}

export default App;
