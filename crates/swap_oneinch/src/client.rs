use std::str::FromStr;

use gem_evm::address::EthereumAddress;
use primitives::{AssetId, Chain, ChainType, SwapQuote, SwapQuoteProtocolRequest};

use super::model::{QuoteRequest, SwapResult, Tokenlist};

pub struct OneInchClient {
    api_url: String,
    api_key: String,
    fee: f64,
    fee_referral_address: String,
    version: String,
    client: reqwest::Client,
}

const NATIVE_ADDRESS: &str = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
pub const PROVIDER_NAME: &str = "1inch";

impl OneInchClient {
    pub fn new(api_url: String, api_key: String, fee: f64, fee_referral_address: String) -> Self {
        let client = reqwest::Client::builder().build().unwrap();

        Self {
            client,
            api_url,
            api_key,
            fee,
            fee_referral_address,
            version: "v5.2".to_string(),
        }
    }

    pub fn chains(&self) -> Vec<Chain> {
        vec![
            Chain::Ethereum,
            Chain::Arbitrum,
            Chain::Optimism,
            Chain::Polygon,
            Chain::SmartChain,
            Chain::AvalancheC,
            Chain::Base,
            Chain::Fantom,
            Chain::Gnosis,
            Chain::ZkSync,
        ]
    }

    pub async fn get_tokenlist(
        &self,
        chain_id: &str,
    ) -> Result<Tokenlist, Box<dyn std::error::Error>> {
        let url = format!("{}/token/v1.2/{chain_id}", self.api_url);
        Ok(self
            .client
            .get(&url)
            .bearer_auth(self.api_key.as_str())
            .send()
            .await?
            .json::<Tokenlist>()
            .await?)
    }

    pub fn get_asset_ids_for_tokenlist(&self, chain: Chain, tokenlist: Tokenlist) -> Vec<AssetId> {
        tokenlist
            .into_iter()
            .flat_map(|x| match EthereumAddress::from_str(&x.0) {
                Ok(token_id) => Some(AssetId {
                    chain,
                    token_id: Some(token_id.to_checksum()),
                }),
                Err(_) => None,
            })
            .collect::<Vec<AssetId>>()
    }

    pub async fn get_quote(
        &self,
        quote: SwapQuoteProtocolRequest,
    ) -> Result<SwapQuote, Box<dyn std::error::Error + Send + Sync>> {
        let network_id = quote.from_asset.chain.network_id();
        let src = if quote.from_asset.clone().is_native() {
            NATIVE_ADDRESS.to_string()
        } else {
            quote.from_asset.clone().token_id.unwrap()
        };
        let dst = if quote.to_asset.clone().is_native() {
            NATIVE_ADDRESS.to_string()
        } else {
            quote.to_asset.clone().token_id.unwrap()
        };
        let quote_request = QuoteRequest {
            src: src.clone(),
            dst,
            from: quote.wallet_address.clone(),
            amount: quote.amount.clone(),
            slippage: 1.0,
            disable_estimate: false,
            fee: self.fee,
            referrer: self.fee_referral_address.clone(),
        };

        let swap_quote = if quote.include_data {
            self.get_swap_quote_data(quote_request, network_id).await?
        } else {
            self.get_swap_quote(quote_request, network_id).await?
        };
        let data = swap_quote.tx.map(|value| value.get_data());

        let quote = SwapQuote {
            chain_type: ChainType::Ethereum,
            from_amount: quote.amount.clone(),
            to_amount: swap_quote.to_amount,
            fee_percent: self.fee as f32,
            provider: PROVIDER_NAME.into(),
            data,
        };
        Ok(quote)
    }

    pub async fn get_swap_quote(
        &self,
        request: QuoteRequest,
        network_id: &str,
    ) -> Result<SwapResult, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/swap/{}/{}/quote",
            self.api_url, self.version, network_id
        );
        Ok(self
            .client
            .get(&url)
            .query(&request)
            .bearer_auth(self.api_key.as_str())
            .send()
            .await?
            .json::<SwapResult>()
            .await?)
    }

    pub async fn get_swap_quote_data(
        &self,
        request: QuoteRequest,
        network_id: &str,
    ) -> Result<SwapResult, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/swap/{}/{}/swap", self.api_url, self.version, network_id);
        Ok(self
            .client
            .get(&url)
            .query(&request)
            .bearer_auth(self.api_key.as_str())
            .send()
            .await?
            .json::<SwapResult>()
            .await?)
    }
}
