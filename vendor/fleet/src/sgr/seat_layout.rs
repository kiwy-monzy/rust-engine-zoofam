//! Seat layout — numbering + SVG rendering of a railway car.
//!
//! Ported from the Node `SGR-seat-layout-2025/src/seatLayout.js` canvas
//! renderer, but emits **SVG** (pure Rust, no native canvas/image deps) so the
//! result can be served directly and rendered in the browser.

use crate::sgr::models::{seat_type, RailwayCar, Seat};
use std::fmt::Write;

/// Options controlling the rendered seat map.
#[derive(Debug, Clone)]
pub struct LayoutOptions {
    pub seat_w: i32,
    pub seat_h: i32,
    pub padding: i32,
    pub title_h: i32,
    pub legend_h: i32,
    pub show_legend: bool,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self { seat_w: 46, seat_h: 40, padding: 6, title_h: 40, legend_h: 96, show_legend: true }
    }
}

/// Colors per seat state.
mod color {
    pub const AVAILABLE: &str = "#34d186";
    pub const OCCUPIED: &str = "#9aa4b2";
    pub const DISABLED: &str = "#3b82f6";
    pub const WC: &str = "#06b6d4";
    pub const LUGGAGE: &str = "#f59e0b";
    pub const STROKE: &str = "#1f2937";
    pub const TEXT_ON: &str = "#0b1220";
    pub const TEXT_MUTED: &str = "#4b5563";
}

/// Sequentially number the bookable seats (type 39), ordered by row then column.
/// Mutates `new_seat_no` in place, mirroring the JS `nameSeats`.
pub fn name_seats(seats: &mut [Seat]) {
    let mut order: Vec<usize> = (0..seats.len()).collect();
    order.sort_by(|&a, &b| {
        seats[a].row.cmp(&seats[b].row).then(seats[a].column.cmp(&seats[b].column))
    });
    let mut n = 1;
    for i in order {
        if seats[i].seat_type_id == seat_type::SEAT {
            seats[i].new_seat_no = n.to_string();
            n += 1;
        }
    }
}

fn fixture_color(seat_type_id: i32) -> Option<&'static str> {
    match seat_type_id {
        seat_type::DISABLED => Some(color::DISABLED),
        seat_type::WC => Some(color::WC),
        seat_type::LUGGAGE => Some(color::LUGGAGE),
        _ => None,
    }
}

fn fixture_glyph(seat_type_id: i32) -> &'static str {
    match seat_type_id {
        seat_type::DISABLED => "♿",
        seat_type::WC => "WC",
        seat_type::LUGGAGE => "🧳",
        _ => "",
    }
}

/// Render a railway car's seat map as a standalone SVG document.
pub fn render_svg(car: &RailwayCar, opts: &LayoutOptions) -> String {
    let (sw, sh, pad) = (opts.seat_w, opts.seat_h, opts.padding);
    let rows = car.row_count.max(1);
    let cols = car.column_count.max(1);

    // Grid is drawn "sideways" like the original: rows run along X, columns along Y.
    let grid_w = rows * (sw + pad) + pad;
    let grid_h = cols * (sh + pad) + pad;
    let width = grid_w.max(320);
    let legend_h = if opts.show_legend { opts.legend_h } else { 0 };
    let height = opts.title_h + grid_h + legend_h + pad;

    let mut s = String::new();
    let _ = write!(
        s,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}" height="{height}" font-family="system-ui, Arial, sans-serif">"#,
    );
    let _ = write!(s, r##"<rect width="{width}" height="{height}" fill="#ffffff"/>"##);

    // Title: "Coach {orderNo} — {type}"
    let title = if car.railway_car_type.is_empty() {
        format!("Coach {}", car.order_no)
    } else {
        format!("Coach {} — {}", car.order_no, esc(&car.railway_car_type))
    };
    let _ = write!(
        s,
        r#"<text x="{cx}" y="{ty}" text-anchor="middle" font-size="16" font-weight="700" fill="{tc}">{title}</text>"#,
        cx = width / 2, ty = opts.title_h / 2 + 5, tc = color::TEXT_ON,
    );

    // Seats
    for seat in &car.seats {
        if seat.seat_type_id == seat_type::CANCELLED { continue; }
        let x = pad + (seat.row - 1) * (sw + pad);
        let y = opts.title_h + pad + (seat.column - 1) * (sh + pad);

        if seat.seat_type_id == seat_type::SEAT {
            let fill = if seat.is_available { color::AVAILABLE } else { color::OCCUPIED };
            let _ = write!(
                s,
                r#"<g><rect x="{x}" y="{y}" width="{sw}" height="{sh}" rx="6" fill="{fill}" stroke="{st}" stroke-width="1"/>"#,
                st = color::STROKE,
            );
            // column letter (top) + seat number (bottom)
            if !seat.column_description.is_empty() {
                let _ = write!(
                    s,
                    r#"<text x="{tx}" y="{ty}" text-anchor="middle" font-size="10" fill="{tc}">{l}</text>"#,
                    tx = x + sw / 2, ty = y + 13, tc = color::TEXT_MUTED, l = esc(&seat.column_description),
                );
            }
            let label = if seat.new_seat_no.is_empty() { &seat.row_description } else { &seat.new_seat_no };
            let _ = write!(
                s,
                r#"<text x="{tx}" y="{ty}" text-anchor="middle" font-size="13" font-weight="700" fill="{tc}">{n}</text></g>"#,
                tx = x + sw / 2, ty = y + sh - 9, tc = color::TEXT_ON, n = esc(label),
            );
        } else if let Some(fill) = fixture_color(seat.seat_type_id) {
            let _ = write!(
                s,
                r#"<g><rect x="{x}" y="{y}" width="{sw}" height="{sh}" rx="6" fill="{fill}" fill-opacity="0.25" stroke="{fill}" stroke-width="1"/>"#,
            );
            let _ = write!(
                s,
                r#"<text x="{tx}" y="{ty}" text-anchor="middle" font-size="12" fill="{fill}">{g}</text></g>"#,
                tx = x + sw / 2, ty = y + sh / 2 + 4, g = fixture_glyph(seat.seat_type_id),
            );
        }
    }

    if opts.show_legend {
        let ly = opts.title_h + grid_h + pad;
        let _ = write!(
            s, r##"<line x1="0" y1="{ly}" x2="{width}" y2="{ly}" stroke="#e5e7eb" stroke-width="1"/>"##,
        );
        legend(&mut s, pad, ly + 14, &[
            (color::AVAILABLE, "Available"),
            (color::OCCUPIED, "Occupied"),
            (color::DISABLED, "Disabled"),
            (color::WC, "WC"),
            (color::LUGGAGE, "Luggage"),
        ]);
    }

    s.push_str("</svg>");
    s
}

fn legend(s: &mut String, x0: i32, y: i32, items: &[(&str, &str)]) {
    let mut x = x0 + 4;
    for (fill, label) in items {
        let _ = write!(
            s,
            r#"<rect x="{x}" y="{ry}" width="16" height="16" rx="3" fill="{fill}" stroke="{st}" stroke-width="1"/>"#,
            ry = y - 12, st = color::STROKE,
        );
        let _ = write!(
            s,
            r#"<text x="{tx}" y="{ty}" font-size="12" fill="{tc}">{label}</text>"#,
            tx = x + 22, ty = y, tc = color::TEXT_ON,
        );
        x += 26 + (label.len() as i32 * 7) + 16;
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sgr::models::Seat;

    fn demo_car() -> RailwayCar {
        let mut seats = Vec::new();
        for row in 1..=10 {
            for col in 1..=5 {
                let t = if col == 3 { seat_type::CANCELLED } else { seat_type::SEAT };
                seats.push(Seat {
                    column: col, column_description: ["A","B","","C","D"][(col-1) as usize].into(),
                    is_available: (row + col) % 3 != 0,
                    seat_type_id: t, row, row_description: row.to_string(),
                    id: (row * 10 + col) as i64, rotation: 0, new_seat_no: String::new(),
                });
            }
        }
        RailwayCar {
            id: 417, capacity: 40, empty_seats: 12, row_count: 10, column_count: 5,
            name: "ROYAL CLASS".into(), railway_car_type: "Royal Class".into(),
            order_no: 1, has_seat: true, seats,
        }
    }

    #[test]
    fn numbers_only_bookable_seats() {
        let mut car = demo_car();
        name_seats(&mut car.seats);
        let numbered = car.seats.iter().filter(|s| !s.new_seat_no.is_empty()).count();
        let bookable = car.seats.iter().filter(|s| s.is_bookable()).count();
        assert_eq!(numbered, bookable);
        assert!(car.seats.iter().any(|s| s.new_seat_no == "1"));
    }

    #[test]
    fn renders_valid_svg() {
        let mut car = demo_car();
        name_seats(&mut car.seats);
        let svg = render_svg(&car, &LayoutOptions::default());
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("Coach 1"));
    }
}
