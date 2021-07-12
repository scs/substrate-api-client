use substrate_api_client::rpc::json_req::author_submit_extrinsic;
use substrate_api_client::{
    Api, ApiClientError, ApiResult, FromHexString, Hash, RpcClient, Value, XtStatus,
};
struct MyClient {
    // pick any request crate, such as ureq::Agent
    _inner: (),
}

impl MyClient {
    pub fn new() -> Self {
        Self {
            // ureq::agent()
            _inner: (),
        }
    }

    pub fn send_json<R>(
        &self,
        _path: String,
        _json: Value,
    ) -> Result<R, Box<dyn std::error::Error>> {
        // you can figure this out...self.inner...send_json...
        todo!()
    }
}

impl RpcClient for MyClient {
    fn get_request(&self, jsonreq: serde_json::Value) -> ApiResult<String> {
        self.send_json::<Value>("".into(), jsonreq)
            .map(|v| v.to_string())
            .map_err(|err| ApiClientError::RpcClient(err.to_string()))
    }

    fn send_extrinsic(
        &self,
        xthex_prefixed: String,
        _exit_on: XtStatus,
    ) -> ApiResult<Option<Hash>> {
        let jsonreq = author_submit_extrinsic(&xthex_prefixed);
        let res: String = self
            .send_json("".into(), jsonreq)
            .map_err(|err| ApiClientError::RpcClient(err.to_string()))?;
        Ok(Some(Hash::from_hex(res)?))
    }
}

fn main() {
    let client = MyClient::new();
    let _api = Api::<(), _>::new(client);
}
