import * as utils from './utils';
import {storeFiles,retrieveFiles} from './web3.storage';
import { v4 as uuidv4 } from 'uuid';
import * as fs from 'node:fs/promises';
import CryptoJS from "crypto-js";

const cbor = require('@ipld/dag-cbor');
const base64 = require("js-base64");
const path = require('path');

async function encryptSingleFile(file_data:string, file_name:string, password:string) {
    var key = password;
    var encrypted = CryptoJS.AES.encrypt(file_data, key).toString();
    await fs.writeFile("./enc/" + file_name + ".enc", encrypted, "utf8");
}

async function decryptSingleFile(file:Blob, password:string, file_name:string) {
    var textFile = await file.text();
    console.log(textFile.length);
    var key = password;  
    //const textFile = await getFileAsText(file) 
    var decrypted = CryptoJS.AES.decrypt(textFile, key).toString(CryptoJS.enc.Base64);
    var arr = Buffer.from(decrypted,"base64").toString("utf8");
    await fs.writeFile("./dec/"+file_name, arr, "base64");
}

export async function create_file(file_name:string, folder:string,file_type:string)  {
    // const totalSize = files.map(f => f.size).reduce((a, b) => a + b, 0);
    var totalSize = 0;
    var file_data;
    try {
        file_data = await fs.readFile(file_name,"base64");
        console.log('successfully read file');
        var file_stat = await fs.stat(file_name);
        totalSize = file_stat.size;
        console.log("file totalSize is " + totalSize);
    } catch (err) {
        console.error('there was an error:', err);
        return {"ret":"err","data":err};
    }

    let uploaded:number = 0;

    const onRootCidReady = (cid:string) => {
        console.log('upfolderLoading files with cid:', cid);
    }

    const onStoredChunk = (size:number) => {
        uploaded += size;
        const pct = (uploaded / totalSize).toFixed(2);
        console.log(pct);
        //console.log(`UpfolderLoading... ${pct} * 100 % complete`);
    }

    const password = uuidv4();
    let key_pair = await utils.get_user_keys();
    let public_key = key_pair["publicKey"];
    console.log(public_key);
    const {success, cipher} = await utils.encrypt_passwd(public_key, password);
    if (success) {
        console.log("encrypt_passwd successful " + password);
        
        if (file_data == undefined) {
            console.error("file read failed!");
            return;
        }
        
        file_name = path.posix.basename(file_name);
        await encryptSingleFile(file_data.toString(), file_name, password);
        
        console.log('successfully read file');
        const cid = await storeFiles(utils.get_web3_token(), file_name, onRootCidReady, onStoredChunk);
        let file_id = uuidv4();
        console.log("create file new id is " + file_id);
        const current = new Date().getTime();
        var json_data = "[\"" + file_id + 
                        "\",\"" + cid +
                        "\",\"" + file_name + 
                        "\",\"" + folder + 
                        "\",\"" + cipher + 
                        "\",\"" + file_type + 
                        "\"," + current + 
                        ",\"" + "\"," + current + 
                        ",\"" + "\"]";
        var params = Buffer.from(json_data,"utf8").toString('base64');
	    return await utils.lotus_send_msg("sign_up",params);
    } else {
        console.error('Fail to encrypt file ' + file_name);
        return {"ret":"err","data":"Fail to encrypt file"};
    }
}

export async function get_file(file_id:string) {
    var params = Buffer.from(file_id,"utf8").toString('base64');
    console.log(params);
    const get_file_info = await utils.lotus_send_msg("get_file", params);
    console.log(get_file_info);

    if (get_file_info["ret"] == "ok") {
        const file_info_str = Buffer.from(get_file_info["data"].toString(),"base64").toString('utf8');
        console.log(file_info_str);
        let file_info = JSON.parse(file_info_str);
        let file =  {
             id: file_info[0],
             cid: file_info[1],
             file_name: file_info[2],
             folder: file_info[3],
             encrypted_password:file_info[4],
             file_type: file_info[5],
             last_update: file_info[6],
             updated_by: file_info[7],
             created_at: file_info[8],
             created_by: file_info[9]
         };
         console.log(file);
         const files = await retrieveFiles(utils.get_web3_token(),file.cid);
         if (files == null){
            console.log("file not found");
            return;
        }
        
        let cipher = file.encrypted_password;
        if (cipher != null) {
             let key_pair = await utils.get_user_keys();
             let private_key = key_pair["privateKey"];
             const {success, plaintext} = await utils.decrypt_passwd(private_key,cipher);
             if (success) {
                 await decryptSingleFile(files[0],plaintext,file.file_name);
            }
         }
    }

}

export async function share_file(folder_id:string,file_id:string,share_with:string,share_password:string,permissions:number) {
   const current = new Date().getTime();
   var json_data = "[\"" + file_id +
                        "\",\"" + folder_id +
                        "\",\"" + share_with +
                        "\",\"" + share_password +
                        "\"," + permissions +
                        "," + current +
                        ",1]";
  var params = Buffer.from(json_data,"utf8").toString('base64');
  console.log(params);
  return await utils.lotus_send_msg("share_file",params);
}

export async function remove_file(folder_id:string,file_id:string) {
  var params = base64.encode(cbor.encode([folder_id,file_id]));
  console.log(params);
  return await utils.lotus_send_msg("remove_file",params);
}