import * as utils from './utils';

const cbor = require('@ipld/dag-cbor');
const base64 = require("js-base64");

export async function sign_up(pub_key:string,web3_token:string) {
  console.log("user sign up");
  if (pub_key == null) {
      //get public key
      let key_pair = await utils.get_user_keys();
      pub_key = key_pair["publicKey"];
      console.log(pub_key);
  }

  if (web3_token == null) {
      web3_token = utils.get_web3_token();
  }

  var json_data = "[\""+pub_key+"\",\""+web3_token+"\"]";
  var params = Buffer.from(json_data,"utf8").toString('base64');

  console.log(params);
  return await utils.lotus_send_msg("sign_up",params);
}

export async function get_user(account_id:string) {
  var params = Buffer.from(account_id, "utf8").toString('base64');
  console.log(params);

  let res = await utils.lotus_send_msg("get_user",params);
  if (res.ret == "ok") {
      let data = Buffer.from(res.data,"base64").toString('utf8');  
      console.log(data);
  }

  return res;
}