mod blockstore;
use crate::blockstore::Blockstore;
use cid::multihash::Code;
use cid::Cid;
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_ipld_encoding::{to_vec, CborStore, RawBytes, DAG_CBOR,from_slice};
use fvm_sdk as sdk;
use fvm_sdk::message::NO_DATA_BLOCK_ID;
use fvm_shared::ActorID;
use std::collections::HashMap;
//use chrono::Utc;
/// A macro to abort concisely.
/// This should be part of the SDK as it's very handy.

macro_rules! abort {
    ($code:ident, $msg:literal $(, $ex:expr)*) => {
        fvm_sdk::vm::abort(
            fvm_shared::error::ExitCode::$code.value(),
            Some(format!($msg, $($ex,)*).as_str()),
        )
    };
}

macro_rules! call_method {
    ($type:ty,$func:ident,$params:expr,$dec:expr) => {{
        let mut state = State::load();
        let mut param: $type = &serde_json::from_slice($params).unwrap();
        let ret = state.$func(param);
        if ret != 0 {
            abort!(
            USR_ILLEGAL_STATE,
            "failed to {:?}: {:?}",
            $dec,get_err_msg(ret));
        }
        state.save();

        // let ret = to_vec(format!("{:?}", &state).as_str());
        // match ret {
        //     Ok(ret) => Some(RawBytes::new(ret)),
        //     Err(err) => {
        //         abort!(
        //             USR_ILLEGAL_STATE,
        //             "failed to serialize return value: {:?}",
        //             err
        //         );
        //     }
        // }}
        let ret = "Ok".as_bytes().to_vec();
        Some(RawBytes::new(ret))
    }}
}

macro_rules! get_method {
    ($func:ident,$params:expr,$dec:expr) => {{
        let mut state = State::load();
        let _id = String::from_utf8($params).unwrap();
        let _data = state.$func(_id.clone());

        match _data {
            Some(data) => {
                let ret = serde_json::to_string(&data).unwrap().as_bytes().to_vec();
                Some(RawBytes::new(ret))
            }
            None => {
                abort!(
                    USR_ILLEGAL_STATE,
                    "{} not found: {:?}",
                    $dec,_id
                );
            }
        }}
    };
}

#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug, Default)]
pub struct Folder {
    id: String,
    name: String,
    files: Vec<String>,
    parent: String,
    children: Vec<String>,
    folder_type: Option<u8>, // 1 for common folder, 2 for shared folder
    folder_password: Option<String>, // None for common folder, decrypt by private key, then used to decrypt folder
    created_by: String,
    created_at: u64,
}

#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug, Default)]
pub struct File {
    id: String,
    cid: String,
    name: String,
    folder: String,
    encrypted_password: Option<String>, //  decrypt by private key, then used to decrypt file
    file_type: String, // type of file (png, jpeg)
    last_update: u64,
    updated_by: String,
    created_at: u64,
    created_by: String
}

#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug, Default)]
pub struct User {
    public_key: String,
    encrypted_token: String // used by web3.storage
}

#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug, Default)]
pub struct SharedDoc {
    doc_id: String,
    parent: String,
    share_with: String,
    share_password: String, //共享目标用户的公钥加密的文件/文件夹加密密码
    permissions: u8, //读/写权限   
    created_at: u64,
    doc_type: u8 // 1 for file, 2 for folder
}

/// The state object.
#[derive(Serialize_tuple, Deserialize_tuple, Clone, Debug)]
pub struct State {
    folders:  HashMap<String, Folder>,
    users: HashMap<String, User>,
    files: HashMap<String, File>,
    shared_docs: HashMap<String, SharedDoc>,
    shared_docs_of_user: HashMap<String, Vec<String>>, // mapping from user to a mapping user => doc which the user own
}

impl State {

    pub fn new() -> Self {
        Self {
            folders: HashMap::new(),
            users: HashMap::new(),
            files: HashMap::new(),
            shared_docs: HashMap::new(),
            shared_docs_of_user: HashMap::new(),
        }
    }

    pub fn sign_up(&mut self, _public_key: String, _encrypted_token: String, _created_at: u64)->String {
        unsafe{
            let account_id = sdk::message::caller();
            let user = User {
                public_key: _public_key,
                encrypted_token: _encrypted_token
            };
            self.users.insert(account_id.to_string(), user);
            let root = Folder {
                id: account_id.to_string(),
                name: String::from("root"),
                files: Vec::new(),
                parent: account_id.to_string(),
                children: Vec::new(),
                folder_password: None,
                created_by: account_id.to_string(),
                created_at: _created_at,
                folder_type: None,
            };
            self.folders.insert(account_id.to_string(), root);
            account_id.to_string()
        }
    }

    pub fn get_user(&self, account_id: String) -> Option<User> {
            match self.users.get(&account_id) {
            Some(user) => Some(user.clone()),
            None => None
        }
    }

    pub fn verify_user(&self ,account_id: String, owner_id: String)->u8 {
        if account_id.ne(&owner_id) {
            return 8;
        } else {
            return 0;
        }
    }

    pub fn validate_folder_id(&self, _folder_id: String) -> u8 {
        match self.users.get(&_folder_id) {
            Some(_) => {
                return 7;
            },
            None => {return 0;}
        }
    }

    pub fn validate_file_id(&self, _file_id: String) -> u8 {
        match self.files.get(&_file_id) {
            Some(_) => {
                //assert!(false, "file id already exists");
                return 7
            },
            None => {return 0;}
        }
    }

    pub fn get_root(&self, folder_id: String) -> (Option<Folder>, String) {
        let mut result = String::from("");
        match self.folders.get(&folder_id) {
            Some(folder_by_id) => {
                let mut current_id = String::from(&folder_id[..]);
                let mut parent_id = String::from(&folder_by_id.parent[..]);
                if current_id.eq(&parent_id) {
                    result = String::from(&current_id[..]);
                } else {
                    while current_id.ne(&parent_id[..]) {
                        match self.folders.get(&parent_id) {
                            Some(folder) => {
                                let temp = current_id.clone();
                                current_id = String::from(&parent_id[..]);
                                parent_id = folder.parent.clone();
                                if current_id.eq(&parent_id) {
                                    result = String::from(&temp[..]);
                                }
                            },
                            None => {},
                        };
                    }
                };
            },
            None => {
            }
        }
        match self.folders.get(&result) {
            Some(root) => {
                (Some(root.clone()), result)
            },
            None => {
                (None, result)
            }
        }
    }

    pub fn verify_accessible(
        &self,
        root_folder: &Option<Folder>,
        folder_id: String,
        account_id: String,
    ) -> u8 {
        match root_folder {
            Some(folder) => {
                let owner = &folder.parent;
                let root_folder_id = folder_id;
                let share_doc_id = format!("{}_{}_{}", owner, account_id, root_folder_id);
                if !owner.eq(&account_id) {
                    match self.shared_docs.get(&share_doc_id) {
                        Some(share_doc) => {
                            if share_doc.permissions != 2 {
                                return 1;
                            }
                        },
                        None => {
                            return 2;
                        }
                    }
                }
            },
            None => {
                return 3;
            }
        }
        return 0;
    }

    pub fn validate_folder_type(&self, root_folder: &Option<Folder>, folder_type: u8) -> u8 {
        match root_folder {
            Some(root_folder_parsed) => {
                if root_folder_parsed.folder_type.is_some() {
                    if root_folder_parsed.folder_type.unwrap() != folder_type {
                        return 4;
                    }
                } else {
                    return 5;
                }
            },
            None => {
                return 6;
            }
        }
        return 0;
    }

    pub fn create_folder(
        &mut self,
        new_folder: &Folder,
    ) -> u8 {
        unsafe {
            let mut new_folder = new_folder.clone();
            self.validate_folder_id(new_folder.id.clone());
            let _account_id = sdk::message::caller();
            let _account_id = _account_id.to_string();
            let _parent = new_folder.parent.clone();
            let _type = new_folder.folder_type.clone();
            let _password = new_folder.folder_password.clone();
            if _parent.ne(&_account_id) {
                let (root_folder, folder_id) = self.get_root(_parent.clone());
                let ret = self.verify_accessible(&root_folder, folder_id, String::from(&_account_id[..]));
                if ret != 0 {
                    return ret;
                }
            }
            let mut folder_password = None;
            let mut folder_type = None;
            if _parent.eq(&_account_id) && _type.is_some() {
                //share folder
                if  _type.unwrap() == 2 {
                    folder_password = _password;
                }
                folder_type = _type
            }
            let _parent_id: &str = _parent.as_str();
            match self.folders.get(&_parent) {
                Some(folder) => {
                    let mut folder = folder.clone();
                    folder.children.push(new_folder.id.clone());
                    self.folders.insert(new_folder.parent.clone(), folder);
                    self.folders.insert(new_folder.id.clone(), new_folder);
                },
                None => {
                    return 6;
                }
            }
            return 0;
        }
    }

    pub fn create_file(
        &mut self,
        new_file: &File,
    ) -> u8 {
        unsafe {
            let _file_id = new_file.id.clone();
            let _folder = new_file.folder.clone();
            let ret = self.validate_file_id(String::from(&_file_id[..]));

            let _account_id = sdk::message::caller().to_string();
            if _folder.ne(&_account_id) {
                let (root_folder, folder_id) = self.get_root(String::from(&_folder[..]));
                let verify_ret = self.verify_accessible(&root_folder, folder_id, String::from(&_account_id[..]));
                if verify_ret != 0 {
                    return verify_ret;
                }
            }
            match self.folders.get(&_folder) {
                Some(mut folder) => {
                    let mut folder = folder.clone();
                    let file = File {
                        id: _file_id.clone(),
                        cid: new_file.cid.clone(),
                        name: new_file.name.clone(),
                        folder: _folder.clone(),
                        encrypted_password: new_file.encrypted_password.clone(),
                        file_type: new_file.file_type.clone(),
                        created_at: new_file.created_at,
                        created_by: _account_id.clone(),
                        updated_by: _account_id.clone(),
                        last_update: new_file.created_at,
                    };
                    let index = folder.files.iter().position(|x| *x == _file_id);
                    if index.is_none() {
                        folder.files.push(_file_id.clone());
                    }
                    self.folders.insert(_folder.clone(), folder);
                    self.files.insert(_file_id.clone(), file);
                },
                None => {
                    //env::log(format!("Folder not found: '{}'", _folder).as_bytes());
                    return 6;
                }
            };
            return 0;
        }
    }

    pub fn share_file(
        &mut self,
        share_file: &SharedDoc,
    ) -> u8 {
        unsafe {
            let _account_id = sdk::message::caller().to_string();
            let _share_with = share_file.share_with.clone();
            let _parent_folder = share_file.parent.clone();
            let _file_id = share_file.doc_id.clone();
            if _share_with.eq(&_account_id) {
                return 10;
            }
            let (root_folder, folder_id) = self.get_root(String::from(&_parent_folder[..]));

            if folder_id.ne(&_account_id) {
                let verify_ret = self.verify_accessible(&root_folder, folder_id, String::from(&_account_id[..]));
                if verify_ret != 0 {
                    return verify_ret;
                }
                let verify_ret = self.validate_folder_type(&root_folder, 1);
                if verify_ret != 0 {
                    return verify_ret;
                }
            }
            match self.folders.get(&_parent_folder) {
                Some(folder) => {
                    let index = folder.files.iter().position(|f| String::from(&f[..]).eq(&_file_id[..]));
                    if index.is_none() {
                        return 12;
                    }
                },
                None => {
                    return 6;
                }
            }
            let share_doc_id = format!("{}_{}_{}", _account_id, _share_with, _file_id);
            let share_doc = SharedDoc {
                doc_id: _file_id,
                parent: _parent_folder.clone(),
                share_with: _share_with.clone(),
                share_password: share_file.share_password.clone(),
                permissions: share_file.permissions.clone(),
                created_at: share_file.created_at.clone(),
                doc_type: 1
            };
            self.shared_docs.insert(share_doc_id.clone(), share_doc.clone());
            match self.shared_docs_of_user.get(&_share_with) {
                Some(mut user_shared_with_docs) => {
                    let mut user_shared_with_docs = user_shared_with_docs.clone();
                    user_shared_with_docs.push(share_doc_id.clone());
                    self.shared_docs_of_user.insert(_share_with.clone() ,user_shared_with_docs);
                },
                None => {
                    let mut new_shared_set = Vec::new();
                    new_shared_set.push(share_doc_id.clone());
                    self.shared_docs_of_user.insert(_share_with.clone() ,new_shared_set);
                }
            }
            return 0;
        }
    }

    pub fn get_file_info(&self, file_id: String) -> Option<File> {
        match self.files.get(&file_id) {
            Some(file) => Some(file.clone()),
            None => None,
        }
    }

    pub fn get_folder_info(&self, folder_id: String) -> Option<Folder> {
        match self.folders.get(&folder_id) {
            Some(folder) => Some(folder.clone()),
            None => None,
        }
    }

    pub fn share_folder(
        &mut self,
        share_folder: &SharedDoc,
    ) -> u8 {
        unsafe {
            let _account_id = sdk::message::caller().to_string();
            let _share_with = share_folder.share_with.clone();
            let _folder_id = share_folder.doc_id.clone();
            let _parent = share_folder.parent.clone();
            if _share_with.eq(&_account_id) {
                return 10;
            }
            let (root_folder, root_folder_id) = self.get_root(String::from(&_folder_id[..]));
            if root_folder_id.ne(&_folder_id) {
                return 11;
            }

            if root_folder_id.ne(&_account_id) {
                let ret = self.verify_accessible(&root_folder, root_folder_id, String::from(&_account_id[..]));
                if ret != 0 {
                    return ret;
                }
                let ret = self.validate_folder_type(&root_folder, 2);
                if ret != 0 {
                    return ret;
                }
            }
            let share_doc_id = format!("{}_{}_{}", _account_id, _share_with, _folder_id);
            let share_doc = SharedDoc {
                doc_id: _folder_id,
                parent: _parent,
                share_with: share_folder.share_with.clone(),
                share_password: share_folder.share_password.clone(),
                permissions: share_folder.permissions.clone(),
                created_at: share_folder.created_at.clone(),
                doc_type: 2
            };
            self.shared_docs.insert(share_doc_id.clone(), share_doc);
            match self.shared_docs_of_user.get(&_share_with) {
                Some(mut user_shared_with_docs) => {
                    let mut user_shared_with_docs = user_shared_with_docs.clone();
                    user_shared_with_docs.push(share_doc_id.clone());
                    self.shared_docs_of_user.insert(_share_with.clone() ,user_shared_with_docs.clone());
                },
                None => {
                    let mut new_shared_set = Vec::new();
                    new_shared_set.push(share_doc_id.clone());
                    self.shared_docs_of_user.insert(_share_with.clone() ,new_shared_set);
                }
            }
            return 0;
        }
    }

    pub fn remove_file(&mut self, _folder: String, _file: String) -> u8 {
        unsafe{
            let _account_id = sdk::message::caller().to_string();
            let (root_folder, _) = self.get_root(String::from(&_folder[..]));
            match root_folder {
                Some(root_folder_unwrapped) => {
                    let owner_id = root_folder_unwrapped.parent;
                    let ret = self.verify_user(_account_id, owner_id);
                    if ret != 0 {
                        return ret;
                    }
                },
                None => {
                    return 9;
                }
            }
            match self.folders.get(&_folder) {
                Some(mut folder) => {
                    let mut folder = folder.clone();
                    let index = folder.files.iter().position(|x| x.clone() == _file).unwrap();
                    folder.files.remove(index);
                    self.folders.insert(_folder.clone(), folder);
                    self.files.remove(&_file);
                },
                None => {
                    return 6;
                }
            }
            return 0;
        }
    }

    pub fn remove_folder(&mut self, _folder: String) -> u8 {
        unsafe {
            let _account_id = sdk::message::caller().to_string();
            let (root_folder, _) = self.get_root(String::from(&_folder[..]));
            match root_folder {
                Some(root_folder_unwrapped) => {
                    let owner_id = root_folder_unwrapped.parent;
                    let ret = self.verify_user(_account_id, owner_id);
                    if ret != 0 {
                        return ret;
                    }
                },
                None => {
                    // assert!(false, "root folder not found")
                    return 9;
                }
            }
            if let Some(folder) =  self.folders.get(&_folder) {
                if let Some(parent_folder) = self.folders.get(&folder.parent) {
                    let mut parent_folder = parent_folder.clone();
                    let index = parent_folder.children.iter().position(|x| *x == _folder).unwrap();
                    parent_folder.children.remove(index);
                    self.folders.insert(folder.parent.clone(), parent_folder);
                }
                self.folders.remove(&_folder);
            }
            return 0;
        }
    }

}

/// We should probably have a derive macro to mark an object as a state object,
/// and have load and save methods automatically generated for them as part of a
/// StateObject trait (i.e. impl StateObject for State).
impl State {
    pub fn load() -> Self {
        // First, load the current state root.
        let root = match sdk::sself::root() {
            Ok(root) => root,
            Err(err) => abort!(USR_ILLEGAL_STATE, "failed to get root: {:?}", err),
        };

        // Load the actor state from the state tree.
        match Blockstore.get_cbor::<Self>(&root) {
            Ok(Some(state)) => state,
            Ok(None) => abort!(USR_ILLEGAL_STATE, "state does not exist"),
            Err(err) => abort!(USR_ILLEGAL_STATE, "failed to get state: {}", err),
        }
    }

    pub fn save(&self) -> Cid {
        let serialized = match to_vec(self) {
            Ok(s) => s,
            Err(err) => abort!(USR_SERIALIZATION, "failed to serialize state: {:?}", err),
        };
        let cid = match sdk::ipld::put(Code::Blake2b256.into(), 32, DAG_CBOR, serialized.as_slice())
        {
            Ok(cid) => cid,
            Err(err) => abort!(USR_SERIALIZATION, "failed to store initial state: {:}", err),
        };
        if let Err(err) = sdk::sself::set_root(&cid) {
            abort!(USR_ILLEGAL_STATE, "failed to set root ciid: {:}", err);
        }
        cid
    }
}

/// The actor's WASM entrypoint. It takes the ID of the parameters block,
/// and returns the ID of the return value block, or NO_DATA_BLOCK_ID if no
/// return value.
///
/// Should probably have macros similar to the ones on fvm.filecoin.io snippets.
/// Put all methods inside an impl struct and annotate it with a derive macro
/// that handles state serde and dispatch.
#[no_mangle]
pub fn invoke(params: u32) -> u32 {
    // Conduct method dispatch. Handle input parameters and return data.
    let ret: Option<RawBytes> = match sdk::message::method_number() {
        1 => constructor(),
        2 => get(),
        3 => {
            let params = sdk::message::params_raw(params).unwrap().1;
            /*    let user = User {
                    public_key: String::from("0x801daf546e570ea21864d375096f1e1b7ce5331c2ebcd99e2196a96005b4829219cbaf7b275f417efc2797415ee2d2c9099ad0583e33f47663044e619684c37ebaca64b504d563b0fb13412ce674843ce2b83ac5e053fe8246b08c54c68c79fd"),
                    encrypted_token: String::from("1234567890")
                };
                let params = serde_json::to_vec(&user).unwrap();*/
            //let params = RawBytes::new(params);
            //Some(params)
            sign_up(params)
        },
        4 => {
            // let params = sdk::message::params_raw(params).unwrap().1;
            /*let account_id = sdk::message::caller();
            //let dt = Utc::now();
            //let timestamp: u64 = dt.timestamp() as u64;
            let folder = Folder {
                id: String::from("123"),
                name: String::from("sub_folder_1"),
                files: Vec::new(),
                parent: account_id.to_string(),
                children: Vec::new(),
                folder_password: None,
                created_by: account_id.to_string(),
                created_at: 1651919905,
                folder_type: Some(1),
            };
            let params = serde_json::to_vec(&folder).unwrap();*/
            let params = sdk::message::params_raw(params).unwrap().1;
            //let params = RawBytes::new(params);
            create_folder(params)
        },
        5 => {
            /*let file = File {
                id: String::from("234"),
                cid: String::from("aaaaaabbbbbbccccccc"),
                name: "my_file_1.rs".to_string(),
                folder: String::from("100"),
                encrypted_password: None,
                file_type: String::from("1"),
                last_update: 0,
                updated_by: String::from("100"),
                created_at: 0,
                created_by: String::from("100")
            };
            let params = serde_json::to_vec(&file).unwrap();*/
            let params = sdk::message::params_raw(params).unwrap().1;
             // let params = RawBytes::new(params);
             create_file(params)
        },
        6 => {
             let params = sdk::message::params_raw(params).unwrap().1;
             // let params = RawBytes::new(params);
             get_user(params)
        },
        7 => {
            //share folder
            /*let share_doc = SharedDoc {
                doc_id: String::from("100"),
                parent: String::from("100"),
                share_with: String::from("101"),
                share_password: String::from(""),
                permissions: 2,
                created_at: 0,
                doc_type: 2 // 1 for file, 2 for folder
            };
            let params = serde_json::to_vec(&share_doc).unwrap();*/
            let params = sdk::message::params_raw(params).unwrap().1;
            share_folder(params)

        },
        8 => {
            //share file
            /*let share_doc = SharedDoc {
                doc_id: String::from("234"),
                parent: String::from("100"),
                share_with: String::from("101"),
                share_password: String::from(""),
                permissions: 2,
                created_at: 0,
                doc_type: 1 // 1 for file, 2 for folder
            };
            let params = serde_json::to_vec(&share_doc).unwrap();*/
            let params = sdk::message::params_raw(params).unwrap().1;
            share_file(params)

        },
        9 => {
            //get file
            let params = sdk::message::params_raw(params).unwrap().1;
            get_file(params)
        }
        10 => {
            //get folder
            let params = sdk::message::params_raw(params).unwrap().1;
            get_folder(params)
        }
        11 => {
            //remove file
            //let params = sdk::message::params_raw(params).unwrap().1;
            remove_file(params)
        }
        12 => {
            //remove folder
            let params = sdk::message::params_raw(params).unwrap().1;
            remove_folder(params)
        }
        _ => abort!(USR_UNHANDLED_MESSAGE, "unrecognized method"),
    };

    // Insert the return data block if necessary, and return the correct
    // block ID.
    match ret {
        None => NO_DATA_BLOCK_ID,
        Some(v) => match sdk::ipld::put_block(DAG_CBOR, v.bytes()) {
            Ok(id) => id,
            Err(err) => abort!(USR_SERIALIZATION, "failed to store return value: {}", err),
        },
    }
}

/// The constructor populates the initial state.
///
/// Method num 1. This is part of the Filecoin calling convention.
/// InitActor#Exec will call the constructor on method_num = 1.
pub fn constructor() -> Option<RawBytes> {
    // // This constant should be part of the SDK.
    // const INIT_ACTOR_ADDR: ActorID = 1;
    //
    // // Should add SDK sugar to perform ACL checks more succinctly.
    // // i.e. the equivalent of the validate_* builtin-actors runtime methods.
    // // https://github.com/filecoin-project/builtin-actors/blob/master/actors/runtime/src/runtime/fvm.rs#L110-L146
    // if sdk::message::caller() != INIT_ACTOR_ADDR {
    //     abort!(USR_FORBIDDEN, "constructor invoked by non-init actor");
    // }

    let state = State::new();
    state.save();
    None
}

/// Method num 2.
pub fn get() -> Option<RawBytes> {
    let state = State::load();

    let ret = to_vec(format!("{:?}", &state).as_str());
    match ret {
        Ok(ret) => Some(RawBytes::new(ret)),
        Err(err) => {
            abort!(
                USR_ILLEGAL_STATE,
                "failed to serialize return value: {:?}",
                err
            );
        }
    }
}

/// Method num 3.
pub fn sign_up(params: Vec<u8>) -> Option<RawBytes> {
    let mut state = State::load();
    let mut new_user: &User = &serde_json::from_slice(&params).unwrap();
    //let dt = Utc::now();
    //let timestamp: u64 = dt.timestamp() as u64;
    let account = state.sign_up(new_user.public_key.clone(),new_user.encrypted_token.clone(),0u64);
    state.save();

    let ret = account.as_bytes().to_vec();
    Some(RawBytes::new(ret))
}

/// Method num 4.
pub fn create_folder(params: Vec<u8>) -> Option<RawBytes> {
    call_method!(&Folder,create_folder,&params,"create folder")
}

/// Method num 5.
pub fn create_file(params: Vec<u8>) -> Option<RawBytes> {
    call_method!(&File,create_file,&params,"create file")
}

/// Method num 6.
pub fn get_user(params: Vec<u8>) -> Option<RawBytes> {
    get_method!(get_user,params,"user")
}

/// Method num 7.
pub fn share_folder(params: Vec<u8>) -> Option<RawBytes> {
    call_method!(&SharedDoc,share_folder,&params,"share folder")
}

/// Method num 8.
pub fn share_file(params: Vec<u8>) -> Option<RawBytes> {
    call_method!(&SharedDoc,share_file,&params,"share file")
}

/// Method num 9.
pub fn get_file(params: Vec<u8>) -> Option<RawBytes> {
    get_method!(get_file_info,params,"file")
}

/// Method num 10.
pub fn get_folder(params: Vec<u8>) -> Option<RawBytes> {
    get_method!(get_folder_info,params,"folder")
}

/// Method num 11.
pub fn remove_file(params: u32) -> Option<RawBytes> {
    let (_, raw) = match sdk::message::params_raw(params) {
        Ok(tup) => tup,
        Err(err) => abort!(USR_ILLEGAL_ARGUMENT, "remove file:failed to receive params: {:?}", err),
    };

    let p: Vec<String> = match from_slice(raw.as_slice()) {
        Ok(item) => item,
        Err(err) => abort!(USR_ILLEGAL_ARGUMENT, "remove file:failed to unmarshal params: {:?}", err),
    };

    let mut state = State::load();
    //let dt = Utc::now();
    //let timestamp: u64 = dt.timestamp() as u64;
    let ret = state.remove_file(p[0].clone(),p[1].clone());
    if ret != 0 {
       abort!(
        USR_ILLEGAL_STATE,
        "failed to remove file: {:?}",
        get_err_msg(ret));
    }
    state.save();

    let ret = "Ok".as_bytes().to_vec();
    Some(RawBytes::new(ret))

}

/// Method num 12.
pub fn remove_folder(params: Vec<u8>) -> Option<RawBytes> {
    let folder_id = String::from_utf8(params).unwrap();
    let mut state = State::load();
    //let dt = Utc::now();
    //let timestamp: u64 = dt.timestamp() as u64;
    let ret = state.remove_folder(folder_id.clone());
    if ret != 0 {
       abort!(
        USR_ILLEGAL_STATE,
        "failed to remove folder: {:?}",
        get_err_msg(ret));
    }
    state.save();

    let ret = "Ok".as_bytes().to_vec();
    Some(RawBytes::new(ret))
}

pub fn get_err_msg(err_code:u8) -> String {
    match err_code {
        1 => {String::from("You do not have permissions to change this folder")},
        2 => {String::from("You are not shared with this doc")},
        3 => {String::from("You do not have permissions to change this folder")},
        4 => {String::from("folder type is not matched")},
        5 => {String::from("folder type is null")},
        6 => {String::from("folder not found")},
        7 => {String::from("id has been used")},
        8 => {String::from("owner not matched")},
        9 => {String::from("root folder not found")},
        10 => {String::from("cannot share to yourself")},
        11 => {String::from("this is not the root folder")},
        12 => {String::from("file not in folder")},
        _ => {String::from("undefined err code")}
    }
}

/*#[cfg(test)]
mod serde_test {
     use super::*;

     #[test]
     fn test_serde_json() {
                let account_id = 100;
                //let dt = Utc::now();
                //let timestamp: u64 = dt.timestamp() as u64;
                let folder = Folder {
                    id: String::from("123"),
                    name: String::from("sub_folder_1"),
                    files: Vec::new(),
                    parent: account_id.to_string(),
                    children: Vec::new(),
                    folder_password: None,
                    created_by: account_id.to_string(),
                    created_at: 1651919905,
                    folder_type: Some(1),
                };
                let params = serde_json::to_vec(&folder).unwrap();
                println!("{:?}",String::from_utf8(params));
                //let ret = sign_up(json_vec).unwrap();
                //println!("{:?}",ret);

     }
 }
*/