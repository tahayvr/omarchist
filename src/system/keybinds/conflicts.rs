// Chord collisions. Hyprland runs every bind registered on a chord, so two
// binds sharing one are a conflict unless Omarchy stacked them on purpose
// (its `ALT + TAB` cycles *and* raises the window). The rule: a group is a
// conflict when it has two or more active members and at least one of them
// is not an Omarchy default. Press and release binds on the same key are
// separate groups.
use std::collections::HashMap;

use super::chord::{Chord, ModMask};
use super::{BindIdentity, Keybind, Origin};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    pub chord: Chord,
    /// Indices into the scanned bind list.
    pub rows: Vec<usize>,
}

type GroupKey = (ModMask, String, bool);

fn group_key(bind: &Keybind) -> GroupKey {
    let (mods, key) = bind.chord.id();
    (mods, key, bind.options.release)
}

pub fn find_conflicts(binds: &[Keybind]) -> Vec<Conflict> {
    let mut groups: HashMap<GroupKey, Vec<usize>> = HashMap::new();
    for (ix, bind) in binds.iter().enumerate().filter(|(_, b)| b.is_active()) {
        groups.entry(group_key(bind)).or_default().push(ix);
    }

    let mut conflicts: Vec<Conflict> = groups
        .into_values()
        .filter(|rows| rows.len() >= 2)
        .filter(|rows| rows.iter().any(|&ix| binds[ix].origin != Origin::Default))
        .map(|rows| Conflict {
            chord: binds[rows[0]].chord.clone(),
            rows,
        })
        .collect();
    conflicts.sort_by_key(|c| c.rows[0]);
    conflicts
}

/// Active binds that would fire on `chord`, for the editor's live notice.
/// `exclude` drops the bind being edited so it does not conflict with itself.
pub fn binds_on_chord<'a>(
    binds: &'a [Keybind],
    chord: &Chord,
    release: bool,
    exclude: Option<&BindIdentity>,
) -> Vec<&'a Keybind> {
    binds
        .iter()
        .filter(|b| b.is_active() && b.options.release == release && b.chord.same_as(chord))
        .filter(|b| exclude.is_none_or(|id| b.identity() != *id))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::system::keybinds::{BindOptions, BindStatus, Dispatcher};

    fn bind(seq: u32, keys: &str, origin: Origin, release: bool) -> Keybind {
        Keybind {
            seq,
            chord: Chord::parse(keys).unwrap(),
            keys_raw: keys.into(),
            description: format!("bind {seq}"),
            dispatcher: Dispatcher::Exec(format!("cmd {seq}")),
            options: BindOptions {
                release,
                ..BindOptions::default()
            },
            origin,
            source: PathBuf::from("/x.lua"),
            status: BindStatus::Active,
        }
    }

    #[test]
    fn default_stacks_are_not_conflicts_but_user_collisions_are() {
        let binds = vec![
            bind(1, "ALT + TAB", Origin::Default, false),
            bind(2, "ALT + TAB", Origin::Default, false),
            bind(3, "SUPER + K", Origin::Default, false),
            bind(4, "SUPER + k", Origin::User, false),
            bind(5, "F9", Origin::Default, false),
            bind(6, "F9", Origin::Default, true),
            bind(7, "SUPER + 1", Origin::Default, false),
            bind(8, "SUPER + code:10", Origin::Omarchist, false),
        ];
        let conflicts = find_conflicts(&binds);
        assert_eq!(conflicts.len(), 2);
        assert_eq!(conflicts[0].rows, vec![2, 3]);
        assert_eq!(conflicts[1].rows, vec![6, 7]);
    }

    #[test]
    fn unbound_binds_are_ignored() {
        let mut binds = vec![
            bind(1, "SUPER + K", Origin::Default, false),
            bind(2, "SUPER + K", Origin::User, false),
        ];
        binds[0].status = BindStatus::Unbound {
            by_seq: 2,
            by_origin: Origin::User,
        };
        assert!(find_conflicts(&binds).is_empty());
    }

    #[test]
    fn binds_on_chord_excludes_the_edited_bind() {
        let binds = vec![
            bind(1, "SUPER + K", Origin::Default, false),
            bind(2, "SUPER + K", Origin::User, false),
            bind(3, "SUPER + K", Origin::User, true),
        ];
        let chord = Chord::parse("SUPER + K").unwrap();
        let identity = binds[0].identity();
        let others = binds_on_chord(&binds, &chord, false, Some(&identity));
        assert_eq!(others.len(), 1);
        assert_eq!(others[0].seq, 2);
        assert_eq!(binds_on_chord(&binds, &chord, false, None).len(), 2);
    }
}
