use celestia_rpc::StateClient;
use celestia_types::blob::RawBlob;
use celestia_types::consts::appconsts;
use celestia_types::nmt::Namespace;
use celestia_types::{AppVersion, Blob};
use jsonrpsee::http_client::HttpClient;
use rand::{Rng, RngCore};

pub(crate) const APP_VERSION: AppVersion = AppVersion::V3;
const DEFAULT_GAS_PER_BLOB_BYTE: usize = 8;
// https://github.com/celestiaorg/celestia-app/blob/a92de7236e7568aa1e9032a29a68c64ef751ce0a/x/blob/types/payforblob.go#L37
const PFB_GAS_FIXED_COST: usize = 75_000;
const GAS_PRICE: f64 = 0.002101;

const NAMESPACE_PRECEDING_1: Namespace = Namespace::const_v0(*b"\0\0aaa-test");
const NAMESPACE_PRECEDING_2: Namespace = Namespace::const_v0(*b"\0\0bbb-test");
const NAMESPACE_PRECEDING_SAME: Namespace = Namespace::const_v0(*b"\0\0sov-test");

const NAMESPACES: [Namespace; 3] = [
    NAMESPACE_PRECEDING_1,
    NAMESPACE_PRECEDING_2,
    NAMESPACE_PRECEDING_SAME,
];

async fn submit_blobs(client: &HttpClient, blobs: Vec<Vec<u8>>, namespace: Namespace) {
    let mut shares_needed = 0;
    let mut raw_blobs = Vec::with_capacity(blobs.len());

    for blob in blobs {
        let cel_blob = Blob::new(namespace, blob.to_vec(), APP_VERSION).unwrap();
        shares_needed += cel_blob.shares_len();
        raw_blobs.push(RawBlob::from(cel_blob));
    }

    let gas = shares_needed
        .saturating_mul(appconsts::SHARE_SIZE)
        .saturating_mul(DEFAULT_GAS_PER_BLOB_BYTE)
        .saturating_add(PFB_GAS_FIXED_COST) as u64;
    println!("Gas: {}", gas);

    let mut tx_config = celestia_rpc::TxConfig::default();
    tx_config.with_gas_price(GAS_PRICE).with_gas(gas);
    let tx_response = client
        .state_submit_pay_for_blob(&raw_blobs, tx_config)
        .await
        .unwrap();
    println!("{:?}", tx_response);
}

#[tokio::main]
async fn main() {
    // let blob = vec![1, 2, 3];

    // submit_blob(&client, blob, NAMESPACE_PRECEDING_1).await;

    let mut rng = rand::rng();
    let client = jsonrpsee::http_client::HttpClientBuilder::default()
        .build("http://127.0.0.1:26658")
        .expect("Client initialization is valid");
    let sizes = [100, 512, 1024];

    loop {
        // TODO: Handle zero blobs and sleep for the next block head.
        let num_blobs = rng.random_range(1..=3);
        let mut blobs = Vec::with_capacity(num_blobs);
        for _ in 0..num_blobs {
            let size = sizes[rng.random_range(0..sizes.len())];
            let mut blob = vec![0u8; size];
            rng.fill_bytes(&mut blob);
            blobs.push(blob);
        }
        let namespace = NAMESPACES[rng.random_range(0..NAMESPACES.len())];
        println!(
            "GOING TO SUBMIT {} BLOBS TO NS {}",
            blobs.len(),
            String::from_utf8_lossy(&namespace.0)
        );
        submit_blobs(&client, blobs, namespace).await;
    }
}
