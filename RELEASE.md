# DiscordQuest v1.3.0

Esta versión parte nuevamente de la base estable de v1.1.0 e incorpora el modo Steam de forma aislada.

## Novedades

- Nuevo flujo Steam solo para juegos sin ejecutables Win32 publicados por Discord y con AppID de Steam disponible.
- Creación temporal del runner en `Steam\\steamapps\\common\\<juego>` y de `appmanifest_<appid>.acf` con `StateFlags=1026`.
- Consulta de la ruta y el ejecutable mediante los metadatos públicos de SteamCMD.
- Restauración automática de cualquier ejecutable o appmanifest preexistente.
- Limpieza automática al detener el juego, cerrar el runner o salir de DiscordQuest.
- Journal de recuperación para completar la limpieza después de un cierre inesperado.
- Inicio del runner más tolerante en Windows 10 y Windows 11; un fallo de la bandeja ya no impide abrir su ventana.
- Se conserva sin cambios el flujo normal para los juegos que sí publican una ruta ejecutable válida.

## Requisitos del modo Steam

- Steam debe estar instalado y registrado en Windows.
- No es obligatorio mantener Steam abierto para iniciar el runner.
- Se necesita conexión al iniciar para consultar los metadatos de SteamCMD.

## Correcciones

- Separación entre procesos normales, RPC y Steam para evitar detener el proceso equivocado.
- Prevención de dobles inicios y de sesiones Steam superpuestas.
- Validación de rutas para impedir escrituras fuera de `steamapps\\common`.
- Empaquetado reproducible del runner antes de construir el instalador.
