import { Web3Storage, getFilesFromPath,File } from 'web3.storage';

function makeStorageClient(TOKEN:string) {
    return new Web3Storage({ token: TOKEN })
}

export async function storeFiles(TOKEN:string, file_path:string, onRootCidReady: (cid:string) => void, onStoredChunk:(size:number)=>void ) {
    const client = makeStorageClient(TOKEN);
    const files = await getFilesFromPath("./enc/" +file_path + ".enc");
    console.log(files);
    const cid = await client.put(files, { onRootCidReady, onStoredChunk })
    return cid
}

export async function retrieveFiles(TOKEN:string, cid:string):Promise<File[]> {
    const client = makeStorageClient(TOKEN)
    const res = await client.get(cid)
    if (res == null){
        console.error("res is null");
	return [];
    }
    console.log(`Got a response! [${res.status}] ${res.statusText}`)
    if (!res.ok) {
        throw new Error(`failed to get ${cid} - [${res.status}] ${res.statusText}`)
    }

    // unpack File objects from the response
    const files = await res.files()
    for (const file of files)  {
       console.log(`${file.name}`);
    }

    return files
}

export async function retrieve(TOKEN:string, cid:string) {
    const client = makeStorageClient(TOKEN)
    const res = await client.get(cid)
    if (res == null){
        console.error("res is null");
        return;
    }

    console.log(`Got a response! [${res.status}] ${res.statusText}`)
    if (!res.ok) {
        throw new Error(`failed to get ${cid}`)
    }

    // request succeeded! do something with the response object here...
}

export async function checkFileStatus(TOKEN:string, cid:string) {
    const client = makeStorageClient(TOKEN)
    const status = await client.status(cid)
    if (status) {
        console.log(status)
    }
}

export async function validateToken(token:string) {
    const web3storage = new Web3Storage({ token })
    try {
        for await (const _ of web3storage.list({ maxResults: 1})) {
            break
        }
        return true
    } catch (e) {
        return false
    }
}
