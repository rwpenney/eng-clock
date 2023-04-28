/*
 *  Engineering real-time clock display
 *  RW Penney, April 2023
 */

use gtk::glib;
use gtk::prelude::*;
use std::thread;

use eng_clock::{ Ticker, TickEvent, UImessage, utc_now };


#[derive(Clone)]
struct Widgets {
    hms_label: gtk::Label,
    phase_label: gtk::Label
}

impl Widgets {
    const PHASE_CHARS: [char; 4] = [ '=', ':', '.', ':' ];

    pub fn new(root: &gtk::ApplicationWindow) -> Widgets {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        root.add(&hbox);

        let hms_label = gtk::Label::new(None);
        hbox.pack_start(&hms_label, false, false, 3);

        let phase_label = gtk::Label::new(None);
        hbox.pack_start(&phase_label, false, false, 3);

        Widgets {
            hms_label,
            phase_label
        }
    }

    pub fn receive_tick(&self, event: TickEvent) {
        let hms_txt =
            format!(r#"<span foreground="blue" size="x-large">{}</span>"#,
                    event.t_nominal.format("%H:%M:%S"));
        self.hms_label.set_markup(&hms_txt);

        let phase = (event.tick_id % 4) as usize;
        let phase_txt = format!(r#"<span size="small">{}</span>"#,
                                Widgets::PHASE_CHARS[phase]);
        self.phase_label.set_markup(&phase_txt);

        let latency_us = (utc_now() - event.t_transmit).num_microseconds().unwrap_or(0);
        println!("Latency= {latency_us}us")
    }
}


fn on_activate(app: &gtk::Application) {
    let win = gtk::ApplicationWindow::new(app);
    win.set_title("UTC Engineering Clock");
    win.set_border_width(8);
    win.set_position(gtk::WindowPosition::Center);
    win.set_default_size(200, 100);

    let widgets = Widgets::new(&win);

    let (sender, receiver) =
        glib::MainContext::channel(glib::PRIORITY_HIGH);

    {   let w = widgets.clone();

        receiver.attach(None, move |msg| {
            match msg {
                UImessage::Tick(event) => w.receive_tick(event)
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
