use tray_icon::menu::Menu;
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub fn create_tray_icon(tray_menu: Menu, title: &str) -> Result<TrayIcon, String> {
    let icon = Icon::from_resource(1, None)
        .map_err(|error| format!("No se pudo cargar el icono: {}", error))?;

    TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip(title)
        .with_icon(icon)
        .build()
        .map_err(|error| format!("No se pudo crear el icono de bandeja: {}", error))
}
