<p>
  <h1 align="center">DiscordQuest</h1>
</p>

<p align="center">
  Aplicacion de escritorio para Windows que simula actividad de juegos verificados de Discord, permitiendo completar Discord Quests sin necesidad de instalar los juegos reales.
</p>

<p align="center">
  <img src="Home1.png" alt="DiscordQuest Home" width="65%" />
</p>

---

## Caracteristicas

- Simula jugar juegos verificados de Discord sin instalarlos
- Completa Discord Quests que requieren 15 minutos de juego
- Busqueda rapida entre +21,000 juegos detectables por Discord
- Lista de juegos persistente entre sesiones
- Deteccion automatica cuando el juego se cierra externamente
- Ventana fake con timer y bandeja del sistema
- Actualizacion optimista de la UI (respuesta instantanea)
- Ciclo de vida bidireccional entre la app y la ventana fake
- Terminacion graceful de procesos

## Como funciona

La app crea pequenos ejecutables (~250KB) que imitan los procesos que Discord busca para detectar juegos verificados. Al ejecutarlos, Discord los reconoce como si fuera el juego real y activa Rich Presence.

Los ejecutables se almacenan en una carpeta `games/` relativa al ejecutable principal:

```
DiscordQuest/
+-- DiscordQuest.exe
+-- data/
|   +-- src-win.exe
+-- games/
    +-- <app-id>/
        +-- <ruta-del-juego>/
            +-- nombre-del-juego.exe
```

> [!TIP]
> Con el tiempo estos archivos pueden acumularse. Puedes eliminar manualmente las carpetas dentro de `games/` cuando quieras.

---

## Instalacion

Descarga `DiscordQuest_x64-setup.exe` desde [Releases](../../releases), ejecutalo e instala. No requiere permisos de administrador.

> [!NOTE]
> WebView2 se instala automaticamente si no lo tienes. Viene preinstalado en Windows 11.

---

## Tech Stack

- **Rust** - Backend (Tauri v2) + Ventana fake (Win32 API)
- **Vue.js 3** - Frontend con Composition API + TypeScript
- **Fuse.js** - Busqueda difusa de juegos

---

## Desarrollo

### Requisitos

- [Rust](https://www.rust-lang.org/tools/install) y los [prerrequisitos de Tauri](https://tauri.app/start/prerequisites/)
- [Node.js](https://nodejs.org/) 20+

### Setup

```bash
npm install

cd src-win
cargo build --release
cd ..
mkdir -p src-tauri/resources
cp src-win/target/release/src-win.exe src-tauri/resources/src-win.exe

npx tauri dev
```

### Build

```bash
npx tauri build
```

---

## Aviso legal

Esta herramienta es para fines educativos y uso personal. Respeta los terminos de servicio de Discord, los derechos de los editores de juegos y los anunciantes.

Los creadores y mantenedores de este proyecto no son responsables de danos, suspensiones de cuenta u otras consecuencias derivadas del uso de este software. **Usalo bajo tu propia responsabilidad.**

Discord es una marca registrada de Discord Inc. Se hace referencia a ella unicamente con fines descriptivos.

---

## Licencia

[MIT License](LICENSE) - Basado en trabajo de [Mark Terence Tiglao](https://github.com/markterence).

---

Desarrollado por [4ismael1](https://github.com/4ismael1)
