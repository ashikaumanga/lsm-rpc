use std::io::Read;
use std::ops::{Deref, RangeFrom};
use std::ops::Range;

use fjall::{Config, Keyspace, PartitionCreateOptions, PartitionHandle};
use tokio::sync::mpsc;
use tokio_stream::{Stream, wrappers::ReceiverStream};
use tonic::{Request, Response, Status};
use tonic::transport::Server;

use lsmapi::{GetValueResponse, IntResponse, KeyRequest, KeyValueRequest, RangeRequest, RangeResponse};
use lsmapi::lsm_service_server::{LsmService, LsmServiceServer};

//use std::sync::mpsc;

pub mod lsmapi {
    tonic::include_proto!("lsmrpc");
}

//#[derive(Default)]
pub struct LsmServiceImpl {
   partition: PartitionHandle
}


impl LsmServiceImpl {
    fn new() -> Self {
        let keyspace = Config::new("/Users/ashika.umanga/kd5").open().unwrap();
        LsmServiceImpl {
            partition: keyspace.open_partition("my_items", PartitionCreateOptions::default()).unwrap()
        }
    }
}



#[tonic::async_trait]
impl LsmService for LsmServiceImpl  {


    async fn set_value(&self, request: Request<KeyValueRequest>) -> Result<Response<IntResponse>, Status> {

        let keyRef = request.get_ref();
        //request.get_ref()



        let k  = &keyRef.key;
        let v = &keyRef.value;


        match self.partition.insert(k, v) {
            Ok(_) => {
                Ok(Response::new(IntResponse { result : 1}))
            },
            Err(_) => {
                Ok(Response::new(IntResponse { result : 0}))
            }
        }

    }

    async fn get_value(&self, request: Request<KeyRequest>) -> Result<Response<GetValueResponse>, Status> {

        let keyRef = &request.get_ref();
        let key = &keyRef.key;
        let result = self.partition.get(key);


        let empty_res = lsmapi::GetValueResponse { value: Vec::new()};

        match result {
            Ok(val) => {
                match val {
                    None => {
                        Ok(Response::new(empty_res))
                    }
                    Some(value) => {
                        let res = lsmapi::GetValueResponse { value: value.to_vec() };
                        Ok(Response::new(res))
                    }
                }
            },
            Err(_) => {
                let empty_byte_vector: Vec<u8> = Vec::new();
                Ok(Response::new(empty_res))
            }
        }


        //TODO return different status for error
    }

    async fn delete(&self, request: Request<KeyRequest>) -> Result<Response<IntResponse>, Status> {
        let res = lsmapi::IntResponse { result : 200};
        Ok(Response::new(res))
    }

    type RangeStream = ReceiverStream<Result<RangeResponse, Status>>;

    async fn range(&self, request: Request<RangeRequest>) -> Result<Response<Self::RangeStream>, Status> {

        let key_start = &request.get_ref().key_start;
        let key_end = &request.get_ref().key_end;


        let scan_range = match (key_start,key_end)  {
            (Some(start), Some(end)) => {
                Some(start.as_slice()..end.as_slice())
            }
            (Some(start), None) => {
                //only start
                let lk = self.partition.last_key_value()?;
                
                Some(start.as_slice()..lk);
            }
            (None, Some(end)) => {
                //only end
                Some(..end.as_slice())
            }
            (None,None) => {
                None
            }
        };


        let (mut tx,mut rx) = mpsc::channel::<lsmapi::RangeResponse>(4);
        let range_result = self.partition.range(scan_range.unwrap());
        for i in range_result.into_iter() {
            let (k,v) = i.unwrap();

        }

        let v = lsmapi::RangeResponse { key: Vec::new(), value: Vec::new()};
        //tx.send(Ok(v));
        let r = ReceiverStream::new(rx);
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("lms-server started at ");
    let addr = "[::1]:50051".parse()?;
    let lsm_service = LsmServiceImpl::new();

    Server::builder()
        .add_service(LsmServiceServer::new(lsm_service))
        .serve(addr)
        .await?;

    Ok(())

}

fn test() {

    let keyspace = Config::new("/Users/ashika.umanga/kd3").open().unwrap();

    let items = keyspace.open_partition("my_items", PartitionCreateOptions::default()).unwrap();

    /*
    for i in 0..=1000 {
        //let key = "key".to_string() + &i.to_string();
        //let val = "Val".to_string() + &i.to_string();
        //items.insert(key, val).unwrap();
        let tmp = &i;
        let key : i32  = *tmp;
        let kb = key.to_be_bytes();
        let val = "Val".to_string() + &i.to_string();
        items.insert(kb, val).unwrap();
    }
    */
    let k : i32 = 950;
    let kb = k.to_be_bytes();

    let range  = RangeFrom{start: kb};

    let result = items.range(range);
    for i in &result {
        let (x,y) = i.unwrap();
        //let kk = x.
        let v = String::from_utf8_lossy(y.deref());
        println!("{:?}", v);
    }

    /*
    items.insert("t", "hello2").unwrap();
    let bytes = items.get("t").unwrap();

    items.range("1"..);


    let binding  = bytes.unwrap().to_vec();
    let v= binding.as_slice();



    let value =    String::from_utf8_lossy(v);
    println!("Hello, world! {:?}", &value);
    */
}
