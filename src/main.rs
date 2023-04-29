/*  eng-clock - a dynamically synchronized realtime clock display
    Copyright (C) 2023, RW Penney

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>
 */

use gtk::glib;
use gtk::prelude::*;
use std::{ rc::Rc, thread };

use eng_clock::{ Ticker, TickEvent, UImessage, UIsender, utc_now };
use eng_clock::stats::ExpoAvg;


/// Collection of GTK widgets that may need dynamic updates
#[derive(Clone)]
struct Widgets {
    hms_label: gtk::Label,
    phase_label: gtk::Label,

    avg_latency: Rc<ExpoAvg>
}

impl Widgets {
    pub fn new(root: &gtk::ApplicationWindow) -> Widgets {
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        root.add(&hbox);

        let hms_label = gtk::Label::new(None);
        hbox.pack_start(&hms_label, false, false, 3);

        let phase_label = gtk::Label::new(None);
        hbox.pack_start(&phase_label, false, false, 3);

        Widgets {
            hms_label,
            phase_label,
            avg_latency: Rc::new(ExpoAvg::new(0.1))
        }
    }

    /// Prepare UI-update communication channel and associated event handlers
    pub fn init_channel(&self) -> UIsender {
        let (sender, receiver) =
            glib::MainContext::channel(glib::PRIORITY_HIGH);
        let mut w = self.clone();

        receiver.attach(None, move |msg| {
            match msg {
                UImessage::Tick(event) => w.receive_tick(event)
            };
            glib::Continue(true)
        });

        sender
    }

    /// Update GUI elements after receiving clock-tick from Ticker
    pub fn receive_tick(&mut self, event: TickEvent) {
        const PHASE_CHARS: [char; 4] = [ '=', '.', ':', '\'' ];

        let hms_txt = format!(r#"<span size="x-large">{}</span>"#,
                              event.t_nominal.format("%H:%M:%S"));
        self.hms_label.set_markup(&hms_txt);

        let phase = (event.tick_id % 4) as usize;
        let phase_txt = format!(r#"<span size="small">{}</span>"#,
                                PHASE_CHARS[phase]);
        self.phase_label.set_markup(&phase_txt);

        let latency = utc_now() - event.t_transmit;
        let avg_latency = Rc::get_mut(&mut self.avg_latency).map(|ea| ea.add_duration(latency)).expect("ExpoAvg should be accessible");
        // FIXME - screen-update latency is likely to be sub-millisecond, but might be worth including in ticker offset
        if phase == 0 { println!("Latency= {avg_latency}") };
    }
}


/// Prepare GTK window and subcomponents, with clock ticking thread
fn on_activate(app: &gtk::Application) {
    let win = gtk::ApplicationWindow::new(app);
    win.set_title("UTC Engineering Clock");
    win.set_border_width(8);
    win.set_position(gtk::WindowPosition::Center);
    win.set_default_size(144, 48);

    let widgets = Widgets::new(&win);
    let sender = widgets.init_channel();

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
