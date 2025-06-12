use std::{
    cell::{Cell, RefCell},
    env,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use gtk4::{
    Application, ApplicationWindow, Box, EventControllerKey, Label, TextView,
    gdk::Key,
    gio::{
        ApplicationFlags,
        prelude::{ApplicationExt, ApplicationExtManual},
    },
    glib::{WeakRef, object::ObjectExt},
    prelude::{BoxExt, EventControllerExt, GtkWindowExt, WidgetExt},
};
use gtk4_layer_shell::LayerShell;

mod loaders;
mod utils;
use loaders::Loader;
use rand::{Rng, rngs::ThreadRng};
use utils::errors::FlashError;

fn expand_path(input: &str) -> PathBuf {
    // Handle tilde expansion (~) manually
    if input.starts_with("~/") || input == "~" {
        let home_dir = env::home_dir().expect("Could not get home directory");
        if input == "~" {
            return home_dir;
        } else {
            return home_dir.join(&input[2..]);
        }
    }

    let path = Path::new(input);

    if path.exists() {
        path.to_path_buf()
    } else {
        let current_dir = env::current_dir().expect("Failed to get current directory");
        current_dir.join(path)
    }
}

fn main() {
    let app = create_application();

    let args: Vec<String> = env::args().collect();
    let file = args
        .get(1)
        .expect("There was no file provided. Usecase: flash <file>");
    let path = expand_path(file);
    if !path.exists() || !path.is_file() {
        panic!("The first argument was not a file.")
    }

    app.connect_activate({
        move |app| {
            let _ = Loader::load_resources();
            let _ = Loader::load_css();

            if let Ok(cards) = load_cards(&path) {
                let window = create_window(app);
                build_ui(&window, cards);

                window.present();
                window.grab_focus();
            }
        }
    });
    app.connect_command_line(|app, _| {
        app.activate();
        0
    });
    app.run();
}

fn create_application() -> Application {
    let app = Application::builder()
        .flags(ApplicationFlags::HANDLES_COMMAND_LINE | ApplicationFlags::NON_UNIQUE)
        .application_id("dev.skxxtz.flashmd")
        .build();
    app
}
fn create_window(app: &Application) -> ApplicationWindow {
    // setup window
    let win = ApplicationWindow::builder()
        .name("flash-md.main")
        .width_request(600)
        .height_request(200)
        .vexpand(false)
        .hexpand(false)
        .resizable(false)
        .application(app)
        .decorated(false)
        .focusable(true)
        .sensitive(true)
        .can_focus(true)
        .build();

    // setup layershell capabilities
    win.init_layer_shell();
    win.set_layer(gtk4_layer_shell::Layer::Overlay);
    win.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);

    win
}

#[derive(Debug)]
pub struct Card {
    title: String,
    body: String,
}

fn load_cards(path: &PathBuf) -> Result<Vec<Card>, FlashError> {
    let file = File::open(&path).map_err(|e| FlashError {
        error: utils::errors::FlashErrorType::FileReadError(format!("{}", path.display())),
        traceback: e.to_string(),
    })?;
    let reader = BufReader::new(file);

    let mut current_title = None;
    let mut current_body = String::new();
    let mut cards = Vec::new();
    reader
        .lines()
        .into_iter()
        .filter_map(Result::ok)
        .for_each(|line| {
            if let Some(stripped) = line.strip_prefix("# ") {
                push_section(&mut cards, current_title.take(), &current_body);
                current_body.clear();
                let trimmed = stripped.trim().to_string();
                if !trimmed.is_empty() {
                    current_title = Some(trimmed);
                }
            } else {
                current_body.push_str(&line);
                current_body.push_str("\n");
            }
        });
    push_section(&mut cards, current_title, &current_body);

    Ok(cards)
}

fn push_section(sections: &mut Vec<Card>, title: Option<String>, body: &String) {
    if let Some(title) = title {
        sections.push(Card {
            title,
            body: body.to_string(),
        });
    }
}

fn build_ui(win: &ApplicationWindow, cards: Vec<Card>) {
    let title = Label::builder()
        .valign(gtk4::Align::Start)
        .hexpand(true)
        .name("title")
        .build();
    let text_view = Label::builder()
        .valign(gtk4::Align::Start)
        .halign(gtk4::Align::Start)
        .vexpand(true)
        .hexpand(true)
        .name("body")
        .build();
    let holder = Box::builder()
        .name("viewport")
        .vexpand(true)
        .hexpand(true)
        .spacing(10)
        .orientation(gtk4::Orientation::Vertical)
        .build();

    holder.append(&title);
    holder.append(&text_view);

    win.set_child(Some(&holder));

    // setup keyevent
    let key_event = EventControllerKey::new();
    key_event.set_propagation_phase(gtk4::PropagationPhase::Capture);
    key_event.connect_key_pressed({
        let win = win.downgrade();

        let num_cards = cards.len() - 1;
        let rng: RefCell<ThreadRng> = RefCell::new(rand::rng());
        let current_page: Cell<u8> = Cell::new(0);
        let view = text_view.downgrade();
        let current_index: Cell<usize> = Cell::new(rng.borrow_mut().random_range(0..=num_cards));

        if let Some(card) = cards.get(current_index.get()) {
            title.set_text(&card.title);
        }
        let title = title.downgrade();
        move |_controller, key, _code, _mods| {
            match key {
                Key::Escape => {
                    if let Some(win) = win.upgrade() {
                        win.close();
                        return true.into();
                    }
                }
                Key::Return => {
                    if current_page.get() == 1 {
                        current_page.set(0); // flip page to front
                        // next card
                        let index = rng.borrow_mut().random_range(0..=num_cards);
                        if let Some(card) = cards.get(index) {
                            current_index.set(index);
                            if let Some(view) = view.upgrade() {
                                view.set_text("");
                            }
                            if let Some(title) = title.upgrade() {
                                title.set_markup(&card.title);
                            }
                        }

                        // show card title of new
                    } else {
                        current_page.set(1); // flip page to back
                        // show card back
                        if let Some(card) = cards.get(current_index.get()) {
                            if let Some(view) = view.upgrade() {
                                view.set_markup(&card.body);
                            }
                        }
                    }
                }
                _ => {}
            }
            return false.into();
        }
    });

    win.add_controller(key_event);
}
