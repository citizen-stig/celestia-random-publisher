use celestia_rpc::StateClient;
use celestia_types::consts::appconsts;
use celestia_types::nmt::Namespace;
use celestia_types::{AppVersion, Blob};
use jsonrpsee::http_client::HttpClient;

pub(crate) const APP_VERSION: AppVersion = AppVersion::V3;
const DEFAULT_GAS_PER_BLOB_BYTE: usize = 8;
// https://github.com/celestiaorg/celestia-app/blob/a92de7236e7568aa1e9032a29a68c64ef751ce0a/x/blob/types/payforblob.go#L37
const PFB_GAS_FIXED_COST: usize = 75_000;

async fn submit_blob(client: &HttpClient, blob: Vec<u8>, namespace: Namespace) {
    let blob = Blob::new(namespace, blob.to_vec(), APP_VERSION).unwrap();

    let shares_needed = blob.shares_len();
    let gas = shares_needed
        .saturating_mul(appconsts::SHARE_SIZE)
        .saturating_mul(DEFAULT_GAS_PER_BLOB_BYTE)
        .saturating_add(PFB_GAS_FIXED_COST) as u64;
    println!("Gas: {}", gas);

    let mut tx_config = celestia_rpc::TxConfig::default();
    tx_config.with_gas_price(0.002101).with_gas(gas);
    let tx_response = client
        .state_submit_pay_for_blob(&[blob.into()], tx_config)
        .await
        .unwrap();
    println!("{:?}", tx_response);
}

#[tokio::main]
async fn main() {
    let namespace = Namespace::const_v0(*b"\0\0aov-test");
    let blob = vec![1, 2, 3];
    let client =
        { jsonrpsee::http_client::HttpClientBuilder::default().build("http://127.0.0.1:26658") }
            .expect("Client initialization is valid");
    submit_blob(&client, blob, namespace).await;
}
