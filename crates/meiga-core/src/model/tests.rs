use std::collections::{HashMap, HashSet};

use super::{Close, Error, Switch, Tab, TabId, Window};

/// A window with a tab for each name, the first one active, and the
/// identifiers of its tabs.
fn window(names: &[&'static str]) -> (Window<&'static str>, Vec<TabId>) {
    let mut window = Window::new(names[0]);
    check(&window);
    let mut ids = vec![window.active().id()];
    for name in &names[1..] {
        ids.push(window.open(*name));
        check(&window);
    }
    (window, ids)
}

/// The contents of the tabs, from left to right.
fn names<T: Copy>(window: &Window<T>) -> Vec<T> {
    window.tabs().iter().map(|tab| *tab.content()).collect()
}

/// What has to hold after any change.
fn check<T>(window: &Window<T>) {
    assert!(window.tab_count() >= 1, "a window without tabs");
    assert_eq!(window.tabs().len(), window.tab_count());
    assert!(window.active_index() < window.tab_count());
    let active = window.active().id();
    assert_eq!(window.tabs()[window.active_index()].id(), active);
    assert_eq!(window.focused().id(), active);
    let ids: HashSet<TabId> = window.tabs().iter().map(Tab::id).collect();
    assert_eq!(ids.len(), window.tab_count(), "a repeated identifier");
    for (index, tab) in window.tabs().iter().enumerate() {
        assert_eq!(window.index_of(tab.id()), Some(index));
        assert_eq!(window.is_visible(tab.id()), tab.id() == active);
    }
}

/// The answer to closing a tab that leaves the window.
fn removed<T>(content: T, activated: Option<TabId>) -> Result<Close<T>, Error> {
    Ok(Close::Removed { content, activated })
}

/// The answer to a change of the active tab.
const fn switched(from: TabId, to: TabId) -> Option<Switch> {
    Some(Switch { from, to })
}

#[test]
fn a_new_window_has_one_tab_and_it_is_active() {
    let window = Window::new("a");
    check(&window);
    assert_eq!(names(&window), ["a"]);
    assert_eq!(window.active_index(), 0);
    assert_eq!(window.active().content(), &"a");
}

#[test]
fn a_tab_opens_at_the_end_without_becoming_active() {
    let (mut window, ids) = window(&["a", "b"]);
    let c = window.open("c");
    check(&window);
    assert_eq!(names(&window), ["a", "b", "c"]);
    assert_eq!(window.active().id(), ids[0]);
    assert_eq!(window.index_of(c), Some(2));
    assert_eq!(window.get(c), Some(&"c"));
}

#[test]
fn a_content_can_know_the_identifier_of_its_tab() {
    let mut window = Window::try_new_with(Ok::<_, ()>).unwrap();
    check(&window);
    let first = window.active().id();
    let second = window.try_open_with(Ok::<_, ()>).unwrap();
    check(&window);
    assert_eq!(names(&window), [first, second]);
    assert_eq!(window.get(second), Some(&second));
    assert_eq!(window.active().id(), first);
}

#[test]
fn a_content_that_cannot_be_made_opens_nothing() {
    let failed = Window::<&str>::try_new_with(|_| Err("no session"));
    assert_eq!(failed.map(|window| window.tab_count()), Err("no session"));
    let (mut window, ids) = window(&["a", "b"]);
    assert_eq!(
        window.try_open_with(|_| Err("no session")),
        Err("no session")
    );
    check(&window);
    assert_eq!(names(&window), ["a", "b"]);
    assert_eq!(window.active().id(), ids[0]);
}

#[test]
fn an_identifier_comes_back_from_its_number() {
    let (mut window, ids) = window(&["a", "b"]);
    let id = TabId::from_raw(ids[1].get());
    assert_eq!(id, ids[1]);
    assert_eq!(window.index_of(id), Some(1));
    assert_eq!(window.activate(id), Ok(switched(ids[0], ids[1])));
    check(&window);
    // The counter starts at 1, so no tab has the number 0.
    let none = TabId::from_raw(0);
    assert_eq!(window.get(none), None);
    assert_eq!(window.close(none), Err(Error::NoSuchTab(none)));
    check(&window);
}

#[test]
fn identifiers_never_repeat_not_even_in_another_window_or_after_closing() {
    let (mut first, ids) = window(&["a", "b", "c"]);
    let (second, _) = window(&["d", "e"]);
    let mut seen: HashSet<TabId> = first
        .tabs()
        .iter()
        .chain(second.tabs())
        .map(Tab::id)
        .collect();
    assert_eq!(seen.len(), 5);
    assert_ne!(first.id(), second.id());
    assert_eq!(first.close(ids[2]), removed("c", None));
    check(&first);
    assert!(seen.insert(first.open("f")));
    check(&first);
}

#[test]
fn closing_the_active_tab_activates_the_one_on_its_right() {
    let (mut window, ids) = window(&["a", "b", "c"]);
    assert_eq!(window.activate(ids[1]), Ok(switched(ids[0], ids[1])));
    check(&window);
    assert_eq!(window.close(ids[1]), removed("b", Some(ids[2])));
    check(&window);
    assert_eq!(names(&window), ["a", "c"]);
    assert_eq!(window.active().id(), ids[2]);
}

#[test]
fn closing_the_last_tab_when_it_is_active_activates_the_one_on_its_left() {
    let (mut window, ids) = window(&["a", "b", "c"]);
    assert_eq!(window.activate(ids[2]), Ok(switched(ids[0], ids[2])));
    check(&window);
    assert_eq!(window.close(ids[2]), removed("c", Some(ids[1])));
    check(&window);
    assert_eq!(names(&window), ["a", "b"]);
    assert_eq!(window.active().id(), ids[1]);
}

#[test]
fn closing_another_tab_keeps_the_active_one() {
    let (mut window, ids) = window(&["a", "b", "c", "d"]);
    assert_eq!(window.activate(ids[2]), Ok(switched(ids[0], ids[2])));
    check(&window);
    // A tab on its left: the active tab moves one place to the left.
    assert_eq!(window.close(ids[0]), removed("a", None));
    check(&window);
    assert_eq!(names(&window), ["b", "c", "d"]);
    assert_eq!(window.active_index(), 1);
    // A tab on its right: the active tab does not move.
    assert_eq!(window.close(ids[3]), removed("d", None));
    check(&window);
    assert_eq!(names(&window), ["b", "c"]);
    assert_eq!(window.active().id(), ids[2]);
}

#[test]
fn the_only_tab_stays_and_the_window_closes_with_it() {
    let (mut window, ids) = window(&["a", "b"]);
    assert_eq!(window.close(ids[0]), removed("a", Some(ids[1])));
    check(&window);
    assert_eq!(window.close(ids[1]), Ok(Close::LastTab));
    check(&window);
    assert_eq!(names(&window), ["b"]);
}

#[test]
fn moving_a_tab_shifts_the_ones_in_between() {
    let (mut window, ids) = window(&["a", "b", "c", "d", "e"]);
    let moves = [
        (ids[1], 3, ["a", "c", "d", "b", "e"]),
        (ids[4], 0, ["e", "a", "c", "d", "b"]),
        // To its own position: nothing changes.
        (ids[2], 2, ["e", "a", "c", "d", "b"]),
    ];
    for (id, to, expected) in moves {
        assert_eq!(window.move_tab(id, to), Ok(()));
        check(&window);
        assert_eq!(names(&window), expected);
    }
}

#[test]
fn the_active_tab_stays_the_same_tab_when_tabs_move() {
    let (mut window, ids) = window(&["a", "b", "c", "d"]);
    assert_eq!(window.activate(ids[1]), Ok(switched(ids[0], ids[1])));
    check(&window);
    let moves = [
        // The active tab itself.
        (ids[1], 3, ["a", "c", "d", "b"]),
        // A tab from its left to its right.
        (ids[0], 3, ["c", "d", "b", "a"]),
        // A tab from its right to its left.
        (ids[0], 0, ["a", "c", "d", "b"]),
        // A tab that does not cross it.
        (ids[2], 2, ["a", "d", "c", "b"]),
    ];
    for (id, to, expected) in moves {
        assert_eq!(window.move_tab(id, to), Ok(()));
        check(&window);
        assert_eq!(names(&window), expected);
        assert_eq!(window.active().id(), ids[1]);
    }
}

#[test]
fn a_position_past_the_last_tab_is_refused() {
    let (mut window, ids) = window(&["a", "b", "c"]);
    assert_eq!(
        window.move_tab(ids[0], 3),
        Err(Error::OutOfRange { index: 3, count: 3 })
    );
    check(&window);
    assert_eq!(names(&window), ["a", "b", "c"]);
}

#[test]
fn a_closed_tab_is_refused() {
    let (mut window, ids) = window(&["a", "b"]);
    assert_eq!(window.close(ids[1]), removed("b", None));
    check(&window);
    let gone = ids[1];
    assert_eq!(window.close(gone), Err(Error::NoSuchTab(gone)));
    check(&window);
    assert_eq!(window.move_tab(gone, 0), Err(Error::NoSuchTab(gone)));
    check(&window);
    assert_eq!(window.activate(gone), Err(Error::NoSuchTab(gone)));
    check(&window);
    assert_eq!(window.get(gone), None);
    assert_eq!(window.get_mut(gone), None);
    assert_eq!(window.index_of(gone), None);
    assert!(!window.is_visible(gone));
    assert_eq!(names(&window), ["a"]);
}

#[test]
fn a_tab_of_another_window_is_refused() {
    let (mut first, _) = window(&["a", "b"]);
    let (second, _) = window(&["c"]);
    let other = second.active().id();
    assert_eq!(first.activate(other), Err(Error::NoSuchTab(other)));
    check(&first);
    assert_eq!(first.close(other), Err(Error::NoSuchTab(other)));
    check(&first);
    assert_eq!(names(&first), ["a", "b"]);
}

#[test]
fn activating_answers_the_switch_or_none_if_it_was_already_active() {
    let (mut window, ids) = window(&["a", "b"]);
    assert_eq!(window.activate(ids[1]), Ok(switched(ids[0], ids[1])));
    check(&window);
    assert_eq!(window.activate(ids[1]), Ok(None));
    check(&window);
}

#[test]
fn next_and_previous_go_round_the_bar() {
    let (mut window, ids) = window(&["a", "b", "c"]);
    let steps = [
        (true, 0, 1),
        (true, 1, 2),
        (true, 2, 0),
        (false, 0, 2),
        (false, 2, 1),
    ];
    for (next, from, to) in steps {
        let switch = if next {
            window.activate_next()
        } else {
            window.activate_previous()
        };
        assert_eq!(switch, switched(ids[from], ids[to]));
        check(&window);
    }
}

#[test]
fn with_one_tab_next_and_previous_change_nothing() {
    let mut window = Window::new("a");
    assert_eq!(window.activate_next(), None);
    check(&window);
    assert_eq!(window.activate_previous(), None);
    check(&window);
}

#[test]
fn the_keyboard_and_the_screen_follow_the_active_tab() {
    let (mut window, ids) = window(&["a", "b"]);
    assert_eq!(window.activate(ids[1]), Ok(switched(ids[0], ids[1])));
    check(&window);
    assert_eq!(window.focused().id(), ids[1]);
    assert!(window.is_visible(ids[1]));
    assert!(!window.is_visible(ids[0]));
    *window.focused_mut() = "B";
    check(&window);
    if let Some(content) = window.get_mut(ids[0]) {
        *content = "A";
    }
    check(&window);
    assert_eq!(names(&window), ["A", "B"]);
}

#[test]
fn errors_say_what_failed() {
    let (_, ids) = window(&["a"]);
    assert_eq!(
        Error::NoSuchTab(ids[0]).to_string(),
        format!("the window has no tab {}", ids[0].get())
    );
    assert_eq!(
        Error::OutOfRange { index: 3, count: 3 }.to_string(),
        "position 3 is out of range for 3 tabs"
    );
    assert_eq!(
        Error::OutOfRange { index: 1, count: 1 }.to_string(),
        "position 1 is out of range for 1 tab"
    );
}

#[test]
fn random_changes_keep_the_rules_and_match_a_plain_list() {
    // A fixed pseudo-random sequence (xorshift), so that a failure can be
    // repeated.
    let mut state = 0x2545_F491_4F6C_DD1D_u64;
    let mut next = |limit: usize| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        usize::try_from(state % limit as u64).unwrap_or(0)
    };
    let mut sizes = HashSet::new();
    for _ in 0..20 {
        // The same window as a plain list: the contents of the tabs, from
        // left to right, and the content of the active tab.
        let mut window = Window::new(0_u32);
        let mut list = vec![0_u32];
        let mut active = 0_u32;
        let mut ids = HashMap::from([(0_u32, window.active().id())]);
        let mut closed = Vec::new();
        let mut last = 0_u32;
        for _ in 0..500 {
            let count = list.len();
            sizes.insert(count);
            match next(8) {
                0 | 1 => {
                    last += 1;
                    let id = match next(3) {
                        0 => window.open(last),
                        1 => window.try_open_with(|_| Ok::<_, ()>(last)).unwrap(),
                        _ => {
                            // A content that cannot be made opens nothing.
                            assert_eq!(window.try_open_with(|_| Err::<u32, _>(())), Err(()));
                            check(&window);
                            assert_eq!(names(&window), list);
                            window.open(last)
                        }
                    };
                    ids.insert(last, id);
                    list.push(last);
                }
                2 | 3 => {
                    let index = next(count);
                    let tab = list[index];
                    let answer = window.close(ids[&tab]);
                    if count == 1 {
                        assert_eq!(answer, Ok(Close::LastTab));
                    } else {
                        list.remove(index);
                        let activated = if tab == active {
                            active = list[index.min(count - 2)];
                            Some(ids[&active])
                        } else {
                            None
                        };
                        assert_eq!(answer, removed(tab, activated));
                        closed.push(ids[&tab]);
                    }
                }
                4 => {
                    // Sometimes one past the last tab.
                    let (index, to) = (next(count), next(count + 1));
                    let tab = list[index];
                    let answer = window.move_tab(ids[&tab], to);
                    if to == count {
                        assert_eq!(answer, Err(Error::OutOfRange { index: to, count }));
                    } else {
                        assert_eq!(answer, Ok(()));
                        list.remove(index);
                        list.insert(to, tab);
                    }
                }
                5 | 6 => {
                    let position = list.iter().position(|tab| *tab == active).unwrap_or(0);
                    let (answer, target) = match next(3) {
                        0 => {
                            let target = list[next(count)];
                            (window.activate(ids[&target]), target)
                        }
                        1 => (Ok(window.activate_next()), list[(position + 1) % count]),
                        _ => (
                            Ok(window.activate_previous()),
                            list[(position + count - 1) % count],
                        ),
                    };
                    let switch = (target != active).then(|| Switch {
                        from: ids[&active],
                        to: ids[&target],
                    });
                    assert_eq!(answer, Ok(switch));
                    active = target;
                }
                _ => {
                    if !closed.is_empty() {
                        let gone = closed[next(closed.len())];
                        assert_eq!(window.close(gone), Err(Error::NoSuchTab(gone)));
                        check(&window);
                        assert_eq!(window.activate(gone), Err(Error::NoSuchTab(gone)));
                    }
                }
            }
            check(&window);
            assert_eq!(names(&window), list);
            assert_eq!(*window.active().content(), active);
        }
    }
    // The bar went from one tab to many: the edges were tried too.
    assert!(sizes.contains(&1) && sizes.contains(&2) && sizes.iter().any(|size| *size >= 10));
}
