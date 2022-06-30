# replica-fvm
Linked data built on FVM actors

An experiment in writing a Filecoin Virtual Machine smart contract using Rust. Create a custom actor for private fileDrive (aka. smart contract) that stores the state in your local devnet Lotus node's blockstore.

For more info on the FVM:

* https://fvm.filecoin.io/


## Demo Instructions

* Set up a Lotus devnet
  * Experimental branch: experimental/fvm-m2
  * Instructions: https://lotus.filecoin.io/developers/local-network/
  * Pre-built container images for Docker/Kubernetes: https://github.com/jimpick/lotus-fvm-localnet


* Create replica actor into Lotus
  * `cargo build`
  * ./install-actor.sh 
  * ./create-actor.sh

* Next step Invoke
  * ./invoke.sh actorCID <method num> <encoded-params>

```   
    # Create new folder 
    ./invoke.sh t01020 4 $(echo "[\"123\",\"sub_folder_1\",[],\"1019\",[],1,null,\"100\",1651919905]"|base64 -w 0)    
```


## Dynamic Data

Explore the possibilities of FVM personal database, generalized to social graphs, thread comment, folder system and more. The fundamental is to define on-chain data structures of immutable stream identitiy and append-only versioned commits. 

```   
{StreamCID, owner, permissions} -> {CommitCID_1, CommitCID_2, ...}
```

