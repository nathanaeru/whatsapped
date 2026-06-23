use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    plugin::{Builder as PluginBuilder, TauriPlugin},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, Runtime, WindowEvent,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;

// Custom plugin to route WhatsApp web notifications to the native OS
fn notification_hijack_plugin<R: Runtime>() -> TauriPlugin<R> {
    let script = r#"
        function triggerTauriNotification(title, msgOptions) {
            try {
                if (window.__TAURI__ && window.__TAURI__.core) {
                    window.__TAURI__.core.invoke("plugin:notification|notify", {
                        options: { // <-- Wrapped in options!
                            title: title || 'WhatsApp',
                            body: msgOptions ? msgOptions.body : ''
                        }
                    });
                }
            } catch (e) {
                console.error("Failed to trigger native notification", e);
            }
        }

        window.Notification = class Notification {
            constructor(title, options) { triggerTauriNotification(title, options); }
            static requestPermission() { return Promise.resolve('granted'); }
            static get permission() { return 'granted'; }
        };

        if (window.ServiceWorkerRegistration) {
            window.ServiceWorkerRegistration.prototype.showNotification = function(title, options) {
                triggerTauriNotification(title, options);
                return Promise.resolve();
            };
        }
    "#;

    PluginBuilder::<R>::new("notification-hijack")
        .js_init_script(script.to_string())
        .build()
}

// Custom plugin to intercept link clicks and route external links to the OS browser
fn navigation_hijack_plugin<R: Runtime>() -> TauriPlugin<R> {
    PluginBuilder::<R>::new("navigation-hijack")
        .on_navigation(|webview, url| {
            let host = url.host_str().unwrap_or("");

            // Allow internal WhatsApp routing, local files, and Tauri protocols
            if host.contains("whatsapp.com")
                || host.contains("whatsapp.net")
                || url.scheme() == "tauri"
                || host.is_empty()
            {
                true // Allow the webview to load the page
            } else {
                // It's an external link! Route it to the OS default browser
                let _ = webview
                    .app_handle()
                    .opener()
                    .open_url(url.as_str(), None::<&str>);
                false // Block the webview from navigating away from WhatsApp
            }
        })
        .build()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ));

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(
            |app: &tauri::AppHandle, args, _cwd| {
                if let Some(window) = app.get_webview_window("main") {
                    // Unhide and focus the window
                    if !window.is_visible().unwrap_or(false) {
                        let _ = window.show();
                    }
                    let _ = window.set_focus();

                    // Parse the incoming deep link from the arguments
                    // args contains the launch parameters. Deep links usually appear as the last argument.
                    for arg in args {
                        if arg.starts_with("whatsapp://") || arg.starts_with("wapped://") {
                            // WhatsApp web uses web.whatsapp.com for deep linking routing
                            // We translate the native protocol into the web protocol
                            let web_url = arg
                                .replace("whatsapp://", "https://web.whatsapp.com/")
                                .replace("wapped://", "https://web.whatsapp.com/");

                            // Tell the webview to navigate to the new link
                            let script = format!("window.location.href = '{}';", web_url);
                            let _ = window.eval(&script);
                            break;
                        }
                    }
                }
            },
        ));
    }

    builder
        .plugin(tauri_plugin_notification::init()) //  Enable Native Notifications
        .plugin(notification_hijack_plugin()) //  Inject our Hijacker
        .plugin(navigation_hijack_plugin()) //  Inject our Navigation Hijacker
        .setup(|app| {
            // Handle deep link startup arguments if launched via protocol (first instance)
            if let Some(window) = app.get_webview_window("main") {
                for arg in std::env::args().skip(1) {
                    if arg.starts_with("whatsapp://") || arg.starts_with("wapped://") {
                        let web_url = arg
                            .replace("whatsapp://", "https://web.whatsapp.com/")
                            .replace("wapped://", "https://web.whatsapp.com/");
                        if let Ok(parsed_url) = tauri::Url::parse(&web_url) {
                            let _ = window.navigate(parsed_url);
                        }
                        break;
                    }
                }
            }

            let autostart_manager = app.autolaunch();
            let is_start_enabled = autostart_manager.is_enabled().unwrap_or(false);

            // Build the System Tray
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show app window", true, None::<&str>)?;

            // Create the "Open on startup" checkbox item
            let startup_i = CheckMenuItem::with_id(
                app,
                "toggle_startup",
                "Open on startup",
                true,
                is_start_enabled,
                None::<&str>,
            )?;

            let menu = Menu::with_items(app, &[&show_i, &startup_i, &quit_i])?;

            let menu_handle = menu.clone();

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "toggle_startup" => {
                        // Directly pattern match on the Check variant of the MenuItemKind enum
                        if let Some(tauri::menu::MenuItemKind::Check(item)) =
                            menu_handle.get("toggle_startup")
                        {
                            let is_checked = item.is_checked().unwrap_or(false);
                            let am = app.autolaunch();

                            if is_checked {
                                let _ = am.enable();
                            } else {
                                let _ = am.disable();
                            }
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    // Restrict window focusing strictly to Left Clicks
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // Hide to Tray instead of quitting when hitting "X"
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
