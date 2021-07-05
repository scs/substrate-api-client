use substrate_api_client::rpc::json_req::author_submit_extrinsic;
use substrate_api_client::{Api, ApiClientError, ApiResult, Hash, RpcClient};
struct MyClient {
    // pick any request crate, such as ureq::Agent
    inner: (),
}

impl MyClient {
    pub fn new() -> Self {
        Self {
            // ureq::agent()
            inner: (),
        }
    }

    pub fn send_json<R>(&self, path: String, json: Value) -> Result<R> {
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
    let api = Api::<(), _>::new(client);
}
