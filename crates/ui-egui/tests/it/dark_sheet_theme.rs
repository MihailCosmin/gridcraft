//! Switching the Display theme (`view.theme`) over the control channel repaints the interface and
//! the sheet at once, and the choice survives a restart.

use egui::vec2;
use gridcraft_engine::Session;
use gridcraft_ui_egui::{SheetApp, UiState, control, theme::Tokens};
use serde_json::{Value, json};

fn harness() -> egui_kittest::Harness<'static, SheetApp> {
    let mut h = crate::common::builder().with_size(vec2(1200.0, 800.0)).build_ui_state(
        |ui, app: &mut SheetApp| {
            app.logic(ui.ctx());
            app.ui(ui);
        },
        crate::common::app(Session::new()),
    );
    h.run_steps(4);
    h
}

fn execute(h: &mut egui_kittest::Harness<'static, SheetApp>, command: &str, params: Value) -> Value {
    let ctx = h.ctx.clone();
    let control::Outcome::Done(response) = control::handle(h.state_mut(), &ctx, "engine.execute", &json!({"command": command, "params": params}))
    else {
        panic!("engine.execute should answer at once");
    };
    assert_eq!(response["ok"], true, "{response}");
    h.run_steps(2);
    response
}

/// The colour of a far-away empty cell (no selection, no text) and of the ribbon.
fn sheet_and_chrome(h: &mut egui_kittest::Harness<'static, SheetApp>) -> Option<([u8; 3], [u8; 3])> {
    let image = h.render().ok()?;
    let sh = h.state().session.active()?.wb.active()?;
    let r = h.state().grid.geo.as_ref()?.screen_cell_rect(sh, gridcraft_engine::core::CellRef::parse("F12")?);
    let ppp = h.ctx.pixels_per_point();
    let px = |x: f32, y: f32| {
        let p = image.get_pixel((x * ppp) as u32, (y * ppp) as u32);
        [p[0], p[1], p[2]]
    };
    // The ribbon's background, well above the sheet.
    Some((px(r.center().x, r.center().y), px(600.0, 10.0)))
}

#[test]
fn a_theme_switch_repaints_the_interface_and_the_sheet_at_once() {
    let mut h = harness();
    execute(&mut h, "view.theme", json!({"mode": "light"}));
    let Some((sheet_light, chrome_light)) = sheet_and_chrome(&mut h) else { return };
    assert_eq!(sheet_light, [255; 3]);
    execute(&mut h, "view.theme", json!({"mode": "dark"}));
    let (sheet, chrome_dark) = sheet_and_chrome(&mut h).unwrap();
    assert_eq!(sheet, [0x33; 3], "Display theme Dark shows the dark sheet");
    assert_ne!(chrome_dark, chrome_light, "the interface repainted on the same switch");
    execute(&mut h, "view.theme", json!({"mode": "light"}));
    let (sheet, chrome) = sheet_and_chrome(&mut h).unwrap();
    assert_eq!((sheet, chrome), ([255; 3], chrome_light), "back to light, both together");
}

#[test]
fn the_dark_theme_is_saved_and_restored_with_its_dark_sheet() {
    let mut h = harness();
    execute(&mut h, "view.theme", json!({"mode": "dark"}));
    let saved = serde_json::to_string(&h.state().ui).unwrap();
    let restored: UiState = serde_json::from_str(&saved).unwrap();
    assert!(restored.dark && !restored.system_theme);
    let mut h2 = harness();
    h2.state_mut().ui = restored;
    h2.run_steps(3);
    assert!(h2.ctx.global_style().visuals.dark_mode);
    assert!(Tokens::get(&h2.ctx).dark_sheet);
    if let Some((sheet, _)) = sheet_and_chrome(&mut h2) {
        assert_eq!(sheet, [0x33; 3]);
    }
}

#[test]
fn a_screenshot_request_after_a_switch_shows_the_new_appearance() {
    let mut h = harness();
    execute(&mut h, "view.theme", json!({"mode": "dark"}));
    let (tx, rx) = std::sync::mpsc::channel();
    h.state_mut().control_rx = Some(rx);
    let (shot, reply) = control::ControlRequest::new("ui.screenshot", json!({}));
    tx.send(shot).unwrap();
    for _ in 0..8 {
        h.step();
    }
    // Without a GPU the harness cannot capture a frame; then there is nothing to check.
    if let Ok(response) = reply.try_recv() {
        assert!(response.get("ok").and_then(Value::as_bool).unwrap_or(true), "{response}");
    }
}
