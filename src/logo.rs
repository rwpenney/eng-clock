/*
 *  Wrapper for SVG logo of eng-clock
 *  RW Penney, November 2023
 */

use gtk::gdk_pixbuf::Pixbuf;
use gtk::gio::{ Cancellable, MemoryInputStream };
use gtk::glib::Bytes;


static LOGO_SVG: &[u8] = include_bytes!("../logo.svg");


pub fn get_pixbuf() -> Result<Pixbuf, gtk::glib::error::Error> {
    let strm = MemoryInputStream::from_bytes(&Bytes::from_static(&LOGO_SVG));
    Pixbuf::from_stream(&strm, Cancellable::NONE)
}
