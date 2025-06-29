// ref: https://piecash.readthedocs.io/en/master/object_model.html
// ref: https://wiki.gnucash.org/wiki/SQL

use chrono::NaiveDateTime;
use rusqlite::Row;
#[cfg(feature = "decimal")]
use rust_decimal::Decimal;

use super::SQLiteQuery;
use crate::error::Error;
use crate::query::{PriceQ, PriceT};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Hash)]
pub struct Price {
    pub guid: String,
    pub commodity_guid: String,
    pub currency_guid: String,
    pub date: NaiveDateTime,
    pub source: Option<String>,
    pub r#type: Option<String>,
    pub value_num: i64,
    pub value_denom: i64,
}

impl<'a> TryFrom<&'a Row<'a>> for Price {
    type Error = rusqlite::Error;

    fn try_from(row: &'a Row<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            guid: row.get(0)?,
            commodity_guid: row.get(1)?,
            currency_guid: row.get(2)?,
            date: row.get(3)?,
            source: row.get(4)?,
            r#type: row.get(5)?,
            value_num: row.get(6)?,
            value_denom: row.get(7)?,
        })
    }
}

impl PriceT for Price {
    fn guid(&self) -> String {
        self.guid.clone()
    }
    fn commodity_guid(&self) -> String {
        self.commodity_guid.clone()
    }
    fn currency_guid(&self) -> String {
        self.currency_guid.clone()
    }
    fn datetime(&self) -> NaiveDateTime {
        self.date
    }
    fn source(&self) -> String {
        self.source.clone().unwrap_or_default()
    }
    fn r#type(&self) -> String {
        self.r#type.clone().unwrap_or_default()
    }

    #[cfg(not(feature = "decimal"))]
    #[allow(clippy::cast_precision_loss)]
    fn value(&self) -> f64 {
        self.value_num as f64 / self.value_denom as f64
    }

    #[cfg(feature = "decimal")]
    fn value(&self) -> Decimal {
        Decimal::new(self.value_num, 0) / Decimal::new(self.value_denom, 0)
    }
}

const SEL: &str = r"
SELECT
guid,
commodity_guid,
currency_guid,
date,
source,
type,
value_num,
value_denom
FROM prices
";

impl PriceQ for SQLiteQuery {
    type P = Price;

    async fn all(&self) -> Result<Vec<Self::P>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(SEL)?;
        let result = stmt
            .query([])?
            .mapped(|row| Price::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }
    async fn guid(&self, guid: &str) -> Result<Vec<Self::P>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!("{SEL}\nWHERE guid = ?"))?;
        let result = stmt
            .query([guid])?
            .mapped(|row| Price::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }
    async fn commodity_guid(&self, guid: &str) -> Result<Vec<Self::P>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!("{SEL}\nWHERE commodity_guid = ?"))?;
        let result = stmt
            .query([guid])?
            .mapped(|row| Price::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }
    async fn currency_guid(&self, guid: &str) -> Result<Vec<Self::P>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!("{SEL}\nWHERE currency_guid = ?"))?;
        let result = stmt
            .query([guid])?
            .mapped(|row| Price::try_from(row))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(result)
    }
    async fn commodity_or_currency_guid(&self, guid: &str) -> Result<Vec<Self::P>, Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&format!(
            "{SEL}\nWHERE commodity_guid = ? OR currency_guid = ?"
        ))?;
        let result = stmt
            .query([guid, guid])?
            .mapped(|row| Price::try_from(row))
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
    fn test_price_schemas() {
        let _ = sqlx::query_as!(
            Price,
            r#"
				SELECT
				guid,
				commodity_guid,
				currency_guid,
				date as "date: NaiveDateTime",
				source,
				type,
				value_num,
				value_denom
				FROM prices
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
    async fn test_price() {
        let query = setup().await;
        let result = query
            .guid("0d6684f44fb018e882de76094ed9c433")
            .await
            .unwrap();

        let result = &result[0];
        assert_eq!(result.guid(), "0d6684f44fb018e882de76094ed9c433");
        assert_eq!(result.commodity_guid(), "d821d6776fde9f7c2d01b67876406fd3");
        assert_eq!(result.currency_guid(), "5f586908098232e67edb1371408bfaa8");
        assert_eq!(
            result.datetime(),
            NaiveDateTime::parse_from_str("2018-02-20 23:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );
        assert_eq!(result.source(), "user:price-editor");
        assert_eq!(result.r#type(), "unknown");
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result.value(), 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(result.value(), Decimal::new(15, 1));
    }

    #[tokio::test]
    async fn test_all() {
        let query = setup().await;
        let result = query.all().await.unwrap();
        assert_eq!(result.len(), 5);
    }

    #[tokio::test]
    async fn test_guid() {
        let query = setup().await;
        let result = query
            .guid("0d6684f44fb018e882de76094ed9c433")
            .await
            .unwrap();

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result[0].value(), 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(result[0].value(), Decimal::new(15, 1));
    }

    #[tokio::test]
    async fn commodity_guid() {
        let query = setup().await;
        let result = query
            .commodity_guid("d821d6776fde9f7c2d01b67876406fd3")
            .await
            .unwrap();

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result[0].value(), 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(result[0].value(), Decimal::new(15, 1));
    }

    #[tokio::test]
    async fn currency_guid() {
        let query = setup().await;
        let result = query
            .currency_guid("5f586908098232e67edb1371408bfaa8")
            .await
            .unwrap();

        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result[0].value(), 1.5);
        #[cfg(feature = "decimal")]
        assert_eq!(result[0].value(), Decimal::new(15, 1));
    }

    #[tokio::test]
    async fn commodity_or_currency_guid() {
        let query = setup().await;
        let result = query
            .commodity_or_currency_guid("5f586908098232e67edb1371408bfaa8")
            .await
            .unwrap();
        assert_eq!(result.len(), 4);
    }
}
