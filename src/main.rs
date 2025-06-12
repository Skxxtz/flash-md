use std::{cell::{Cell, RefCell}, collections::VecDeque, fs::File, io::{BufRead, BufReader}};

use gtk4::{
    gdk::Key, gio::{
        prelude::{ApplicationExt, ApplicationExtManual}, ApplicationFlags,

    }, glib::{object::ObjectExt, WeakRef}, prelude::{BoxExt, EventControllerExt, GtkWindowExt, WidgetExt}, Application, ApplicationWindow, Box, EventControllerKey, Label, TextView
};
use gtk4_layer_shell::LayerShell;

mod loaders;
mod utils;
use loaders::Loader;
use rand::{rngs::ThreadRng, Rng};
use utils::errors::FlashError;

fn main() {
    let app = create_application();

    app.connect_activate({
        move |app| {
            let _ = Loader::load_resources();
            let _ = Loader::load_css();

            if let Ok(cards) = load_cards("/home/basti/uni/computer-vision/vocab.md"){
                let window = create_window(app);
                let ui = build_ui(&window, cards);

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
    history: VecDeque<bool>
}


fn load_cards(path: &str) -> Result<Vec<Card>, FlashError> {
    let file = File::open(path).map_err(|e| FlashError {
        error: utils::errors::FlashErrorType::FileReadError(path.to_string()),
        traceback: e.to_string()
    })?;
    let reader = BufReader::new(file);
    
    let mut current_title = None;
    let mut current_body = String::new();
    let mut cards = Vec::new();
    reader.lines().into_iter().filter_map(Result::ok).for_each(|line| {
        if let Some(stripped) = line.strip_prefix("# "){
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
            history: VecDeque::new(),
        });
    }
}

struct UI {
    buffer: WeakRef<TextView>,
}
fn build_ui(win: &ApplicationWindow, cards: Vec<Card>){
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
    let holder =Box::builder()
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
        let rng: RefCell<ThreadRng>= RefCell::new(rand::rng());
        let current_page: Cell<u8> = Cell::new(0);
        let view = text_view.downgrade();
        let current_index: Cell<usize> = Cell::new(rng.borrow_mut().random_range(0..=num_cards));
        
        if let Some(card) = cards.get(current_index.get()){
            title.set_text(&card.title);
        }
        let title = title.downgrade();
        move |_controller, key, _code, _mods| {
            match key {
                Key::Escape => {
                    if let Some(win) = win.upgrade(){
                        win.close();
                        return true.into();
                    }
                }
                Key::Return => {
                    if current_page.get() == 1 {
                        current_page.set(0); // flip page to front
                        // next card
                        let index = rng.borrow_mut().random_range(0..=num_cards);
                        if let Some(card) = cards.get(index){
                            current_index.set(index);
                            if let Some(view) = view.upgrade(){
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
                        if let Some(card) = cards.get(current_index.get()){
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
