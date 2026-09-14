use runon_config::{Config, Selector};
use runon_core::event::{Event, Kind};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub(crate) struct Matcher {
    config: Arc<Config>,
    index: BTreeMap<Kind, Vec<usize>>,
}

fn matches(selector: &Selector, event: &Event) -> bool {
    selector.kind == event.kind
        && selector
            .fields
            .iter()
            .all(|(key, value)| event.fields.get(key) == Some(value))
}

impl Matcher {
    pub(crate) fn new(config: Arc<Config>) -> Self {
        let mut index: BTreeMap<Kind, Vec<usize>> = BTreeMap::new();
        for (id, action) in config.actions.iter().enumerate() {
            for kind in action
                .selectors
                .iter()
                .map(|selector| selector.kind)
                .collect::<BTreeSet<_>>()
            {
                index.entry(kind).or_default().push(id);
            }
        }
        Self { config, index }
    }

    pub(crate) fn batches(&self, event: &Event) -> BTreeMap<usize, Vec<usize>> {
        let mut batches: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        if let Some(actions) = self.index.get(&event.kind) {
            for &id in actions {
                let action = &self.config.actions[id];
                if action
                    .selectors
                    .iter()
                    .any(|selector| matches(selector, event))
                {
                    batches.entry(action.group).or_default().push(id);
                }
            }
        }
        batches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runon_core::event::Value;

    #[test]
    fn exact_fields_and_alternative_selectors() {
        let config = Arc::new(Config::parse("action a { on app.activated bundle-id=editor name=Редактор; on app.activated; exec p; }").unwrap());
        let selector = &config.actions[0].selectors[0];
        let event = Event::new(Kind::AppActivated).text("bundle-id", "editor");
        assert!(!matches(selector, &event));
        assert!(!matches(selector, &event.clone().text("name", "other")));
        let event = event.text("name", "Редактор");
        assert!(matches(selector, &event));
        let matcher = Matcher::new(config);
        // Multiple matching selectors still schedule an action only once.
        assert_eq!(matcher.batches(&event)[&0], vec![0]);
        assert_eq!(
            matcher.batches(&Event::new(Kind::AppActivated))[&0],
            vec![0]
        );
        assert!(matcher.batches(&Event::new(Kind::Wake)).is_empty());
    }

    #[test]
    fn all_actions_and_numeric_filters() {
        let config = Config::parse("action a group=g { on screen.connected id=12; exec p; }\naction b group=g { on screen.connected; exec p; }\ngroup g { debounce \"1m\"; }\n").unwrap();
        let matcher = Matcher::new(Arc::new(config));
        let mut event = Event::new(Kind::ScreenConnected);
        assert_eq!(matcher.batches(&event)[&0], vec![1]);
        event.fields.insert("id".into(), Value::Id(12));
        assert_eq!(matcher.batches(&event)[&0], vec![0, 1]);
        let round_trip = Config::parse(&format!(
            "action x {{ {}; exec p; }}",
            runon_config::format_selector(&event)
        ))
        .unwrap();
        assert_eq!(
            Matcher::new(Arc::new(round_trip)).batches(&event)[&0],
            vec![0]
        );
        event.fields.insert("id".into(), Value::Id(13));
        assert_eq!(matcher.batches(&event)[&0], vec![1]);
    }
}
