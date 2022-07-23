# replica-fvm
Linked data built on FVM actors

An experiment in writing a Filecoin Virtual Machine smart contract using Rust. Create a custom actor for fileDrive that stores the state in local Lotus. Later it should be linked data with on-chain relations, building blocks for data DAOs.

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

Explore possibilities of FVM relational database, that generalizes to social graphs, thread comment, folder system and more. The fundamental is to define on-chain `stream` with immutable identity and append-only, versioned `commits`. 

```   
{streamID, owner, permissions} -> {commitID_1, commitID_2, ...}
```

This can be a supplement for backup of Ceramic network. Users can aggregate 24h of Ceramic streams (offchain), bundle it as a new FVM commit (on-chain).

