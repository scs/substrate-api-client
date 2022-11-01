
use sp_core::{Decode, Encode};
use sp_keyring::AccountKeyring;
use sp_runtime::AccountId32;
use sp_core::{sr25519, Pair};

use sp_runtime::app_crypto::Ss58Codec;
use substrate_api_client::rpc::WsRpcClient;
use substrate_api_client::{
    compose_extrinsic_offline, AccountId, Api, PlainTipExtrinsicParams, UncheckedExtrinsicV4,
    XtStatus,
};

fn main() {
    env_logger::init();
    
    let url = get_node_url_from_cli();

    let from = sp_keyring::AccountKeyring::Alice.pair();

    let from = AccountKeyring::Alice.pair();
    let client = WsRpcClient::new(&url);
    let api = Api::<_, _, PlainTipExtrinsicParams>::new(client)
        .map(|api| api.set_signer(from))
        .unwrap();

    let mut exposure: Exposure<AccountId32, u128> = Exposure {
        total: 0,
        own: 0,
        others: vec![],
    };

    let account: AccountId;
    match AccountId32::from_ss58check("5DJcEbkNxsnNwHGrseg7cgbfUG8eiKzpuZqgSph5HqHrjgf6") {
        Ok(address) => account = address,
        Err(e) => panic!("Invalid Account id : {:?}", e),
    }
    // initialize api and set the signer (sender) that is used to sign the extrinsics
    let pair = sr25519::Pair::from_string(/* Give Your Secret Key for test, dont upload in repo */"", None).unwrap();
    let api = api.set_signer(pair.clone());

    let active_era: Staking = api
        .get_storage_value("Staking", "ActiveEra", None)
        .unwrap()
        .unwrap();
    println!("{:?}", active_era);
    let idx = active_era.index - 1;
    match api
        .get_storage_double_map("Staking", "ErasStakers", idx, &account, None)
        .unwrap()
    {
        Some(exp) => {
            exposure = exp;
        }
        None => (),
    }
    if exposure.total > 0_u128 {
        let call = api.payout_stakers(idx, account.clone());
        let result = api
            .send_extrinsic(call.hex_encode(), XtStatus::InBlock)
            .unwrap();
        println!("{:?}", result);
    }
}

pub fn get_node_url_from_cli() -> String {
    let url = format!("wss://westend-rpc.polkadot.io:443"); //wss://rpc.polkadot.io:443
    println!("Interacting with node on {}\n", url);
    url
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug)]
pub struct Staking {
    index: u32,
    start: Option<u32>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, Default)]
pub struct Exposure<_0, _1> {
    #[codec(compact)]
    pub total: _1,
    #[codec(compact)]
    pub own: _1,
    pub others: Vec<IndividualExposure<_0, _1>>,
}
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, Debug, Default)]
pub struct IndividualExposure<_0, _1> {
    pub who: _0,
    #[codec(compact)]
    pub value: _1,
}
