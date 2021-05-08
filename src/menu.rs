use std::convert::TryFrom;

pub struct Menu<T> {
    entries: Vec<T>,
    selected_idx: usize,
}

impl<T> Menu<T> {
    pub fn into_selected_entry(mut self) -> T {
        self.entries.remove(self.selected_idx)
    }

    pub fn selected_entry(&self) -> &T {
        self.entries.get(self.selected_idx).unwrap()
    }

    // https://depth-first.com/articles/2020/06/22/returning-rust-iterators/
    pub fn iter(&self) -> impl Iterator<Item = (&T, bool)> + '_ {
        self.entries
            .iter()
            .enumerate()
            .map(move |(idx, entry)| (entry, idx == self.selected_idx))
    }

    pub fn select_next(&mut self) {
        if self.selected_idx < self.entries.len() - 1 {
            self.selected_idx += 1;
        } else {
            self.selected_idx = 0;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        } else {
            self.selected_idx = self.entries.len() - 1;
        }
    }
}

impl<T> TryFrom<Vec<T>> for Menu<T> {
    type Error = &'static str;

    fn try_from(entries: Vec<T>) -> Result<Self, Self::Error> {
        if entries.is_empty() {
            Err("Cannot create empty menus!")
        } else {
            Ok(Menu {
                entries,
                selected_idx: 0,
            })
        }
    }
}

#[test]
fn test_it_works_with_filled_vec() {
    let mut menu = Menu::try_from(vec![1, 2, 3]).unwrap();
    assert_eq!(menu.selected_idx, 0);
    menu.select_next();
    assert_eq!(menu.selected_idx, 1);
    menu.select_prev();
    assert_eq!(menu.selected_idx, 0);
    menu.select_prev();
    assert_eq!(menu.selected_idx, 2);
    menu.select_next();
    assert_eq!(menu.selected_idx, 0);

    let mut menu_items: Vec<(i32, bool)> = vec![];

    for (value, is_selected) in menu.iter() {
        menu_items.push((*value, is_selected));
    }

    assert_eq!(menu_items, vec![(1, true), (2, false), (3, false),]);

    assert_eq!(menu.into_selected_entry(), 1);
}

#[test]
fn test_try_from_fails_with_empty_vec() {
    let menu = Menu::<usize>::try_from(vec![]);
    assert_eq!(menu.is_err(), true);
}
