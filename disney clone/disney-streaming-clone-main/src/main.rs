// Copyright 2019 The Druid Authors.

// On Windows platform, don't show a console when opening the app.
#![windows_subsystem = "windows"]

mod content_set;
mod root_widget;
mod thumbnail;

use widget_cruncher::{AppLauncher, WindowDesc};

fn main() {
    let main_window = WindowDesc::new(root_widget::RootWidget::new()).title("Title list");
    AppLauncher::with_window(main_window)
        .log_to_console()
        .launch()
        .expect("launch failed");
}
