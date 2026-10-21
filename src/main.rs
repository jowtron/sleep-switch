//! Sleep Switch: a menu bar toggle for `pmset disablesleep`.
//!
//! With sleep disabled the Mac stays awake even when the lid is closed or
//! Sleep is chosen from the Apple menu. The switch re-reads the real setting
//! every few seconds, so changes made in Terminal show up too.

mod icons;
mod power;

use smappservice_rs::{AppService, ServiceStatus, ServiceType};
use std::time::{Duration, Instant};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

const POLL: Duration = Duration::from_secs(3);

enum UserEvent {
    Menu(MenuEvent),
    /// A background privileged action finished; Err carries a message to show.
    Done(Result<(), String>),
}

struct Ui {
    tray: TrayIcon,
    status: MenuItem,
    keep_awake: CheckMenuItem,
    password_free: MenuItem,
    login: CheckMenuItem,
    quit: MenuItem,
    shown: Option<bool>,
}

impl Ui {
    fn new() -> Self {
        let status = MenuItem::new("Reading pmset…", false, None);
        let keep_awake = CheckMenuItem::new("Keep Mac Awake (even with lid closed)", true, false, None);
        let password_free = MenuItem::new("", true, None);
        let login = CheckMenuItem::new("Open at Login", true, false, None);
        let quit = MenuItem::new("Quit Sleep Switch", true, None);

        let menu = Menu::new();
        menu.append_items(&[
            &status,
            &keep_awake,
            &PredefinedMenuItem::separator(),
            &password_free,
            &login,
            &PredefinedMenuItem::separator(),
            &quit,
        ])
        .expect("build menu");

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_icon(icons::moon())
            .with_icon_as_template(true)
            .with_tooltip("Sleep Switch")
            .build()
            .expect("create tray icon");

        let mut ui = Ui { tray, status, keep_awake, password_free, login, quit, shown: None };
        ui.refresh();
        ui
    }

    /// Bring every item in line with the real system state.
    fn refresh(&mut self) {
        let state = power::sleep_disabled();
        if state != self.shown {
            let (icon, text, tip) = match state {
                Some(true) => (icons::sun(), "Sleep is OFF: Mac stays awake", "Sleep Switch: staying awake"),
                Some(false) => (icons::moon(), "Sleep is on: Mac sleeps normally", "Sleep Switch: sleeping normally"),
                None => (icons::moon(), "Couldn't read pmset", "Sleep Switch"),
            };
            let _ = self.tray.set_icon_with_as_template(Some(icon), true);
            let _ = self.tray.set_tooltip(Some(tip));
            self.status.set_text(text);
            self.shown = state;
        }
        // Menu clicks flip a CheckMenuItem on their own; always put it back to the truth.
        self.keep_awake.set_checked(state == Some(true));
        self.keep_awake.set_enabled(state.is_some());
        self.password_free.set_text(if power::password_free_installed() {
            "Require Password to Switch…"
        } else {
            "Switch Without Password…"
        });
        self.login.set_checked(login_service().status() == ServiceStatus::Enabled);
    }
}

fn login_service() -> AppService {
    AppService::new(ServiceType::MainApp)
}

fn main() {
    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    // tao defaults to a Dock icon regardless of LSUIElement; this keeps it menu-bar only.
    event_loop.set_activation_policy(ActivationPolicy::Accessory);

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |e| {
        let _ = proxy.send_event(UserEvent::Menu(e));
    }));
    let proxy = event_loop.create_proxy();

    let mut ui: Option<Ui> = None;
    let mut busy = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + POLL);

        match event {
            // The tray icon has to be created once the app has finished launching.
            Event::NewEvents(StartCause::Init) => ui = Some(Ui::new()),
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                if let Some(ui) = ui.as_mut() {
                    ui.refresh();
                }
            }
            Event::UserEvent(UserEvent::Menu(e)) => {
                let Some(ui) = ui.as_mut() else { return };
                if e.id == ui.quit.id() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                if e.id == ui.login.id() {
                    let svc = login_service();
                    let r = if svc.status() == ServiceStatus::Enabled { svc.unregister() } else { svc.register() };
                    if let Err(err) = r {
                        eprintln!("login item: {err}");
                    }
                    if svc.status() == ServiceStatus::RequiresApproval {
                        AppService::open_system_settings_login_items();
                    }
                    ui.refresh();
                    return;
                }
                // The rest may raise the admin password dialog, so run them off the main thread.
                let action: Option<fn() -> Result<(), String>> = if e.id == ui.keep_awake.id() {
                    Some(if power::sleep_disabled() == Some(true) {
                        || power::set_sleep_disabled(false)
                    } else {
                        || power::set_sleep_disabled(true)
                    })
                } else if e.id == ui.password_free.id() {
                    Some(if power::password_free_installed() {
                        power::remove_password_free
                    } else {
                        power::install_password_free
                    })
                } else {
                    None
                };
                ui.refresh();
                if let (Some(action), false) = (action, busy) {
                    busy = true;
                    let proxy = proxy.clone();
                    std::thread::spawn(move || {
                        let _ = proxy.send_event(UserEvent::Done(action()));
                    });
                }
            }
            Event::UserEvent(UserEvent::Done(result)) => {
                busy = false;
                let Some(ui) = ui.as_mut() else { return };
                ui.refresh();
                if let Err(err) = result {
                    if err != "cancelled" {
                        eprintln!("sleep-switch: {err}");
                        // Shown until the next poll redraws the status line.
                        ui.status.set_text(format!("Failed: {err}"));
                        ui.shown = None;
                    }
                }
            }
            _ => {}
        }
    });
}
