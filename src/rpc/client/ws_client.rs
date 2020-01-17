use ws::connect;

use ws::{CloseCode,Handler, Result,Message,Sender,Handshake};
use std::thread;
use super::{ResultE,OnMessageFn};
use futures_signals::signal::Mutable;

pub struct RpcClient {
    pub out: Sender,
    pub request: String,
    pub result: Mutable<String>,
    pub on_message_fn: OnMessageFn,
}
impl Handler for RpcClient {
  fn on_open(&mut self,_: Handshake ) -> Result<()> {
      self.out.send(self.request.clone()).unwrap();
      Ok(())
  }

  fn on_message(&mut self, msg: Message) -> Result<()> {
      let msgg = msg.as_text().unwrap();
      let res_e = (self.on_message_fn)(&msgg);
      match res_e {
        ResultE::None=>{},
        ResultE::Close=>{
          self.out.close(CloseCode::Normal).unwrap();
        },
        ResultE::S(s)=>{
          self.result.set(s);
        },
        ResultE::SClose(s)=>{
          self.result.set(s);
          self.out.close(CloseCode::Normal).unwrap();
        }
      }
      Ok(())
  }
}


pub fn start_rpc_client_thread(
  url: String,
  jsonreq: String,
  result_in: Mutable<String>,
  on_message_fn: OnMessageFn,
) {
  let _client = thread::Builder::new()
      .name("client".to_owned())
      .spawn(move || {
          connect(url, |out| RpcClient {
              out,
              request: jsonreq.clone(),
              result: result_in.clone(),
              on_message_fn,
          })
          .unwrap()
      })
      .unwrap();
}
