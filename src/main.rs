extern crate gdk;
extern crate gdk_sys;
extern crate gdk_pixbuf;
extern crate gio;
extern crate glib;
extern crate gtk;
extern crate cairo;

use std::cell::RefCell;

use glib::translate::*;
use gtk::prelude::*;

thread_local!(
    static GLOBAL: RefCell<Option<cairo::Surface>> = RefCell::new(None);
);

fn clear_surface() {
    GLOBAL.with(|global| {
        if let Some(ref surface) = *global.borrow() {
            let cr = cairo::Context::new(surface);

            cr.set_source_rgb(1., 1., 1.);
            cr.paint();
        }
    });
}

/* Create a new surface of the appropriate size to store our scribbles */
fn configure_event_cb(widget: &gtk::DrawingArea, event: &gdk::EventConfigure) -> bool {
    GLOBAL.with(|global| {
        *global.borrow_mut() = Some(unsafe {
            from_glib_full(gdk_sys::gdk_window_create_similar_surface(
                    widget.get_window().unwrap().to_glib_none().0,
                    cairo::Content::Color,
                    widget.get_allocated_width(),
                    widget.get_allocated_height()))
        });

        /* Initialize the surface to white */
        clear_surface();
    });

    /* We've handled the configure event, no need for further processing. */
    true
}

/* Redraw the screen from the surface. Note that the ::draw
 * signal receives a ready-to-be-used cairo_t that is already
 * clipped to only draw the exposed areas of the widget
 */
fn draw_cb(widget: &gtk::DrawingArea, cr: &cairo::Context) -> Inhibit {
    GLOBAL.with(|global| {
        if let Some(ref surface) = *global.borrow() {
            cr.set_source_surface(surface, 0., 0.);
            cr.paint();
        }
    });

    Inhibit(false)
}

/* Draw a rectangle on the surface at the given position */
fn draw_brush(widget: &gtk::DrawingArea, x: f64, y: f64) {
    /* Paint to the surface, where we store our state */
    GLOBAL.with(|global| {
        if let Some(ref surface) = *global.borrow() {
            let cr = cairo::Context::new(surface);

            cr.rectangle(x - 3., y - 3., 6., 6.);
            cr.fill();

            /* Now invalidate the affected region of the drawing area. */
            widget.queue_draw_area (x as i32 - 3, y as i32 - 3, 6, 6);
        }
    });
}

/* Handle button press events by either drawing a rectangle
 * or clearing the surface, depending on which button was pressed.
 * The ::button-press signal handler receives a GdkEventButton
 * struct which contains this information.
 */
fn button_press_event_cb (widget: &gtk::DrawingArea, event: &gdk::EventButton) -> Inhibit {
    GLOBAL.with(|global| {
        /* paranoia check, in case we haven't gotten a configure event */
        if let Some(ref surface) = *global.borrow() {
            if event.get_button() == gdk_sys::GDK_BUTTON_PRIMARY as u32 {
                let (x, y) = event.get_position();
                draw_brush(widget, x, y);
            } else if event.get_button() == gdk_sys::GDK_BUTTON_SECONDARY as u32 {
                clear_surface();
                widget.queue_draw();
            }
        }
    });

    /* We've handled the event, stop processing */
    Inhibit(true)
}

/* Handle motion events by continuing to draw if button 1 is
 * still held down. The ::motion-notify signal handler receives
 * a GdkEventMotion struct which contains this information.
 */
fn motion_notify_event_cb (widget: &gtk::DrawingArea, event: &gdk::EventMotion) -> Inhibit {
    if (event.get_state() & gdk::BUTTON1_MASK).bits() != 0 {
        let (x, y) = event.get_position();
        draw_brush(widget, x, y);
    }

    /* We've handled it, stop processing */
    Inhibit(true)
}

fn close_window(_: &gtk::ApplicationWindow) {
    // drop global
    GLOBAL.with(|global| {
        *global.borrow_mut() = None;
    });
}

fn activate(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(&app);
    window.set_title("Drawing Area");
    window.connect_destroy(close_window);

    window.set_border_width(8);

    let frame = gtk::Frame::new(None);
    frame.set_shadow_type (gtk::ShadowType::In);
    window.add(&frame);

    let drawing_area = gtk::DrawingArea::new();
    /* set a minimum size */
    drawing_area.set_size_request(100, 100);

    frame.add(&drawing_area);

    /* Signals used to handle the backing surface */
    drawing_area.connect_draw(draw_cb);
    drawing_area.connect_configure_event(configure_event_cb);

    /* Event signals */
    drawing_area.connect_motion_notify_event(motion_notify_event_cb);
    drawing_area.connect_button_press_event(button_press_event_cb);

    /* Ask to receive events the drawing area doesn't normally
     * subscribe to. In particular, we need to ask for the
     * button press and motion notify events that want to handle.
     */
    drawing_area.add_events((gdk::BUTTON_PRESS_MASK | gdk::POINTER_MOTION_MASK).bits() as i32);

    window.show_all();
}

fn main() {
    let app = gtk::Application::new("my.app.testing", gio::ApplicationFlags::empty()).unwrap();

    app.connect_activate(activate);
    app.run(0, &[]);
}
