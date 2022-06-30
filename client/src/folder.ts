import { v4 as uuidv4 } from 'uuid';
import * as utils from './utils';

export async function create_folder(folder_name:string, parent:string, folder_type:string) {const currentTimeStamp = new Date().getTime();
    const id = uuidv4();
    const folder_password = uuidv4();
    const keys = await utils.get_user_keys();
    const public_key = keys["publicKey"];
    const {success, cipher} = await utils.encrypt_passwd(public_key, folder_password);
    if (success) {
      var json_data = "[\"" + id + 
                        "\",\"" + folder_name + 
                        "\",[]," + 
                        "\"" + parent + 
                        "\",[],\"" + cipher + 
                        "\",\"\"," + currentTimeStamp + 
                        ",\"" + folder_type + "\"]";
      console.log(json_data);
      var params = Buffer.from(json_data, "utf8").toString('base64');
      console.log(params);
      return await utils.lotus_send_msg("remove_file", params);
    } else {
      return {"ret":"err","data":"failed to encrypt password"}; 
    }
}

export async function get_folder(folder_id:string) {
  var params = Buffer.from(folder_id,"utf8").toString('base64');
  let res = await utils.lotus_send_msg("get_folder",params);
  console.log(params);
  if (res.ret == "ok") {
      let data = Buffer.from(res.data,"base64").toString('utf8');
      console.log(data);
  }

  return res;
}

export async function remove_folder(folder_id:string) {
  var params = Buffer.from(folder_id,"utf8").toString('base64');
  console.log(params);
  return await utils.lotus_send_msg("remove_folder",params);
}

export async function share_folder(folder_id:string,parent:string,share_with:string,share_password:string,permissions:number) {
   const current = new Date().getTime();
   var json_data = "[\"" + folder_id +
                        "\",\"" + parent +
                        "\",\"" + share_with +
                        "\",\"" + share_password +
                        "\"," + permissions +
                        "," + current +
                        ",2]";
  var params = Buffer.from(json_data,"utf8").toString('base64');
  console.log(params);
  return await utils.lotus_send_msg("share_folder",params);
}