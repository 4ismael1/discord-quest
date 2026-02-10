# DiscordQuest v1.0.0

Simulador de actividad de juegos verificados de Discord para completar Discord Quests sin instalar los juegos reales.

## Descargar

📦 **[DiscordQuest_1.0.0_x64-setup.exe]()**  
Instalador para Windows (x64) — No requiere permisos de administrador.

> WebView2 se instala automáticamente si no lo tienes. Viene preinstalado en Windows 11.

## Características

- Simula jugar juegos verificados de Discord sin instalarlos
- Completa Discord Quests que requieren 15 minutos de juego
- Búsqueda rápida entre +21,000 juegos detectables por Discord
- Lista de juegos persistente entre sesiones
- Detección automática cuando el juego se cierra externamente
- Ventana fake con timer y bandeja del sistema
- UI optimista con respuesta instantánea
- Ciclo de vida bidireccional entre la app y la ventana fake

## Tech Stack

- **Rust** — Backend (Tauri v2) + Ventana fake (Win32 API)
- **Vue.js 3** — Frontend con Composition API + TypeScript
- **Fuse.js** — Búsqueda difusa de juegos

## Instalación

1. Descarga `DiscordQuest_1.0.0_x64-setup.exe`
2. Ejecuta el instalador
3. Abre **DiscordQuest** desde el menú de inicio
4. Abre Discord y busca un Quest activo
5. Selecciona el juego requerido y dale Play

## Aviso

Esta herramienta es para fines educativos y uso personal. Úsala bajo tu propia responsabilidad.
