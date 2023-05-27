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
use std::{ cell::RefCell, rc::Rc, thread };

use eng_clock::{
    OffsetEvent, TickEvent, UImessage, UIsender, utc_now,
    config::ECConfig,
    stats::ExpoAvg,
    sync::OffsetEstimator,
    ticker::Ticker
};


/// Collection of GTK widgets that may need dynamic updates
#[derive(Clone)]
struct Widgets {
    hms_label: gtk::Label,
    phase_label: gtk::Label,
    latency_label: gtk::Label,
    avg_offs_label: gtk::Label,

    avg_latency: Rc<RefCell<ExpoAvg>>
}

impl Widgets {
    pub fn new(root: &gtk::ApplicationWindow) -> Widgets {
        let rtbox = gtk::Box::new(gtk::Orientation::Vertical, 3);
        root.add(&rtbox);

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        rtbox.pack_start(&hbox, false, false, 0);

        let hms_label = gtk::Label::new(None);
        hms_label.set_halign(gtk::Align::Center);
        hbox.pack_start(&hms_label, false, false, 2);

        let phase_label = gtk::Label::new(None);
        phase_label.set_halign(gtk::Align::End);
        hbox.pack_start(&phase_label, false, false, 6);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
        rtbox.pack_start(&vbox, false, false, 0);

        let avg_offs_label = gtk::Label::new(None);
        vbox.pack_start(&avg_offs_label, false, false, 0);

        let latency_label = gtk::Label::new(None);
        vbox.pack_start(&latency_label, false, false, 0);

        Widgets {
            hms_label,
            phase_label,
            avg_offs_label,
            latency_label,
            avg_latency: Rc::new(RefCell::new(ExpoAvg::new(0.1)))
        }
    }

    /// Prepare UI-update communication channel and associated event handlers
    pub fn init_channel(&self) -> UIsender {
        let (sender, receiver) =
            glib::MainContext::channel(glib::PRIORITY_HIGH);
        let w = self.clone();

        // Wire-up message handlers:
        receiver.attach(None, move |msg| {
            match msg {
                UImessage::Tick(event) =>   w.receive_tick(event),
                UImessage::Offset(event) => w.receive_offset(event)
            };
            glib::Continue(true)
        });

        sender
    }

    /// Update GUI elements after receiving clock-tick from Ticker
    pub fn receive_tick(&self, event: TickEvent) {
        const PHASE_CHARS: [char; 4] = [ '=', '.', ':', '\'' ];

        let hms_txt = format!(r#"<span size="x-large">{}</span>"#,
                              event.t_nominal.format("%H:%M:%S"));
        self.hms_label.set_markup(&hms_txt);

        let phase = (event.tick_id % 4) as usize;
        let phase_txt = format!(r#"<span size="small">{}</span>"#,
                                PHASE_CHARS[phase]);
        self.phase_label.set_markup(&phase_txt);

        let latency = utc_now() - event.t_transmit;
        let avg_latency = self.avg_latency.borrow_mut()
                                          .add_duration(latency);
        // FIXME - screen-update latency is likely to be sub-millisecond, but might be worth including in ticker offset
        if phase == 2 {
            let latency_txt = format!("UI latency: {:.2}ms",
                    avg_latency.num_microseconds()
                               .expect("UI latency should be finite") as f64 * 1e-3);
            self.latency_label.set_text(&latency_txt);
        }
    }

    pub fn receive_offset(&self, event: OffsetEvent) {
        let offs_txt = format!("Offset: {:.1}ms ± {:.1}ms",
                               event.avg_offset.num_microseconds()
                                    .expect("Offset should be finit") as f64 * 1e-3,
                               event.stddev_offset * 1e3);
        self.avg_offs_label.set_text(&offs_txt);
    }
}


fn read_config() -> ECConfig {
    match ECConfig::from_user_config() {
        Ok(cfg) => {
            println!("Ingested user-config: {:?}", cfg);
            cfg
        },
        Err(e) => {
            println!("Failed to read configuration file - {:?}", e);
            ECConfig::default()
        }
    }
}


/// Prepare GTK window and subcomponents, with clock ticking thread
fn on_activate(app: &gtk::Application) {
    let cfg = read_config();
    let win = gtk::ApplicationWindow::new(app);
    win.set_title("UTC Engineering Clock");
    win.set_border_width(8);
    win.set_position(gtk::WindowPosition::Center);
    win.set_default_size(144, 48);
    win.set_resizable(false);

    let widgets = Widgets::new(&win);
    let sender = widgets.init_channel();

    let mut ticker = Ticker::new(sender.clone());
    let mut offest = OffsetEstimator::new(ticker.get_sync(),
                                          sender.clone(), &cfg.sync);
    thread::spawn(move || { ticker.run() });
    thread::spawn(move || { offest.run() });

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
