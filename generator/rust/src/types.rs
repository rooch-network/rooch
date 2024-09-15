use heapless::{String, Vec, LinearMap};
use minicbor::{Decode, Encode, Decoder, Encoder, decode::Error };
use super::constants::{MAX_STRING_LEN, MAX_CONTENT_SIZE, MAX_DEPLOY_ARGS };

const MAX_ARGS: usize = 8;

pub struct DeployArgs {
    pub args: Vec<DeployArg, MAX_ARGS>,
}

pub struct DeployArg {
    pub name: String<MAX_STRING_LEN>,
    pub arg: Arg,
}

pub struct Arg {
    pub type_name: String<MAX_STRING_LEN>,
    pub data: ArgData,
}

pub struct ArgData {
    pub min: u64,
    pub max: u64,
}

impl<'a, C> Decode<'a, C> for DeployArgs {
  fn decode(d: &mut Decoder<'a>, _ctx: &mut C) -> Result<Self, Error> {
      let mut args = Vec::new();

      let len = d.array()?.unwrap_or(0);

      for _ in 0..len {
          let arg = DeployArg::decode(d, _ctx)?;
          args.push(arg).map_err(|_| Error::message("Too many arguments"))?;
      }

      Ok(DeployArgs { args })
  }
}

impl<'a, C> Decode<'a, C> for DeployArg {
  fn decode(d: &mut Decoder<'a>, _ctx: &mut C) -> Result<Self, Error> {
      let mut name = String::new();
      let mut arg = Arg {
          type_name: String::new(),
          data: ArgData { min: 0, max: 0 },
      };

      let len = d.map()?.unwrap_or(0);

      for _ in 0..len {
          let key = String::<MAX_STRING_LEN>::try_from(d.str()?).map_err(|_| Error::message("Invalid string length"))?;

          match key.as_str() {
              _ => {
                  name = key;
                  arg = Arg::decode(d, _ctx)?;
              }
          }
      }

      Ok(DeployArg { name, arg })
  }
}

impl<'a, C> Decode<'a, C> for Arg {
  fn decode(d: &mut Decoder<'a>, _ctx: &mut C) -> Result<Self, Error> {
      let mut type_name = String::new();
      let mut data = ArgData { min: 0, max: 0 };

      let len = d.map()?.unwrap_or(0);

      for _ in 0..len {
          let key = String::<MAX_STRING_LEN>::try_from(d.str()?).map_err(|_| Error::message("Invalid string length"))?;

          match key.as_str() {
              "type" => type_name = String::<MAX_STRING_LEN>::try_from(d.str()?).map_err(|_| Error::message("Invalid string length"))?,
              "data" => data = ArgData::decode(d, _ctx)?,
              _ => {
                  d.skip()?;
              }
          }
      }

      Ok(Arg { type_name, data })
  }
}

impl<'a, C> Decode<'a, C> for ArgData {
  fn decode(d: &mut Decoder<'a>, _ctx: &mut C) -> Result<Self, Error> {
      let mut min = 0;
      let mut max = 0;

      let len = d.map()?.unwrap_or(0);

      for _ in 0..len {
          let key = String::<MAX_STRING_LEN>::try_from(d.str()?).map_err(|_| Error::message("Invalid string length"))?;

          match key.as_str() {
              "min" => min = d.u64()?,
              "max" => max = d.u64()?,
              _ => {
                  d.skip()?;
              }
          }
      }

      Ok(ArgData { min, max })
  }
}

pub struct InputData {
  pub deploy_args: Vec<u8, MAX_CONTENT_SIZE>,
  pub seed: String<MAX_STRING_LEN>,
  pub user_input: String<MAX_STRING_LEN>,
}

impl<'a, C> Decode<'a, C> for InputData {
  fn decode(d: &mut Decoder<'a>, _ctx: &mut C) -> Result<Self, Error> {
      let mut deploy_args = Vec::new();
      let mut seed = String::new();
      let mut user_input = String::new();

      let len = d.map()?.unwrap_or(0);

      for _ in 0..len {
          let key = String::<MAX_STRING_LEN>::try_from(d.str()?).map_err(|_| Error::message("Invalid string length"))?;

          match key.as_str() {
              "attrs" => {
                  let array_len = d.array()?.unwrap_or(0);
                  for _ in 0..array_len {
                      let b = d.u32()?;
                      deploy_args.push(b as u8).map_err(|_| Error::message("Deploy arguments too large"))?;
                  }
              }
              "seed" => seed = String::<MAX_STRING_LEN>::try_from(d.str()?).map_err(|_| Error::message("Invalid string length"))?,
              "user_input" => user_input = String::<MAX_STRING_LEN>::try_from(d.str()?).map_err(|_| Error::message("Invalid string length"))?,
              _ => {
                  d.skip()?;
              }
          }
      }

      Ok(InputData {
          deploy_args,
          seed,
          user_input,
      })
  }
}

pub struct Content {
    pub content_type: String<MAX_STRING_LEN>,
    pub body: Vec<u8, MAX_CONTENT_SIZE>,
}

impl<C> Encode<C> for Content {
  fn encode<W: minicbor::encode::Write>(
      &self,
      e: &mut Encoder<W>,
      _ctx: &mut C,
  ) -> Result<(), minicbor::encode::Error<W::Error>> {
      e.map(2)?;

      e.str("content_type")?;
      e.str(&self.content_type)?;

      e.str("body")?;
      e.bytes(&self.body)?;

      Ok(())
  }
}

pub enum Value {
  Null,
  Bool(bool),
  Int(i64),
  UInt(u64),
  String(String<MAX_STRING_LEN>),
}

pub struct OutputData {
  pub amount: u64,
  pub attributes: Option<LinearMap<String<MAX_STRING_LEN>, Value, MAX_DEPLOY_ARGS>>,
  pub content: Option<Content>,
}

impl<'a, C> Encode<C> for OutputData {
  fn encode<W: minicbor::encode::Write>(
      &self,
      e: &mut Encoder<W>,
      _ctx: &mut C,
  ) -> Result<(), minicbor::encode::Error<W::Error>> {
      e.map(3)?;

      e.str("amount")?;
      e.u64(self.amount)?;

      e.str("attributes")?;
      if let Some(ref attributes) = self.attributes {
          e.map(attributes.len() as u64)?;
          for (key, value) in attributes {
              e.str(key)?;

              match value {
                Value::Null => e.null()?,
                Value::Bool(v) => e.bool(*v)?,
                Value::Int(v) => e.i64(*v)?,
                Value::UInt(v) => e.u64(*v)?,
                Value::String(v) => e.str(v)?,
              };
          }
      } else {
          e.null()?;
      }

      e.str("content")?;
      if let Some(ref content) = self.content {
          content.encode(e, _ctx)?;
      } else {
          e.null()?;
      }

      Ok(())
  }
}


#[cfg(test)]
mod tests {
    use super::*;

    const MAX_TEST_CONTENT_SIZE: usize = 1024;
    
    #[test]
    fn test_input_data_decode() {
        let input= [163,101,97,116,116,114,115,152,77,24,130,24,161,24,102,24,108,24,101,24,118,24,101,24,108,24,49,24,162,24,100,24,116,24,121,24,112,24,101,24,101,24,114,24,97,24,110,24,103,24,101,24,100,24,100,24,97,24,116,24,97,24,162,24,99,24,109,24,105,24,110,1,24,99,24,109,24,97,24,120,24,25,3,24,232,24,161,24,102,24,108,24,101,24,118,24,101,24,108,24,50,24,162,24,100,24,116,24,121,24,112,24,101,24,101,24,114,24,97,24,110,24,103,24,101,24,100,24,100,24,97,24,116,24,97,24,162,24,99,24,109,24,105,24,110,1,24,99,24,109,24,97,24,120,24,25,3,24,232,100,115,101,101,100,107,114,97,110,100,111,109,45,115,101,101,100,106,117,115,101,114,95,105,110,112,117,116,106,117,115,101,114,45,105,110,112,117,116];

        let decoded_input: InputData = minicbor::decode::<InputData>(&input).unwrap();

        assert_eq!(decoded_input.deploy_args.len(), 77);
        assert_eq!(decoded_input.seed, "random-seed");
        assert_eq!(decoded_input.user_input, "user-input");
    }

    #[test]
    fn test_deploy_args_decode() {
        let input= [130,161,102,108,101,118,101,108,49,162,100,116,121,112,101,101,114,97,110,103,101,100,100,97,116,97,162,99,109,105,110,1,99,109,97,120,25,3,232,161,102,108,101,118,101,108,50,162,100,116,121,112,101,101,114,97,110,103,101,100,100,97,116,97,162,99,109,105,110,1,99,109,97,120,25,3,232];
        let decoded: DeployArgs = minicbor::decode::<DeployArgs>(&input).unwrap();

        assert_eq!(decoded.args.len(), 2);
        assert_eq!(decoded.args[0].name, "level1");
        assert_eq!(decoded.args[0].arg.type_name, "range");
        assert_eq!(decoded.args[0].arg.data.min, 1);
        assert_eq!(decoded.args[0].arg.data.max, 1000);
        assert_eq!(decoded.args[1].name, "level2");
        assert_eq!(decoded.args[1].arg.type_name, "range");
        assert_eq!(decoded.args[1].arg.data.min, 1);
        assert_eq!(decoded.args[1].arg.data.max, 1000);
    }

    #[test]
    fn test_output_data_encode() {
      let output_data = OutputData {
          amount: 1000,
          attributes: Some(LinearMap::from_iter([
            (String::<MAX_STRING_LEN>::try_from("key1").unwrap(), Value::String(String::<MAX_STRING_LEN>::try_from("value1").unwrap())),
            (String::<MAX_STRING_LEN>::try_from("key2").unwrap(), Value::String(String::<MAX_STRING_LEN>::try_from("value2").unwrap())),
            (String::<MAX_STRING_LEN>::try_from("key3").unwrap(), Value::Int(10)),
            (String::<MAX_STRING_LEN>::try_from("key4").unwrap(), Value::UInt(10)),
            (String::<MAX_STRING_LEN>::try_from("key5").unwrap(), Value::Bool(true)),
            (String::<MAX_STRING_LEN>::try_from("key6").unwrap(), Value::Null)
          ])),
          content: Some(Content {
              content_type: String::<MAX_STRING_LEN>::try_from("text/plain").unwrap(),
              content: Vec::<u8, MAX_CONTENT_SIZE>::from_slice(b"Hello, World!").unwrap(),
          }),
      };

      let mut buf: [u8; MAX_TEST_CONTENT_SIZE] = [0; MAX_TEST_CONTENT_SIZE];
      minicbor::encode(&output_data, buf.as_mut_slice()).unwrap();
    }
}
