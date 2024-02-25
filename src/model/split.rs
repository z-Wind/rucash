use chrono::NaiveDateTime;
use std::sync::Arc;

use crate::error::Error;
use crate::model::{Account, Transaction};
use crate::query::{AccountQ, Query, SplitT, TransactionQ};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Split<Q>
where
    Q: Query,
{
    query: Arc<Q>,

    pub guid: String,
    pub tx_guid: String,
    pub account_guid: String,
    pub memo: String,
    pub action: String,
    pub reconcile_state: bool,
    pub reconcile_datetime: Option<NaiveDateTime>,
    pub value: crate::Num,
    pub quantity: crate::Num,
    pub lot_guid: String,
}

impl<Q> Split<Q>
where
    Q: Query,
{
    pub(crate) fn from_with_query<T: SplitT>(item: &T, query: Arc<Q>) -> Self {
        Self {
            query,

            guid: item.guid(),
            tx_guid: item.tx_guid(),
            account_guid: item.account_guid(),
            memo: item.memo(),
            action: item.action(),
            reconcile_state: item.reconcile_state(),
            reconcile_datetime: item.reconcile_datetime(),
            lot_guid: item.lot_guid(),
            value: item.value(),
            quantity: item.quantity(),
        }
    }

    pub async fn transaction(&self) -> Result<Transaction<Q>, Error> {
        if self.tx_guid.is_empty() {
            return Err(Error::GuidNotFound {
                model: "Transaction".to_string(),
                guid: self.tx_guid.clone(),
            });
        };

        let mut transactions = TransactionQ::guid(&*self.query, &self.tx_guid).await?;

        match transactions.pop() {
            None => Err(Error::GuidNotFound {
                model: "Transaction".to_string(),
                guid: self.tx_guid.clone(),
            }),
            Some(x) if transactions.is_empty() => {
                Ok(Transaction::from_with_query(&x, self.query.clone()))
            }
            _ => Err(Error::GuidMultipleFound {
                model: "Transaction".to_string(),
                guid: self.tx_guid.clone(),
            }),
        }
    }

    pub async fn account(&self) -> Result<Account<Q>, Error> {
        if self.account_guid.is_empty() {
            return Err(Error::GuidNotFound {
                model: "Account".to_string(),
                guid: self.account_guid.clone(),
            });
        };

        let mut accounts = AccountQ::guid(&*self.query, &self.account_guid).await?;

        match accounts.pop() {
            None => Err(Error::GuidNotFound {
                model: "Account".to_string(),
                guid: self.account_guid.clone(),
            }),
            Some(x) if accounts.is_empty() => Ok(Account::from_with_query(&x, self.query.clone())),
            _ => Err(Error::GuidMultipleFound {
                model: "Account".to_string(),
                guid: self.account_guid.clone(),
            }),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "sqlite")]
mod tests {
    use super::*;

    use crate::query::sqlite::split::Split as SplitSQL;
    use crate::Book;
    use crate::SQLiteQuery;

    #[cfg(not(feature = "decimal"))]
    use float_cmp::assert_approx_eq;
    use pretty_assertions::assert_eq;
    #[cfg(feature = "decimal")]
    use rust_decimal::Decimal;

    async fn setup() -> SQLiteQuery {
        let uri: &str = &format!(
            "sqlite://{}/tests/db/sqlite/complex_sample.gnucash",
            env!("CARGO_MANIFEST_DIR")
        );

        println!("work_dir: {:?}", std::env::current_dir());
        SQLiteQuery::new(&format!("{uri}?mode=ro")).await.unwrap()
    }

    #[tokio::test]
    async fn test_from_with_query() {
        let query = Arc::new(setup().await);
        let item = SplitSQL {
            guid: "guid".to_string(),
            tx_guid: "commodity_guid".to_string(),
            account_guid: "currency_guid".to_string(),
            memo: "currency_guid".to_string(),
            action: "currency_guid".to_string(),
            reconcile_state: "currency_guid".to_string(),
            reconcile_date: NaiveDateTime::parse_from_str(
                "2014-12-24 10:59:00",
                "%Y-%m-%d %H:%M:%S",
            )
            .ok(),
            lot_guid: Some("source".to_string()),

            value_num: 1000,
            value_denom: 10,
            quantity_num: 1100,
            quantity_denom: 10,
        };

        let result = Split::from_with_query(&item, query);

        assert_eq!(result.guid, "guid");
        assert_eq!(result.tx_guid, "commodity_guid");
        assert_eq!(result.account_guid, "currency_guid");
        assert_eq!(result.memo, "currency_guid");
        assert_eq!(result.action, "currency_guid");
        assert_eq!(result.reconcile_state, false);
        assert_eq!(
            result.reconcile_datetime,
            NaiveDateTime::parse_from_str("2014-12-24 10:59:00", "%Y-%m-%d %H:%M:%S").ok()
        );
        assert_eq!(result.lot_guid, "source");
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result.value, 100.0);
        #[cfg(feature = "decimal")]
        assert_eq!(result.value, Decimal::new(100, 0));
        #[cfg(not(feature = "decimal"))]
        assert_approx_eq!(f64, result.quantity, 110.0);
        #[cfg(feature = "decimal")]
        assert_eq!(result.quantity, Decimal::new(110, 0));
    }

    #[tokio::test]
    async fn transaction() {
        let query = setup().await;
        let book = Book::new(query).await.unwrap();
        let split = book
            .splits()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "de832fe97e37811a7fff7e28b3a43425")
            .unwrap();
        let transaction = split.transaction().await.unwrap();
        assert_eq!(transaction.description, "income 1");
    }

    #[tokio::test]
    async fn account() {
        let query = setup().await;
        let book = Book::new(query).await.unwrap();
        let split = book
            .splits()
            .await
            .unwrap()
            .into_iter()
            .find(|x| x.guid == "de832fe97e37811a7fff7e28b3a43425")
            .unwrap();
        let account = split.account().await.unwrap();
        assert_eq!(account.name, "Cash");
    }
}
