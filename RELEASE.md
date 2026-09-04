# DiscordQuest v1.4.0

## Novedades

- Nueva sección **Configuración**.
- Opción para ejecutar varias quests al mismo tiempo. Está desactivada por defecto para conservar el comportamiento anterior.
- Política configurable de actualización de la API: al iniciar, diaria, semanal o manual.
- Botón para actualizar la lista de juegos inmediatamente.

## Mejoras

- Cada juego Steam utiliza una sesión y un journal independientes, permitiendo varias simulaciones simultáneas sin mezclar su limpieza.
- La lista almacenada aparece de inmediato y la comprobación remota ya no se duplica al abrir la aplicación.
- El contador de procesos incluye tanto simulaciones normales como Steam.
- Al cerrar un proceso solo se detiene y elimina el estado correspondiente a ese juego.
- La interfaz muestra la versión real instalada en lugar de un valor fijo.

## Notas

- Con la ejecución múltiple desactivada, iniciar un juego detiene el anterior como en las versiones previas.
- Los archivos temporales del modo Steam continúan restaurándose o eliminándose al detener la simulación; los journals pendientes también se recuperan en el siguiente inicio.
