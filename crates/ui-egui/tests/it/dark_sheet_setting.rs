//! The dark sheet setting (`view.darkSheet`): off by default, a toggle, a preference, screen only.

use egui::vec2;
use gridcraft_engine::Session;
use gridcraft_ui_egui::theme::{self, Tokens};
use gridcraft_ui_egui::{SheetApp, UiState};
use serde_json::json;

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

#[test]
fn off_by_default_and_the_sheet_stays_light() {
    assert!(!UiState::default().dark_sheet);
    let h = harness();
    assert!(!h.state().ui.dark_sheet);
    assert!(!theme::dark_sheet(&h.ctx));
}

#[test]
fn the_command_sets_toggles_and_accepts_the_wordcraft_spelling() {
    let mut h = harness();
    assert_eq!(h.state_mut().run("view.darkSheet", json!({})), Ok(json!({"on": true})));
    assert_eq!(h.state_mut().run("view.darkSheet", json!({})), Ok(json!({"on": false})));
    assert_eq!(h.state_mut().run("view.darkSheet", json!({"on": true})), Ok(json!({"on": true})));
    assert_eq!(h.state_mut().run("view.darkSheet", json!({"value": false})), Ok(json!({"on": false})));
}

#[test]
fn the_flag_reaches_the_tokens_and_is_independent_of_the_display_theme() {
    let mut h = harness();
    h.state_mut().run("view.darkSheet", json!({"on": true})).unwrap();
    h.run_steps(2);
    assert!(theme::dark_sheet(&h.ctx));
    assert_ne!(Tokens::get(&h.ctx).grid_bg, Tokens::get(&h.ctx).with_dark_sheet(false).grid_bg);
    h.state_mut().run("view.theme", json!({"mode": "light"})).unwrap();
    h.run_steps(2);
    assert!(theme::dark_sheet(&h.ctx), "the Display theme does not touch it");
}

#[test]
fn the_preference_survives_a_restart_and_older_settings_read_as_off() {
    let state = UiState { dark_sheet: true, ..UiState::default() };
    let saved = serde_json::to_string(&state).unwrap();
    assert!(serde_json::from_str::<UiState>(&saved).unwrap().dark_sheet);
    assert!(!serde_json::from_str::<UiState>("{}").unwrap().dark_sheet);
}

#[test]
fn it_is_not_a_document_change() {
    let mut h = harness();
    let dirty = h.state().session.active().map(|d| d.is_dirty());
    h.state_mut().run("view.darkSheet", json!({"on": true})).unwrap();
    assert_eq!(h.state().session.active().map(|d| d.is_dirty()), dirty);
}
