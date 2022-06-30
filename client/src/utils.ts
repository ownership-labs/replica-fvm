import { HttpJsonRpcConnector, LotusClient, LotusWalletProvider } from 'filecoin.js';
import * as secp256k1 from 'noble-secp256k1';
import BigNumber from 'bignumber.js';

const EthCrypto = require('eth-crypto');
const __LOTUS_RPC_ENDPOINT__="http://127.0.0.1:1234/rpc/v0";
const __LOTUS_AUTH_TOKEN__="Your Auth Token";
const WEB3_TOKEN = "Your Web3.Storage Token";
const CONTRACT_ID_ADDRESS = "t01002";

export function get_web3_token() {
    return WEB3_TOKEN;
}

export function get_lotus_rpc_endpoint() {
    return __LOTUS_RPC_ENDPOINT__;
}

export function get_lotus_auth_token() {
    return __LOTUS_AUTH_TOKEN__;
}

export function get_contract_addr() {
    return CONTRACT_ID_ADDRESS;
}

export function get_method_num(method_name:string) {
    switch (method_name) {
        case "sign_up":
            return 3;
        case "create_folder":
            return 4;
        case "create_file":
            return 5;
        case "get_user":
            return 6;
        case "share_folder":
            return 7;
        case "share_file":
            return 8;
        case "get_file":
            return 9;
        case "get_folder":
            return 10;
        case "remove_file":
            return 11;
        case "remove_folder":
            return 12;
        default:
            return 0;
    }
}

export async function get_user_keys() {
  //get public key
  console.log("get_user_keys");

  const connector = new HttpJsonRpcConnector({ url: get_lotus_rpc_endpoint(), token: get_lotus_auth_token() });
  const jsonRpcProvider = new LotusClient(connector);
  const walletProvider = new LotusWalletProvider(jsonRpcProvider);
  const myAddress = await walletProvider.getAddresses();
  const key = await walletProvider.exportPrivateKey(myAddress[0]);
  let private_key = Buffer.from(key.PrivateKey.toString(),"base64").toString("hex");
  const pubkey = Buffer.from(secp256k1.getPublicKey(private_key)).toString();

  return { "privateKey": private_key, "publicKey": pubkey };
}

export async function encrypt_passwd(publicKey:string, data:string) {
    try {
        const encrypted = await EthCrypto.encryptWithPublicKey(
            publicKey,
            data
        );
        const cipher = EthCrypto.cipher.stringify(encrypted)
        return {success: true, cipher}
    } catch(err) {
        return {success: false, error: err}
    } 
}

export async function decrypt_passwd(privateKey:string, data:string) {
    try {
        const cipher = EthCrypto.cipher.parse(data)
        const encrypted = await EthCrypto.decryptWithPrivateKey(
            privateKey,
            cipher
        );
        return {success: true, plaintext: encrypted}
    } catch(err) {
        return {success: false, error: err}
    } 
}

export async function lotus_send_msg(method_name:string, params:string) {
	const connector = new HttpJsonRpcConnector({ url: get_lotus_rpc_endpoint(), token: get_lotus_auth_token() });
        const jsonRpcProvider = new LotusClient(connector);
        const walletProvider = new LotusWalletProvider(jsonRpcProvider);

	const owner = await walletProvider.getDefaultAddress();
	let contract_addr = get_contract_addr();
	let contract_method_num = get_method_num(method_name);
    let _message = await walletProvider.createMessage({
        To: contract_addr,
        From: owner,
        Value: new BigNumber(0),
        Method: contract_method_num,
        Params: params
    });
    
    console.log("going to call method, num: " + contract_method_num);
    console.log(_message);

    let signMessage = await walletProvider.signMessage(_message);
    let mcid = await walletProvider.sendSignedMessage(signMessage);
    let res = await jsonRpcProvider.state.waitMsg(mcid, 1);
    
    let data = Buffer.from(res.Receipt.Return,"base64").toString('utf8');

    if (res.Receipt.ExitCode == 0) {
        console.log("method executed successfully");
        return {"ret":"ok", "data":data};
    } else {
        console.log(res.Receipt.Return);
        return {"ret":"err", "data":data};
    }
}
