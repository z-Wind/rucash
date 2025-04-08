// ref: https://piecash.readthedocs.io/en/master/object_model.html
// ref: https://wiki.gnucash.org/wiki/SQL

use chrono::NaiveDateTime;
use rusqlite::Row;
#[cfg(feature = "decimal")]
use rust_decimal::Decimal;

use super::SQLiteQuery;
use crate::error::Error;
use crate::query::{SplitQ, SplitT};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct Split {
    pub guid: String,
    pub tx_guid: String,
    pub account_guid: String,
    pub memo: String,
    pub action: String,
    pub reconcile_state: String,
    pub reconcile_date: Option<NaiveDateTime>,
    pub value_num: i64,
    pub value_denom: i64,
    pub quantity_num: i64,
    pub quantity_denom: i64,
    pub lot_guid: Option<String>,
}

impl<'a> TryFrom<&'a Row<'a>> for Split {
    type Error = rusqlite::Error;

    fn try_from(row: &'a Row<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            guid: row.get(0)?,
            tx_guid: row.get(1)?,
            account_guid: row.get(2)?,
            memo: row.get(3)?,
            action: row.get(4)?,
            reconcile_state: row.get(5)?,
            reconcile_date: row.get(6)?,
            value_num: row.get(7)?,
            value_denom: row.get(8)?,
            quantity_num: row.get(9)?,
            quantity_denom: row.get(10)?,
            lot_guid: row.get(11)?,
        })
    }
}

impl SplitT for Split {
    fn guid(&self) -> String {
        self.guid.clone()
    }
    fn tx_guid(&self) -> String {
        self.tx_guid.clone()
    }
    fn account_guid(&self) -> String {
        self.account_guid.clone()
    }
    fn memo(&self) -> String {
        self.memo.clone()
    }
    fn action(&self) -> String {
        self.action.clone()
    }
    fn reconcile_state(&self) -> bool {
        self.reconcile_state == "y" || self.reconcile_state == "Y"
    }
    fn reconcile_datetime(&self) -> Option<NaiveDateTime> {
        let datetime = self.reconcile_date?;
        if datetime == NaiveDateTime::UNIX_EPOCH {
            return None;
        }
        Some(datetime)
    }
    fn lot_guid(&self) -> String {
        self.lot_guid.clone().unwrap_or_default()
    }

    #[cfg(not(feature = "decimal"))]
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    fn value(&self) -> f64 {
        self.value_num as f64 / self.value_denom as f64
    }

    #[cfg(feature = "decimal")]
    #[must_use]
    fn value(&self) -> Decimal {
        Decimal::new(self.value_num, 0) / Decimal::new(self.value_denom, 0)
    }

    #[cfg(not(feature = "decimal"))]
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    fn quantity(&self) -> f64 {
        self.quantity_num as f64 / self.quantity_denom as f64
    }

    #[cfg(feature = "decimal")]
    #[must_use]
    fn quantity(&self) -> Decimal {
        Decimal::new(self.quantity_num, 0) / Decimal::new(self.quantity_denom, 0)
    }
}

const SEL: &str = r"
SELECT
guid,
tx_guid,
account_guid,
memo,
action,
reconcile_state,
reconcile_date,
value_num,
value_denom,
quantity_num,
quantity_denom,
lot_guid
FROM splits
";

impl SplitQ for SQLiteQuery {
    type S = Split;

    async fn all(&self) -> Result<Vec<Self::S>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(SEL)?;
        let result = stmt
            .query([])?
            .mapped(|row| Split::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    async fn guid(&self, guid: &str) -> Result<Vec<Self::S>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!("{SEL}\nWHERE guid = ?"))?;
        let result = stmt
            .query([guid])?
            .mapped(|row| Split::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    async fn account_guid(&self, guid: &str) -> Result<Vec<Self::S>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!("{SEL}\nWHERE account_guid = ?"))?;
        let result = stmt
            .query([guid])?
            .mapped(|row| Split::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }

    async fn tx_guid(&self, guid: &str) -> Result<Vec<Self::S>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!("{SEL}\nWHERE tx_guid = ?"))?;
        let result = stmt
            .query([guid])?
            .mapped(|row| Split::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    use pretty_assertions::assert_eq;
    use tokio::sync::OnceCell;

    #[cfg(feature = "schema")]
    // test schemas on compile time
    #[allow(dead_code)]
    fn test_split_schemas() {
        let _ = sqlx::query_as!(
            Split,
            r#"
				SELECT 	
				guid,
				tx_guid,
				account_guid,
				memo,
				action,
				reconcile_state,
				reconcile_date as "reconcile_date: NaiveDateTime",
				value_num,
				value_denom,
				quantity_num,
				quantity_denom,
				lot_guid
				FROM splits
				"#,
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
    async fn test_split() {
        let query = setup().await;
        let result = query
            .guid("de832fe97e37811a7fff7e28b3a43425")
            .await
            .unwrap();

        let result = &result[0];
        assert_eq!(result.guid(), "de832fe97e37811a7fff7e28b3a43425");
        assert_eq!(result.tx_guid(), "6c8876003c4a6026e38e3afb67d6f2b1");
        assert_eq!(result.account_guid(), "93fc043c3062aaa1297b30e543d2cd0d");
        assert_eq!(result.memo(), "");
        assert_eq!(result.action(), "");
        assert_eq!(result.reconcile_state(), false);
        assert_eq!(result.reconcile_datetime(), None);
        assert_eq!(result.lot_guid(), "");
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result.value(), 150.0);
        #[cfg(feature = "decimal")]
        assert_eq!(result.value(), Decimal::new(150, 0));
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result.quantity(), 150.0);
        #[cfg(feature = "decimal")]
        assert_eq!(result.quantity(), Decimal::new(150, 0));
    }

    #[tokio::test]
    async fn test_all() {
        let query = setup().await;
        let result = query.all().await.unwrap();
        assert_eq!(result.len(), 25);
    }

    #[tokio::test]
    async fn test_guid() {
        let query = setup().await;
        let result = query
            .guid("de832fe97e37811a7fff7e28b3a43425")
            .await
            .unwrap();

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result[0].value(), 150.0);
        #[cfg(feature = "decimal")]
        assert_eq!(result[0].value(), Decimal::new(150, 0));
    }

    #[tokio::test]
    async fn test_account_guid() {
        let query = setup().await;
        let result = query
            .account_guid("93fc043c3062aaa1297b30e543d2cd0d")
            .await
            .unwrap();
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_tx_guid() {
        let query = setup().await;
        let result = query
            .tx_guid("6c8876003c4a6026e38e3afb67d6f2b1")
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
    }
}
