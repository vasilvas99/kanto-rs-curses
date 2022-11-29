use cursive::align::HAlign;
use cursive_table_view::{TableView, TableViewItem};
use crate::kanto_api;
use std::cmp::Ordering;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ContainerColumn {
    ID,
    Name,
    Image,
    State,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct ContainersTable {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
}

impl TableViewItem<ContainerColumn> for ContainersTable {
    fn to_column(&self, column: ContainerColumn) -> String {
        match column {
            ContainerColumn::ID => self.id.to_string(),
            ContainerColumn::Name => self.name.to_string(),
            ContainerColumn::Image => self.image.to_string(),
            ContainerColumn::State => self.state.to_string(),
        }
    }

    fn cmp(&self, other: &Self, column: ContainerColumn) -> Ordering
    where
        Self: Sized,
    {
        match column {
            ContainerColumn::ID => self.id.cmp(&other.id),
            ContainerColumn::Name => self.name.cmp(&other.name),
            ContainerColumn::Image => self.image.cmp(&other.image),
            ContainerColumn::State => self.state.cmp(&other.state),
        }
    }
}

fn state_to_string(state: &Option<kanto_api::cm_types::State>) -> String {
    if let Some(state) = state {
        return state.status.clone();
    }

    String::from("Unknown?")
}
pub fn items_to_columns(req_items: Vec<kanto_api::Container>) -> Vec<ContainersTable> {
    let mut out: Vec<ContainersTable> = vec![];

    for c in req_items {
        out.push(ContainersTable {
            id: c.id,
            name: c.name,
            image: c.image.expect("Missing field").name,
            state: state_to_string(&c.state),
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

pub fn generate_table_view() -> TableView<ContainersTable, ContainerColumn> {
    TableView::<ContainersTable, ContainerColumn>::new()
        .column(ContainerColumn::ID, "ID", |c| c.width_percent(20))
        .column(ContainerColumn::Name, "Name", |c| c.align(HAlign::Center))
        .column(ContainerColumn::Image, "Image", |c| {
            c.ordering(Ordering::Greater)
                .align(HAlign::Right)
                .width_percent(20)
        })
        .column(ContainerColumn::State, "State", |c| c.align(HAlign::Center))
}
