//use strum::{Display, EnumIter};
use crate::components::combobox::ComboboxItem;


// =========================================================
//                   DATABASE STRUCTURE
// =========================================================
#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq)]
pub struct Client {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub created_at: String,
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub revision: Option<String>,
    pub client_id: i64,
    pub client_name: String,
    pub created_at: String,
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag(pub String);
impl Tag {
    pub fn new(str: &str) -> Self {
        Tag(str.to_string())
    }
}
impl std::ops::Deref for Tag {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl AsRef<str> for Tag {
    fn as_ref(&self) -> &str {
        &self.0
    }
}


//region -------- Combobox Searchability --------
// =========================================================
//                 COMBOBOX SEARCHABILITY
// =========================================================
impl ComboboxItem for Client {
    fn id(&self) -> i64 {
        self.id
    }

    fn label(&self) -> String {
        self.name.clone()
    }
}

impl ComboboxItem for Project {
    fn id(&self) -> i64 {
        self.id
    }

    fn label(&self) -> String {
        if let Some(rev) = self.revision.clone() {
            format!("{} ({})", self.name.clone(), rev)
        } else {
            format!("{}", self.name.clone())
        }
    }
}
//endregion