pub struct Menu<T> {
    entries: Vec<T>,
    pub selected_idx: usize,
}

impl<T> Menu<T> {
    pub fn into_selected_entry(mut self) -> Option<T> {
        if self.selected_idx < self.entries.len() {
            Some(self.entries.remove(self.selected_idx))
        } else {
            None
        }
    }

    // https://depth-first.com/articles/2020/06/22/returning-rust-iterators/
    pub fn iter(&self) -> impl Iterator<Item = (&T, bool)> + '_ {
        self.entries
            .iter()
            .enumerate()
            .map(move |(idx, entry)| (entry, idx == self.selected_idx))
    }

    pub fn select_next(&mut self) {
        if self.entries.len() > 0 && self.selected_idx < self.entries.len() - 1 {
            self.selected_idx += 1;
        } else {
            self.selected_idx = 0;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        } else if self.entries.len() > 0 {
            self.selected_idx = self.entries.len() - 1;
        }
    }
}

impl<T> From<Vec<T>> for Menu<T> {
    fn from(entries: Vec<T>) -> Menu<T> {
        Menu {
            entries,
            selected_idx: 0,
        }
    }
}

#[test]
fn test_it_works_with_filled_vec() {
    let mut menu = Menu::from(vec![1, 2, 3]);
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

    assert_eq!(menu.into_selected_entry(), Some(1));
}

#[test]
fn test_it_works_with_empty_vec() {
    let mut menu: Menu<i32> = Menu::from(vec![]);
    menu.select_next();
    assert_eq!(menu.selected_idx, 0);
    menu.select_prev();
    assert_eq!(menu.selected_idx, 0);
    assert_eq!(menu.into_selected_entry(), None);
}
