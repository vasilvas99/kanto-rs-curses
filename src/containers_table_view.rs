use cursive_table_view::TableViewItem;
use kantocurses::kanto_api;
use std::cmp::Ordering;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ContainerColumn {
    ID,
    Name,
    Image,
    Running,
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct ContainersTable {
    pub id: String,
    pub name: String,
    pub image: String,
    pub running: String,
}

impl TableViewItem<ContainerColumn> for ContainersTable {
    fn to_column(&self, column: ContainerColumn) -> String {
        match column {
            ContainerColumn::ID => self.id.to_string(),
            ContainerColumn::Name => self.name.to_string(),
            ContainerColumn::Image => self.image.to_string(),
            ContainerColumn::Running => self.running.to_string(),
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
            ContainerColumn::Running => self.running.cmp(&other.running),
        }
    }
}

pub fn items_to_columns(req_items: Vec<kanto_api::Container>) -> Vec<ContainersTable> {
    let mut out: Vec<ContainersTable> = vec![];

    for c in req_items {
        let running = if c.state.expect("Missing field").running {
            String::from("Yes")
        } else {
            String::from("No")
        };

        out.push(ContainersTable {
            id: c.id,
            name: c.name,
            image: c.image.expect("Missing field").name,
            running,
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}
