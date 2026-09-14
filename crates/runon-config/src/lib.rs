//! Configuration structures, KDL loading, parsing and validation.
//! File paths are supplied by the caller; no environment or path discovery.
#![forbid(unsafe_code)]

use kdl::{KdlDocument, KdlEntry, KdlNode};
use runon_core::event::{Event, Kind, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    path::Path,
    time::Duration,
};

/// Format an observed event as a selector accepted by the configuration parser.
pub fn format_selector(event: &Event) -> String {
    let mut node = KdlNode::new("on");
    node.push(event.kind.name());
    for (key, value) in &event.fields {
        let value = match value {
            Value::Text(s) => kdl::KdlValue::String(s.clone()),
            Value::Id(id) => kdl::KdlValue::Integer(i128::from(*id)),
        };
        node.push(KdlEntry::new_prop(key.as_str(), value));
    }
    node.to_string().trim_end().to_owned()
}

#[derive(Clone, Debug)]
pub struct Selector {
    pub kind: Kind,
    pub fields: BTreeMap<String, Value>,
}

#[derive(Clone, Debug)]
pub struct Action {
    pub name: String,
    pub selectors: Vec<Selector>,
    pub steps: Vec<Vec<String>>,
    pub timeout: Duration,
    pub group: usize,
}

#[derive(Clone, Debug)]
pub struct Group {
    pub debounce: Duration,
}

#[derive(Debug)]
pub struct Config {
    pub max_parallel: usize,
    pub actions: Vec<Action>,
    pub groups: Vec<Group>,
}

#[derive(Debug)]
pub struct ConfigError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.message)
    }
}
impl std::error::Error for ConfigError {}

struct Reader<'a>(&'a str);
impl Reader<'_> {
    fn error(&self, offset: usize, message: impl Into<String>) -> ConfigError {
        let prefix = &self.0[..offset.min(self.0.len())];
        ConfigError {
            line: prefix.bytes().filter(|b| *b == b'\n').count() + 1,
            column: prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1,
            message: message.into(),
        }
    }
    fn node_error(&self, node: &KdlNode, message: impl Into<String>) -> ConfigError {
        self.error(node.span().offset(), message)
    }
    fn entry_error(&self, entry: &KdlEntry, message: impl Into<String>) -> ConfigError {
        self.error(entry.span().offset(), message)
    }
    fn args<'a>(
        &self,
        node: &'a KdlNode,
        props: &[&str],
    ) -> Result<Vec<&'a KdlEntry>, ConfigError> {
        if node.ty().is_some() {
            return Err(self.node_error(node, "type annotations are not supported"));
        }
        let mut seen = BTreeSet::new();
        let mut args = Vec::new();
        for e in node.entries() {
            if e.ty().is_some() {
                return Err(self.entry_error(e, "type annotations are not supported"));
            }
            if let Some(name) = e.name() {
                let name = name.value();
                if !props.contains(&name) {
                    return Err(self.entry_error(e, format!("unknown property '{name}'")));
                }
                if !seen.insert(name) {
                    return Err(self.entry_error(e, format!("duplicate property '{name}'")));
                }
            } else {
                args.push(e);
            }
        }
        Ok(args)
    }
    fn one<'a>(&self, node: &'a KdlNode, props: &[&str]) -> Result<&'a KdlEntry, ConfigError> {
        let args = self.args(node, props)?;
        if args.len() != 1 {
            return Err(self.node_error(node, "expected exactly one argument"));
        }
        Ok(args[0])
    }
    fn text<'a>(&self, e: &'a KdlEntry) -> Result<&'a str, ConfigError> {
        e.value()
            .as_string()
            .ok_or_else(|| self.entry_error(e, "expected a string"))
    }
    fn name<'a>(&self, e: &'a KdlEntry) -> Result<&'a str, ConfigError> {
        let name = self.text(e)?;
        if name.trim().is_empty() {
            return Err(self.entry_error(e, "name must not be empty"));
        }
        Ok(name)
    }
    fn leaf(&self, node: &KdlNode) -> Result<(), ConfigError> {
        if node.children().is_some() {
            return Err(self.node_error(node, "unexpected child block"));
        }
        Ok(())
    }
    fn children<'a>(&self, node: &'a KdlNode) -> Result<&'a KdlDocument, ConfigError> {
        node.children()
            .ok_or_else(|| self.node_error(node, "expected a child block"))
    }
    fn duration(&self, node: &KdlNode, zero: bool) -> Result<Duration, ConfigError> {
        self.leaf(node)?;
        let e = self.one(node, &[])?;
        let value = self.text(e)?;
        let (digits, scale) = if let Some(n) = value.strip_suffix("ms") {
            (n, 1)
        } else if let Some(n) = value.strip_suffix('s') {
            (n, 1_000)
        } else if let Some(n) = value.strip_suffix('m') {
            (n, 60_000)
        } else {
            ("", 0)
        };
        // Dispatch deadlines use signed nanoseconds; reject overflow at the boundary.
        let millis = (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
            .then(|| digits.parse::<u64>().ok())
            .flatten()
            .and_then(|n| n.checked_mul(scale))
            .filter(|n| (*n > 0 || zero) && *n <= i64::MAX as u64 / 1_000_000);
        millis.map(Duration::from_millis).ok_or_else(|| {
            self.entry_error(
                e,
                "expected an integer duration in ms, s or m within timer range",
            )
        })
    }
    fn selector(&self, node: &KdlNode) -> Result<Selector, ConfigError> {
        self.leaf(node)?;
        let first = node
            .entries()
            .iter()
            .find(|e| e.name().is_none())
            .ok_or_else(|| self.node_error(node, "expected an event name"))?;
        let event_name = self.text(first)?;
        let kind = Kind::parse(event_name)
            .ok_or_else(|| self.entry_error(first, format!("unknown event '{event_name}'")))?;
        self.one(node, kind.fields())?;
        let mut fields = BTreeMap::new();
        for e in node.entries().iter().filter(|e| e.name().is_some()) {
            let key = e.name().unwrap().value();
            let value = if key == "id" {
                let id = e
                    .value()
                    .as_integer()
                    .and_then(|n| u32::try_from(n).ok())
                    .ok_or_else(|| {
                        self.entry_error(e, "display id must be a nonnegative 32-bit integer")
                    })?;
                Value::Id(id)
            } else {
                let s = self.text(e)?;
                if key == "source" && !["ac", "battery", "ups"].contains(&s) {
                    return Err(self.entry_error(e, "power source must be ac, battery or ups"));
                }
                Value::Text(s.into())
            };
            fields.insert(key.into(), value);
        }
        Ok(Selector { kind, fields })
    }
}

impl Config {
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        let r = Reader(text);
        let doc = KdlDocument::parse_v2(text).map_err(|e| {
            let diagnostic = e.diagnostics.first();
            r.error(
                diagnostic.map_or(0, |d| d.span.offset()),
                diagnostic
                    .and_then(|d| d.message.clone())
                    .unwrap_or_else(|| e.to_string()),
            )
        })?;
        let mut result = Self {
            max_parallel: 4,
            actions: Vec::new(),
            groups: Vec::new(),
        };
        let mut named_groups = BTreeMap::new();
        let mut parallel_seen = false;
        for node in doc.nodes() {
            match node.name().value() {
                "max-parallel" => {
                    r.leaf(node)?;
                    if parallel_seen {
                        return Err(r.node_error(node, "duplicate max-parallel"));
                    }
                    parallel_seen = true;
                    let e = r.one(node, &[])?;
                    result.max_parallel = e
                        .value()
                        .as_integer()
                        .and_then(|n| usize::try_from(n).ok())
                        .filter(|n| *n > 0)
                        .ok_or_else(|| {
                            r.entry_error(e, "max-parallel must be a positive integer")
                        })?;
                }
                "group" => {
                    let name = r.name(r.one(node, &[])?)?.to_owned();
                    if named_groups.contains_key(&name) {
                        return Err(r.node_error(node, format!("duplicate group '{name}'")));
                    }
                    let mut debounce = None;
                    for item in r.children(node)?.nodes() {
                        if item.name().value() != "debounce" {
                            return Err(r.node_error(item, "expected debounce"));
                        }
                        if debounce.is_some() {
                            return Err(r.node_error(item, "duplicate debounce"));
                        }
                        debounce = Some(r.duration(item, true)?);
                    }
                    named_groups.insert(name, result.groups.len());
                    result.groups.push(Group {
                        debounce: debounce.unwrap_or_default(),
                    });
                }
                "action" => {}
                other => return Err(r.node_error(node, format!("unknown node '{other}'"))),
            }
        }
        let mut names = BTreeSet::new();
        for node in doc.nodes().iter().filter(|n| n.name().value() == "action") {
            let name = r.name(r.one(node, &["group"])?)?.to_owned();
            if !names.insert(name.clone()) {
                return Err(r.node_error(node, format!("duplicate action '{name}'")));
            }
            let group = if let Some(e) = node.entry("group") {
                let name = r.name(e)?;
                *named_groups
                    .get(name)
                    .ok_or_else(|| r.entry_error(e, format!("unknown group '{name}'")))?
            } else {
                let id = result.groups.len();
                result.groups.push(Group {
                    debounce: Duration::ZERO,
                });
                id
            };
            let mut action = Action {
                name,
                selectors: Vec::new(),
                steps: Vec::new(),
                timeout: Duration::from_secs(30),
                group,
            };
            let mut timeout_seen = false;
            for item in r.children(node)?.nodes() {
                match item.name().value() {
                    "on" => action.selectors.push(r.selector(item)?),
                    "timeout" => {
                        if timeout_seen {
                            return Err(r.node_error(item, "duplicate timeout"));
                        }
                        timeout_seen = true;
                        action.timeout = r.duration(item, false)?;
                    }
                    "exec" => {
                        r.leaf(item)?;
                        let args = r.args(item, &[])?;
                        if args.is_empty() {
                            return Err(r.node_error(item, "exec requires a program"));
                        }
                        r.name(args[0])?;
                        action.steps.push(
                            args.into_iter()
                                .map(|e| r.text(e).map(String::from))
                                .collect::<Result<_, _>>()?,
                        );
                    }
                    "shell" => {
                        r.leaf(item)?;
                        action.steps.push(vec![
                            "/bin/sh".into(),
                            "-c".into(),
                            r.text(r.one(item, &[])?)?.into(),
                        ]);
                    }
                    other => {
                        return Err(r.node_error(item, format!("unknown action node '{other}'")));
                    }
                }
            }
            if action.selectors.is_empty() || action.steps.is_empty() {
                return Err(
                    r.node_error(node, "action requires at least one on and one exec/shell")
                );
            }
            result.actions.push(action);
        }
        Ok(result)
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Self::parse(&text).map_err(|e| format!("{}:{e}", path.display()))
    }

    pub fn kinds(&self) -> BTreeSet<Kind> {
        self.actions
            .iter()
            .flat_map(|action| action.selectors.iter().map(|selector| selector.kind))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_raw_strings_and_defaults() {
        let c = Config::parse("action привет {\n on app.activated bundle-id=editor name=Редактор\n on app.activated\n shell #\"\"\"\n echo \"$HOME\"\n \"\"\"#\n}\n").unwrap();
        assert_eq!(c.max_parallel, 4);
        assert_eq!(c.actions[0].name, "привет");
        assert_eq!(c.actions[0].timeout, Duration::from_secs(30));
        assert_eq!(c.actions[0].steps[0], ["/bin/sh", "-c", "echo \"$HOME\""]);
        assert_eq!(c.groups.len(), 1);
        assert_eq!(c.groups[0].debounce, Duration::ZERO);
        assert_eq!(c.actions[0].group, 0);
        assert_eq!(c.kinds(), BTreeSet::from([Kind::AppActivated]));
        assert_eq!(
            c.actions[0].selectors[0].fields,
            BTreeMap::from([
                ("bundle-id".into(), Value::Text("editor".into())),
                ("name".into(), Value::Text("Редактор".into())),
            ])
        );
        assert!(c.actions[0].selectors[1].fields.is_empty());
    }

    #[test]
    fn forward_groups_and_numeric_filters() {
        let c = Config::parse("action a group=g { on screen.connected id=12; exec p; }\naction b group=g { on screen.connected; exec p; }\ngroup g { debounce \"1m\"; }\n").unwrap();
        assert_eq!(c.groups.len(), 1);
        assert_eq!(c.groups[0].debounce, Duration::from_secs(60));
        assert_eq!(c.actions[0].group, 0);
        assert_eq!(c.actions[1].group, 0);
        assert_eq!(c.actions[0].selectors[0].fields["id"], Value::Id(12));
    }

    #[test]
    fn rejects_invalid_schema_with_locations() {
        let bad = [
            "unknown 1",
            "max-parallel 0",
            "max-parallel 1\nmax-parallel 2",
            "action a {}",
            "group g {}\ngroup g {}",
            "action a group=missing { on system.wake; exec p; }",
            "action a { on system.sleep; exec p; }",
            "action a { on system.wake name=x; exec p; }",
            "action a { on power.changed source=foo; exec p; }",
            "action a { on screen.connected id=\"12\"; exec p; }",
            "action a { on screen.connected name=x name=y; exec p; }",
            "action a { on system.wake; exec 12; }",
            "action a { on system.wake; exec p; timeout \"0s\"; }",
            "action a { on system.wake; exec p; timeout \"1.5s\"; }",
            "action a { on system.wake; exec p; timeout \"1s\"; timeout \"2s\"; }",
            "action a { on system.wake; exec p; timeout \"18446744073709551615m\"; }",
            "action a { on system.wake; exec p; }\naction a { on system.wake; exec p; }",
            "action a group=g group=g { on system.wake; exec p; }\ngroup g {}",
            "action a { on system.wake; shell \"x\" {} }",
            "action a { on system.wake; exec (string)p; }",
            "action \"\" { on system.wake; exec p; }",
            "group g { debounce \"1s\"; debounce \"2s\"; }",
            "group g { debounce \"-1ms\"; }",
            "action a { on system.wake; shell r\"v1 raw\"; }",
            "action a { on screen.connected id=#true; exec p; }",
            "action a { on system.wake; shell #null; }",
        ];
        for text in bad {
            let error = Config::parse(text).unwrap_err();
            assert!(error.line > 0 && error.column > 0, "{text}");
        }
        assert_eq!(Config::parse("\nwat 1").unwrap_err().line, 2);
        let error = Config::parse("action привет { on system.wake wrong=1; exec p; }").unwrap_err();
        assert_eq!((error.line, error.column), (1, 32));
        assert!(Config::parse("group g { debounce \"0ms\"; }").is_ok());
        assert!(Config::parse("").is_ok());
    }
}
