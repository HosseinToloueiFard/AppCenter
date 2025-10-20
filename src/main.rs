use slint::{ModelRc, VecModel, Image, Model};
use std::cell::RefCell;
use std::rc::Rc;
use std::path::Path;

mod shortcuts;
use crate::shortcuts::{JsonShortcut, ShortcutStore};

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "shortcuts.json";
    let store_rc = Rc::new(RefCell::new(ShortcutStore::load_or_default(path)));

    let all_shortcuts: Vec<AppShortcutData> = store_rc
        .borrow()
        .shortcuts
        .iter()
        .map(|item| AppShortcutData {
            name: item.name.clone().into(),
            icon_path: Image::load_from_path(Path::new(&item.icon_path)).unwrap_or_default(),
            command: item.command.clone().into(),
        })
        .collect();

    let all_shortcuts_model = Rc::new(RefCell::new(VecModel::from(all_shortcuts)));
    let current_page = Rc::new(RefCell::new(0usize));
    let items_per_page = 10;

    let make_page_model = {
        let all_shortcuts_model = all_shortcuts_model.clone();
        move |page: usize| -> ModelRc<ModelRc<AppShortcutData>> {
            let shortcuts = all_shortcuts_model.borrow();
            let start = page * items_per_page;
            let end = std::cmp::min(start + items_per_page, shortcuts.row_count());

            let page_items: Vec<AppShortcutData> = (start..end)
                .filter_map(|i| shortcuts.row_data(i))
                .collect();

            let inner_models: Vec<ModelRc<AppShortcutData>> = page_items
                .chunks(5)
                .map(|chunk| ModelRc::new(VecModel::from(chunk.to_vec())))
                .collect();

            ModelRc::new(VecModel::from(inner_models))
        }
    };

    let ui = AppLauncher::new()?;
    let total_pages = (all_shortcuts_model.borrow().row_count() + items_per_page - 1) / items_per_page;

    ui.set_current_page(0);
    ui.set_shortcut_rows(make_page_model(0));
    ui.set_show_add_modal(false);
    let ui_handle = Rc::new(ui.as_weak());

    // Next page
    {
        let current_page = current_page.clone();
        let make_page_model = make_page_model.clone();
        let all_shortcuts_model = all_shortcuts_model.clone();
        let ui_handle = ui_handle.clone();

        ui.on_next_page(move || {
            let total_pages = (all_shortcuts_model.borrow().row_count() + items_per_page - 1) / items_per_page;
            if let Some(ui) = ui_handle.upgrade() {
                if *current_page.borrow() + 1 < total_pages {
                    *current_page.borrow_mut() += 1;
                    ui.set_current_page(*current_page.borrow() as i32);
                    ui.set_shortcut_rows(make_page_model(*current_page.borrow()));
                }
            }
        });
    }

    // Previous page
    {
        let current_page = current_page.clone();
        let make_page_model = make_page_model.clone();
        let ui_handle = ui_handle.clone();

        ui.on_previous_page(move || {
            if let Some(ui) = ui_handle.upgrade() {
                if *current_page.borrow() > 0 {
                    *current_page.borrow_mut() -= 1;
                    ui.set_current_page(*current_page.borrow() as i32);
                    ui.set_shortcut_rows(make_page_model(*current_page.borrow()));
                }
            }
        });
    }

    // Add shortcut
    {
        let store_rc = store_rc.clone();
        let all_shortcuts_model = all_shortcuts_model.clone();
        let current_page = current_page.clone();
        let make_page_model = make_page_model.clone();
        let ui_handle = ui_handle.clone();

        ui.on_add_shortcut(move |name, icon, command| {
            if let Some(ui) = ui_handle.upgrade() {
                let name_str = name.to_string();
                let icon_str = icon.to_string();
                let command_str = command.to_string();

                if name_str.trim().is_empty() || command_str.trim().is_empty() {
                    println!("Empty name or command. Ignored.");
                    return;
                }
                let data = AppShortcutData {
                    name: name.clone(),
                    icon_path: Image::load_from_path(Path::new(&icon_str)).unwrap_or_default(),
                    command: command.clone(),
                };

                all_shortcuts_model.borrow_mut().push(data);

                let json_data = JsonShortcut {
                    name: name_str.clone(),
                    icon_path: icon_str.clone(),
                    command: command_str.clone(),
                };

                store_rc.borrow_mut().add(json_data);
                if let Err(e) = store_rc.borrow().save(path) {
                    println!("Failed to save shortcuts: {:?}", e);
                }

                let total_pages = (all_shortcuts_model.borrow().row_count() + items_per_page - 1) / items_per_page;
                if *current_page.borrow() >= total_pages {
                    *current_page.borrow_mut() = total_pages - 1;
                }

                ui.set_current_page(*current_page.borrow() as i32);
                ui.set_shortcut_rows(make_page_model(*current_page.borrow()));
            }
        });
    }

    // Run command
    ui.on_run(move |command| {
        let command_str = command.to_string();
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(&command_str)
            .spawn();
    });

    ui.run()?;
    Ok(())
}