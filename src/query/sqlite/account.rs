// ref: https://piecash.readthedocs.io/en/master/object_model.html
// ref: https://wiki.gnucash.org/wiki/SQL

use rusqlite::Row;

use super::SQLiteQuery;
use crate::error::Error;
use crate::query::{AccountQ, AccountT};

#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct Account {
    pub(crate) guid: String,
    pub(crate) name: String,
    pub(crate) account_type: String,
    pub(crate) commodity_guid: Option<String>,
    pub(crate) commodity_scu: i64,
    pub(crate) non_std_scu: i64,
    pub(crate) parent_guid: Option<String>,
    pub(crate) code: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) hidden: Option<i64>,
    pub(crate) placeholder: Option<i64>,
}

impl<'a> TryFrom<&'a Row<'a>> for Account {
    type Error = rusqlite::Error;

    fn try_from(row: &'a Row<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            guid: row.get(0)?,
            name: row.get(1)?,
            account_type: row.get(2)?,
            commodity_guid: row.get(3)?,
            commodity_scu: row.get(4)?,
            non_std_scu: row.get(5)?,
            parent_guid: row.get(6)?,
            code: row.get(7)?,
            description: row.get(8)?,
            hidden: row.get(9)?,
            placeholder: row.get(10)?,
        })
    }
}

impl AccountT for Account {
    fn guid(&self) -> String {
        self.guid.clone()
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    fn account_type(&self) -> String {
        self.account_type.clone()
    }
    fn commodity_guid(&self) -> String {
        self.commodity_guid.clone().unwrap_or_default()
    }
    fn commodity_scu(&self) -> i64 {
        self.commodity_scu
    }
    fn non_std_scu(&self) -> bool {
        self.non_std_scu != 0
    }
    fn parent_guid(&self) -> String {
        self.parent_guid.clone().unwrap_or_default()
    }
    fn code(&self) -> String {
        self.code.clone().unwrap_or_default()
    }
    fn description(&self) -> String {
        self.description.clone().unwrap_or_default()
    }
    fn hidden(&self) -> bool {
        self.hidden.is_some_and(|x| x != 0)
    }
    fn placeholder(&self) -> bool {
        self.placeholder.is_some_and(|x| x != 0)
    }
}

const SEL: &str = r"
SELECT
guid,
name,
account_type,
commodity_guid,
commodity_scu,
non_std_scu,
parent_guid,
code,
description,
hidden,
placeholder
FROM accounts
";

impl AccountQ for SQLiteQuery {
    type A = Account;

    async fn all(&self) -> Result<Vec<Self::A>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(SEL)?;
        let result = stmt
            .query([])?
            .mapped(|row| Account::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    async fn guid(&self, guid: &str) -> Result<Vec<Self::A>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!("{SEL}\nWHERE guid = ?"))?;
        let result = stmt
            .query([guid])?
            .mapped(|row| Account::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    async fn commodity_guid(&self, guid: &str) -> Result<Vec<Self::A>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!("{SEL}\nWHERE commodity_guid = ?"))?;
        let result = stmt
            .query([guid])?
            .mapped(|row| Account::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    async fn parent_guid(&self, guid: &str) -> Result<Vec<Self::A>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!("{SEL}\nWHERE parent_guid = ?"))?;
        let result = stmt
            .query([guid])?
            .mapped(|row| Account::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    async fn name(&self, name: &str) -> Result<Vec<Self::A>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!("{SEL}\nWHERE name = ?"))?;
        let result = stmt
            .query([name])?
            .mapped(|row| Account::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    async fn contains_name_ignore_case(&self, name: &str) -> Result<Vec<Self::A>, Error> {
        let name = format!("%{name}%");

        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!("{SEL}\nWHERE name LIKE ?"))?;
        let result = stmt
            .query([name])?
            .mapped(|row| Account::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use tokio::sync::OnceCell;

    #[cfg(feature = "schema")]
    // test schemas on compile time
    #[allow(dead_code)]
    fn test_account_schemas() {
        let _ = sqlx::query_as!(
            Account,
            r"
				SELECT 
				guid, 
				name,
				account_type,
				commodity_guid,
				commodity_scu,
				non_std_scu,
				parent_guid,
				code,
				description,
				hidden,
				placeholder
				FROM accounts
				"
        );
    }

    static Q: OnceCell<SQLiteQuery> = OnceCell::const_new();
    async fn setup() -> &'static SQLiteQuery {
        Q.get_or_init(|| async {
            let uri: &str = &format!(
                "{}/tests/db/sqlite/complex_sample.gnucash",
                env!("CARGO_MANIFEST_DIR")
            );

            println!("work_dir: {:?}", std::env::current_dir());
            SQLiteQuery::new(uri).unwrap()
        })
        .await
    }

    #[tokio::test]
    async fn test_account() {
        let query = setup().await;
        let result = query
            .guid("fcd795021c976ba75621ec39e75f6214")
            .await
            .unwrap();

        let result = &result[0];
        assert_eq!(result.guid(), "fcd795021c976ba75621ec39e75f6214");
        assert_eq!(result.name(), "Asset");
        assert_eq!(result.account_type(), "ASSET");
        assert_eq!(result.commodity_guid(), "346629655191dcf59a7e2c2a85b70f69");
        assert_eq!(result.commodity_scu(), 100);
        assert_eq!(result.non_std_scu(), false);
        assert_eq!(result.parent_guid(), "00622dda21937b29e494179de5013f82");
        assert_eq!(result.code(), "");
        assert_eq!(result.description(), "");
        assert_eq!(result.hidden(), false);
        assert_eq!(result.placeholder(), true);
    }

    #[tokio::test]
    async fn test_all() {
        let query = setup().await;
        let result = query.all().await.unwrap();
        assert_eq!(result.len(), 21);
    }

    #[tokio::test]
    async fn test_guid() {
        let query = setup().await;
        let result = query
            .guid("fcd795021c976ba75621ec39e75f6214")
            .await
            .unwrap();

        assert_eq!(result[0].name, "Asset");
    }

    #[tokio::test]
    async fn test_commodity_guid() {
        let query = setup().await;
        let result = query
            .commodity_guid("346629655191dcf59a7e2c2a85b70f69")
            .await
            .unwrap();
        assert_eq!(result.len(), 14);
    }

    #[tokio::test]
    async fn test_parent_guid() {
        let query = setup().await;
        let result = query
            .parent_guid("fcd795021c976ba75621ec39e75f6214")
            .await
            .unwrap();
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_name() {
        let query = setup().await;
        let result = query.name("Asset").await.unwrap();
        assert_eq!(result[0].guid, "fcd795021c976ba75621ec39e75f6214");
    }

    #[tokio::test]
    async fn test_contains_name_ignore_case() {
        let query = setup().await;
        let result = query.contains_name_ignore_case("AS").await.unwrap();
        assert_eq!(result.len(), 3);
    }
}
