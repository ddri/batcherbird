//! Production UI checks through AccessKit and real event dispatch.
//! These do not substitute for native layout, speech, or dialog acceptance.
use accesskit::{ActionRequest, Node, TreeUpdate};
use batcherbird_vizia::{
    app_data::{AppData, AppState},
    app_event::AppEvent,
    session::SessionSettings,
    views::{stage, toolbar},
};
use vizia::backend::{BackendContext, IntoNode};
use vizia::events::EventManager;
use vizia::prelude::*;

fn named<'a>(tree: &'a TreeUpdate, label: &str) -> (accesskit::NodeId, &'a Node) {
    tree.nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .map(|(id, node)| (*id, node))
        .unwrap_or_else(|| panic!("missing production control: {label}"))
}

#[test]
fn production_stage_exposes_actions_and_blocks_session_operations_during_capture() {
    for (state, action, busy) in [
        (AppState::Idle, "Check input signal", false),
        (AppState::Armed, "Start recording", false),
        (AppState::Recording, "Stop recording", true),
        (AppState::Stopping, "Panic", true),
        (AppState::Review, "Play selected sample", false),
    ] {
        let mut cx = Context::default();
        // Dynamic bindings remove old views; provide the root window state that
        // the native backend normally registers before processing events.
        cx.windows.insert(Entity::root(), Default::default());
        let mut data = AppData::demo();
        data.app_state = state.clone();
        data.controls_busy = data.is_busy();
        data.build(&mut cx);
        toolbar(&mut cx);
        stage(&mut cx);
        let tree = BackendContext::new(cx).init_accessibility_tree();
        assert!(named(&tree, action).1.supports_action(Action::Click));
        for label in ["Open…", "Save session"] {
            let node = named(&tree, label).1;
            assert_eq!(node.is_disabled(), busy, "{state:?}: {label}");
            assert_eq!(
                node.supports_action(Action::Click),
                !busy,
                "{state:?}: {label}"
            );
        }
    }
}

#[test]
fn replacement_banner_click_dispatch_keeps_or_replaces_the_validated_candidate() {
    for replace in [false, true] {
        let mut cx = Context::default();
        // Dynamic bindings remove old views; provide the root window state that
        // the native backend normally registers before processing events.
        cx.windows.insert(Entity::root(), Default::default());
        let mut data = AppData::demo();
        data.instrument_name = "Edited current instrument".into();
        data.has_unsaved_changes = true;
        let original_audio = data.recorded_samples[0].audio_data.clone();
        let mut replacement_samples = data.recorded_samples[..1].to_vec();
        replacement_samples[0].audio_data = vec![0.1, -0.2];
        data.build(&mut cx);
        let mut backend = BackendContext::new(cx);
        let mut events = EventManager::new();
        backend.send_event(Event::new(AppEvent::SessionLoaded {
            path: "replacement.batcherbird".into(),
            settings: SessionSettings {
                instrument_name: "Replacement instrument".into(),
                ..SessionSettings::default()
            },
            samples: replacement_samples,
        }));
        events.flush_events(&mut backend.0, |_| {});
        assert_eq!(
            backend.0.data::<AppData>().unwrap().instrument_name,
            "Edited current instrument"
        );
        toolbar(&mut backend.0);
        stage(&mut backend.0);
        let tree = backend.init_accessibility_tree();
        for label in ["Keep current session", "Open replacement"] {
            assert!(named(&tree, label).1.supports_action(Action::Click));
        }
        assert!(named(&tree, "Open…").1.is_disabled());
        let label = if replace {
            "Open replacement"
        } else {
            "Keep current session"
        };
        let target = named(&tree, label).0;
        let entity = Entity::root()
            .branch_iter(&backend.0.tree)
            .find(|entity| entity.accesskit_id() == target)
            .unwrap();
        backend.send_event(
            Event::new(WindowEvent::ActionRequest(ActionRequest {
                action: Action::Click,
                target,
                data: None,
            }))
            .direct(entity),
        );
        events.flush_events(&mut backend.0, |_| {});
        let data = backend.0.data::<AppData>().unwrap();
        assert!(data.pending_session_name.is_none());
        assert!(!data.session_busy);
        if replace {
            assert_eq!(data.instrument_name, "Replacement instrument");
            assert_eq!(data.recorded_samples.len(), 1);
            assert_eq!(data.recorded_samples[0].audio_data, vec![0.1, -0.2]);
        } else {
            assert_eq!(data.instrument_name, "Edited current instrument");
            assert_eq!(data.recorded_samples[0].audio_data, original_audio);
            assert!(data.has_unsaved_changes);
        }
    }
}
