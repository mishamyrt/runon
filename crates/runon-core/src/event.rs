use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Kind {
    ScreenConnected,
    ScreenDisconnected,
    ScreenLocked,
    ScreenUnlocked,
    AudioConnected,
    AudioDisconnected,
    AppActivated,
    AppDeactivated,
    AppLaunched,
    AppTerminated,
    Wake,
    PowerChanged,
}

impl Kind {
    pub const ALL: [Self; 12] = [
        Self::ScreenConnected,
        Self::ScreenDisconnected,
        Self::ScreenLocked,
        Self::ScreenUnlocked,
        Self::AudioConnected,
        Self::AudioDisconnected,
        Self::AppActivated,
        Self::AppDeactivated,
        Self::AppLaunched,
        Self::AppTerminated,
        Self::Wake,
        Self::PowerChanged,
    ];

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == s)
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::ScreenConnected => "screen.connected",
            Self::ScreenDisconnected => "screen.disconnected",
            Self::ScreenLocked => "screen.locked",
            Self::ScreenUnlocked => "screen.unlocked",
            Self::AudioConnected => "audio.connected",
            Self::AudioDisconnected => "audio.disconnected",
            Self::AppActivated => "app.activated",
            Self::AppDeactivated => "app.deactivated",
            Self::AppLaunched => "app.launched",
            Self::AppTerminated => "app.terminated",
            Self::Wake => "system.wake",
            Self::PowerChanged => "power.changed",
        }
    }

    pub fn fields(self) -> &'static [&'static str] {
        match self {
            Self::ScreenConnected | Self::ScreenDisconnected => &["name", "id"],
            Self::AudioConnected | Self::AudioDisconnected => &["name", "uid"],
            Self::AppActivated | Self::AppDeactivated | Self::AppLaunched | Self::AppTerminated => {
                &["bundle-id", "name"]
            }
            Self::PowerChanged => &["source"],
            _ => &[],
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Text(String),
    Id(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    pub kind: Kind,
    pub fields: BTreeMap<String, Value>,
}

impl Event {
    pub fn new(kind: Kind) -> Self {
        Self {
            kind,
            fields: BTreeMap::new(),
        }
    }

    pub fn text(mut self, key: &str, value: impl Into<String>) -> Self {
        self.fields.insert(key.into(), Value::Text(value.into()));
        self
    }

    pub fn selector(&self) -> String {
        let mut node = kdl::KdlNode::new("on");
        node.push(self.kind.name());
        for (key, value) in &self.fields {
            let value = match value {
                Value::Text(s) => kdl::KdlValue::String(s.clone()),
                Value::Id(id) => kdl::KdlValue::Integer(i128::from(*id)),
            };
            node.push(kdl::KdlEntry::new_prop(key.as_str(), value));
        }
        node.to_string().trim_end().to_owned()
    }
}

/// Compare identities, retaining the old metadata for removed devices.
pub fn changes<K: Ord>(
    old: &BTreeMap<K, Event>,
    new: &BTreeMap<K, Event>,
    removed: Kind,
) -> Vec<Event> {
    let mut events = Vec::new();
    for (key, event) in old {
        if !new.contains_key(key) {
            let mut event = event.clone();
            event.kind = removed;
            events.push(event);
        }
    }
    events.extend(
        new.iter()
            .filter(|(key, _)| !old.contains_key(key))
            .map(|(_, e)| e.clone()),
    );
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_and_removed_metadata() {
        let a = Event::new(Kind::AudioConnected)
            .text("name", "Same")
            .text("uid", "a");
        let b = Event::new(Kind::AudioConnected)
            .text("name", "Same")
            .text("uid", "b");
        let old = BTreeMap::from([(1, a.clone()), (2, b.clone())]);
        let new = BTreeMap::from([(2, b)]);
        let events = changes(&old, &new, Kind::AudioDisconnected);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].fields, a.fields);
        assert_eq!(events[0].kind, Kind::AudioDisconnected);
        assert!(changes(&new, &new, Kind::AudioDisconnected).is_empty());
    }
}
