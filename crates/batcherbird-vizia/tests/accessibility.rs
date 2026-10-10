//! Exercise generated AccessKit nodes, rather than the app's label strings.
use accesskit::{ActionRequest, Node, TreeUpdate};
use batcherbird_vizia::{app_data::AppData, app_event::AppEvent, views::KeyboardView};
use std::sync::Mutex;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use vizia::backend::{BackendContext, IntoNode};
use vizia::events::EventManager;
use vizia::prelude::*;

#[derive(Lens)]
struct ToggleState {
    checked: bool,
}
impl Model for ToggleState {}

fn node_named<'a>(tree: &'a TreeUpdate, label: &str) -> &'a Node {
    &tree
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .unwrap_or_else(|| panic!("missing accessible label: {label}"))
        .1
}

#[test]
fn labels_and_click_actions_are_generated_for_buttons_and_checked_rows() {
    let mut cx = Context::default();
    Button::new(&mut cx, |cx| Label::new(cx, "Open…"))
        .name("Open session")
        .on_press(|_| {});
    Button::new(&mut cx, |cx| Label::new(cx, "C4 · velocity 64"))
        .name("C4 · velocity 64")
        .checked(true)
        .on_press(|_| {});
    Button::new(&mut cx, |cx| Label::new(cx, "Export"))
        .name("Export samples")
        .disabled(true)
        .on_press(|_| {});
    Label::new(&mut cx, "Status").name("Session status");
    // A role alone must never invent an action on a custom/non-actionable view.
    Label::new(&mut cx, "Decorative")
        .role(Role::Button)
        .name("Decorative role");
    ToggleState { checked: false }.build(&mut cx);
    ToggleButton::new(&mut cx, ToggleState::checked, |cx| {
        Label::new(cx, "Monitor")
    })
    .name("Toggle monitoring");
    Checkbox::new(&mut cx, ToggleState::checked).name("Enable trimming");
    RadioButton::new(&mut cx, ToggleState::checked).name("Mono routing");

    let tree = BackendContext::new(cx).init_accessibility_tree();
    let open = node_named(&tree, "Open session");
    assert!(open.supports_action(Action::Click));
    assert!(open.supports_action(Action::Focus));
    assert_eq!(open.value(), None);
    let row = node_named(&tree, "C4 · velocity 64");
    assert_eq!(row.toggled(), Some(accesskit::Toggled::True));
    assert!(row.supports_action(Action::Click));
    let disabled = node_named(&tree, "Export samples");
    assert!(disabled.is_disabled());
    assert!(!disabled.supports_action(Action::Click));
    assert!(!node_named(&tree, "Session status").supports_action(Action::Click));
    assert!(!node_named(&tree, "Decorative role").supports_action(Action::Click));
    for name in ["Toggle monitoring", "Enable trimming", "Mono routing"] {
        assert!(node_named(&tree, name).supports_action(Action::Click));
    }
}

#[test]
fn click_dispatch_activates_enabled_button_and_rejects_stale_disabled_request() {
    let mut cx = Context::default();
    let enabled_count = Arc::new(AtomicUsize::new(0));
    let disabled_count = Arc::new(AtomicUsize::new(0));
    let counter = enabled_count.clone();
    let enabled = Button::new(&mut cx, |cx| Label::new(cx, "Open"))
        .on_press(move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
        })
        .entity();
    let counter = disabled_count.clone();
    let disabled = Button::new(&mut cx, |cx| Label::new(cx, "Export"))
        .on_press(move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
        })
        .disabled(true)
        .entity();
    let mut backend = BackendContext::new(cx);
    let mut events = EventManager::new();
    for target in [enabled, disabled] {
        backend.send_event(
            Event::new(WindowEvent::ActionRequest(ActionRequest {
                action: Action::Click,
                target: target.accesskit_id(),
                data: None,
            }))
            .direct(target),
        );
    }
    events.flush_events(&mut backend.0, |_| {});
    assert_eq!(enabled_count.load(Ordering::SeqCst), 1);
    assert_eq!(disabled_count.load(Ordering::SeqCst), 0);
}

struct AuditionEvents(Arc<Mutex<Vec<Option<u8>>>>);
impl Model for AuditionEvents {
    fn event(&mut self, _: &mut EventContext, event: &mut Event) {
        event.map(|event: &AppEvent, _| match event {
            AppEvent::AuditionNoteOn(note) => self.0.lock().unwrap().push(Some(*note)),
            AppEvent::AuditionNoteOff => self.0.lock().unwrap().push(None),
            _ => {}
        });
    }
}

#[test]
fn keyboard_audition_pairs_note_off_on_release_and_focus_loss() {
    let mut cx = Context::default();
    AppData::demo().build(&mut cx);
    let observed = Arc::new(Mutex::new(Vec::new()));
    AuditionEvents(observed.clone()).build(&mut cx);
    let keyboard = KeyboardView::new(&mut cx).entity();
    let mut backend = BackendContext::new(cx);
    let mut events = EventManager::new();
    let tree = backend.init_accessibility_tree();
    let node = node_named(
        &tree,
        "Synthesizer keyboard: arrow keys choose a note; hold Space to audition",
    );
    assert!(node.supports_action(Action::Focus));
    assert!(!node.supports_action(Action::Click));
    for event in [
        WindowEvent::FocusIn,
        WindowEvent::KeyDown(Code::ArrowRight, None),
        WindowEvent::KeyDown(Code::Space, None),
        WindowEvent::KeyDown(Code::Space, None), // Repeat must not retrigger.
        WindowEvent::KeyUp(Code::Space, None),
        WindowEvent::KeyDown(Code::Space, None),
        WindowEvent::FocusOut,
        WindowEvent::KeyUp(Code::Space, None),
    ] {
        backend.send_event(Event::new(event).direct(keyboard));
        events.flush_events(&mut backend.0, |_| {});
    }
    assert_eq!(
        *observed.lock().unwrap(),
        vec![Some(61), None, Some(61), None]
    );
}

#[test]
fn event_context_checked_change_publishes_incremental_toggle_update() {
    let mut cx = Context::default();
    let button = Button::new(&mut cx, |cx| Label::new(cx, "Enable"))
        .name("Enable option")
        .checked(false)
        .on_press(|cx| cx.set_checked(true))
        .entity();
    let mut backend = BackendContext::new(cx);
    backend.init_accessibility_tree();
    backend.process_tree_updates();
    backend.0.tree_updates.clear();
    backend.send_event(
        Event::new(WindowEvent::ActionRequest(ActionRequest {
            action: Action::Click,
            target: button.accesskit_id(),
            data: None,
        }))
        .direct(button),
    );
    EventManager::new().flush_events(&mut backend.0, |_| {});
    backend.process_tree_updates();
    let node = backend
        .0
        .tree_updates
        .iter()
        .filter_map(Option::as_ref)
        .flat_map(|update| update.nodes.iter())
        .find(|(id, _)| *id == button.accesskit_id())
        .map(|(_, node)| node)
        .expect("event-context checked changes must publish an accessibility update");
    assert_eq!(node.toggled(), Some(accesskit::Toggled::True));
}

#[test]
fn disabled_binding_publishes_updated_accessibility_actions() {
    #[derive(Lens)]
    struct BusyState {
        busy: bool,
    }
    impl Model for BusyState {
        fn event(&mut self, _: &mut EventContext, event: &mut Event) {
            event.map(|busy: &bool, _| self.busy = *busy);
        }
    }
    for inherited in [false, true] {
        let mut cx = Context::default();
        BusyState { busy: false }.build(&mut cx);
        let mut button = Entity::root();
        let parent = VStack::new(&mut cx, |cx| {
            let handle = Button::new(cx, |cx| Label::new(cx, "Export"))
                .name("Export")
                .on_press(|_| {});
            button = if inherited {
                handle.entity()
            } else {
                handle.disabled(BusyState::busy).entity()
            };
        });
        if inherited {
            parent.disabled(BusyState::busy);
        }
        let mut backend = BackendContext::new(cx);
        backend.process_style_updates();
        backend.init_accessibility_tree();
        backend.process_tree_updates();
        backend.0.tree_updates.clear();
        let mut events = EventManager::new();
        for disabled in [true, false] {
            backend.send_event(Event::new(disabled));
            events.flush_events(&mut backend.0, |_| {});
            backend.process_style_updates();
            backend.process_tree_updates();
            let node = backend
                .0
                .tree_updates
                .iter()
                .filter_map(Option::as_ref)
                .flat_map(|update| update.nodes.iter())
                .find(|(id, _)| *id == button.accesskit_id())
                .map(|(_, node)| node)
                .expect("disabled changes must publish an accessibility update");
            assert_eq!(node.is_disabled(), disabled, "inherited={inherited}");
            assert_eq!(
                node.supports_action(Action::Click),
                !disabled,
                "inherited={inherited}"
            );
            backend.0.tree_updates.clear();
        }
    }
}
