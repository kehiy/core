use std::error::Error;

use primitives::TransactionsFetchOption;
use storage::DatabaseClient;

pub struct TransactionsClient {
    database: DatabaseClient,
}

impl TransactionsClient {
    pub async fn new(
        database_url: &str
    ) -> Self {
        let database = DatabaseClient::new(database_url);
        Self {
            database,
        }
    }

    pub fn get_transactions_by_device_id(&mut self, device_id: &str, options: TransactionsFetchOption) -> Result<Vec<primitives::Transaction>, Box<dyn Error>> {
        let addresses = self.database.get_subscriptions_by_device_id(device_id)?.into_iter().map(|x| x.address).collect::<Vec<String>>();

        let transactions = self.database
            .get_transactions_by_device_id(device_id, addresses.clone(), options)?
            .into_iter()
            .map(|x| x.as_primitive(addresses.clone()))
            .collect();
        Ok(transactions)
    }

    pub fn get_transactions_by_hash(&mut self, hash: &str) -> Result<Vec<primitives::Transaction>, Box<dyn Error>> {
        let transactions = self.database
            .get_transactions_by_hash(hash)?
            .into_iter()
            .map(|x| x.as_primitive(vec![]))
            .collect();

        Ok(transactions)
    }
}