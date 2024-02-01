# **dhooks_backend**

**`dhooks_backend`** is an Internet Computer Protocol (ICP) canister project designed to perform periodic tasks by interacting with Ethereum Virtual Machine (EVM) chains. It utilizes a timer to send RPC requests to specified EVM chains at user-defined intervals. Upon receiving the response, it executes a predefined **`icphook`** function within a Solidity contract, processes the return value, and then calls another specified endpoint.

## **Features**

- **EVM Chain Interaction**: Interacts with any EVM-compatible blockchain by sending RPC requests.
- **Customizable Intervals**: Users can define the interval at which the RPC requests are sent.
- **Solidity Integration**: Calls a predefined **`icphook`** function within a Solidity contract and handles the return values.
- **Endpoint Notification**: After processing the return value, it triggers another call to a specified endpoint.

## **Prerequisites**

Before deploying **`dhooks_backend`**, ensure you have the following prerequisites installed:

- DFINITY Canister SDK (dfx): Required for building and deploying ICP canisters.
- [Node.js](https://nodejs.org/): Optional, if your project involves any Node.js scripts or tooling.
- An Internet connection for deploying the canister and sending RPC requests.

## **Installation**

1. Clone the repository to your local machine:

```bash
git clone https://github.com/grandzero/dhooks.git
cd dhooks

```

1. Install any necessary dependencies (if applicable):

```bash
npm install

```

1. Start the local development network (if you wish to test locally before deploying):

```bash
dfx start --background --clean

```

## **Deployment**

To deploy **`dhooks_backend`** with custom configuration, use the following **`dfx`** command, replacing the arguments with your own values:

```bash
dfx deploy dhooks_backend --argument '(<interval_seconds>,"<rpc_endpoint>", "<contract_address>", "<callback_url>")'

```

### **Example**

Deploying with a 60-second interval, using the Mumbai testnet RPC endpoint, targeting a specific contract address, and specifying a callback URL:

```bash
dfx deploy dhooks_backend --argument '(60,"https://endpoints.omniatech.io/v1/matic/mumbai/public", "0x077E6925a039B7818Eaf5DbF088F289c445a7E32", "https://putsreq.com/GhNAATx0manjojh9nIOy")'

```
### **Warning (!)**

Currently, icp https outcalls only supports ipv6. Please make sure your callback url support ipv6. Here putsreq used for example which is supporting ipv6.

### **How It Works**

Canister expects target contract to have a function with the signature "icphooks(bytes) returns (bytes memory)" . If contract implements this function, then every given second, timer will use rpc url to send a rpc request to get data from contract, if it's successfull, then it will simply call given callback endpoint url with the results. This is usefull for projects that receives payment and need to setup their own event listening system. Canister has some limitations like max cycle, max response size etc.

### **Function**

Even canister works automatically once deployed, there are 2 test function for testing purposes. get_data_from_evm function sends rpc request and if it's succcessful, it returns success message and get timer is a sanity check for timer (if it's working)

## **Configuration Parameters**

- **`interval_seconds`**: Time in seconds between each RPC request to the EVM chain.
- **`rpc_endpoint`**: The URL of the RPC endpoint for the EVM chain.
- **`contract_address`**: The address of the contract where the **`icphook`** function is defined.
- **`callback_url`**: The URL to call with the response data after processing.

## **Usage**

After deployment, **`dhooks_backend`** will automatically start performing its tasks based on the provided configuration. There is no additional user interaction required unless you want to update the deployment with new parameters.

## **To Do**
- [ ]  Add endpoint security
- [ ]  Setting multiple hooks
- [ ]  Custom header selection for hooks
- [ ]  Authorized hook register (only authorized user should register new hooks)
- [ ]  Customize response size (in bytes)
- [ ]  Customize cycle use per hook
- [ ]  Using ethabi crate (ethers_core crate is deprecated)
