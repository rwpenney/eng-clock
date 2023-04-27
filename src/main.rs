/*
 *  Engineering real-time clock display
 *  RW Penney, April 2023
 */

use gtk::glib;
use gtk::prelude::*;
use std::thread;

use eng_clock::{ Ticker, Timestamp, UImessage };


fn receive_tick(t: Timestamp, hms_label: &gtk::Label) {
    let hms_txt = format!("{}", t.format("%H:%M:%S"));
    hms_label.set_text(&hms_txt);
}


fn on_activate(app: &gtk::Application) {
    let win = gtk::ApplicationWindow::new(app);
    win.set_title("UTC Engineering Clock");
    win.set_border_width(8);
    win.set_position(gtk::WindowPosition::Center);
    win.set_default_size(200, 100);

    let hms_label = gtk::Label::new(None);
    win.add(&hms_label);

    let (sender, receiver) =
        glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    {   let hms = hms_label.clone();

        receiver.attach(None, move |msg| {
            match msg {
                UImessage::Tick(t) => receive_tick(t, &hms)
            };
            glib::Continue(true)
        });
    }

    let ticker = Ticker::new(&sender);
    thread::spawn(move || { ticker.run() });

    win.show_all();
}


fn main() {
    let app =
        gtk::Application::builder()
            .application_id("uk.rwpenney.engclodk")
            .build();

    app.connect_activate(on_activate);
    app.run();
}

// (C)Copyright 2023, RW Penney
