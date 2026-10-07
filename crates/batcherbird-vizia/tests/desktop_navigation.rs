//! Regression for the production sample list's shared activation/navigation path.
use accesskit::{ActionRequest, Role};
use batcherbird_vizia::{app_data::AppData, views::stage};
use vizia::backend::{BackendContext, IntoNode};
use vizia::events::EventManager;
use vizia::prelude::*;

#[test]
fn repeated_sample_activation_keeps_arrow_navigation_on_current_row() {
    let mut cx = Context::default();
    let data = AppData::demo();
    let chosen_label = data.sample_labels[5].clone();
    data.build(&mut cx);
    stage(&mut cx);
    let mut backend = BackendContext::new(cx);
    let tree = backend.init_accessibility_tree();
    let row_id = tree
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(chosen_label.as_str()))
        .unwrap()
        .0;
    let list_id = tree
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::List)
        .unwrap()
        .0;
    let row = Entity::root()
        .branch_iter(&backend.0.tree)
        .find(|entity| entity.accesskit_id() == row_id)
        .unwrap();
    let list = Entity::root()
        .branch_iter(&backend.0.tree)
        .find(|entity| entity.accesskit_id() == list_id)
        .unwrap();
    let mut events = EventManager::new();
    for _ in 0..2 {
        backend.send_event(
            Event::new(WindowEvent::ActionRequest(ActionRequest {
                action: Action::Click,
                target: row_id,
                data: None,
            }))
            .direct(row),
        );
        events.flush_events(&mut backend.0, |_| {});
        assert_eq!(backend.0.data::<AppData>().unwrap().selected_sample, 5);
    }
    backend.send_event(Event::new(ListEvent::FocusNext).direct(list));
    events.flush_events(&mut backend.0, |_| {});
    assert_eq!(backend.0.data::<AppData>().unwrap().selected_sample, 6);
    backend.send_event(Event::new(ListEvent::FocusPrev).direct(list));
    events.flush_events(&mut backend.0, |_| {});
    assert_eq!(backend.0.data::<AppData>().unwrap().selected_sample, 5);
}
