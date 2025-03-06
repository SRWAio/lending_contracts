## Requirements

1. Rust Programming Language: Ensure that you have Rust installed on your machine. The recommended version is 1.81.0 or later. You can install Rust by following the instructions at rust-lang.org.

2. Scrypto CLI: Install the Scrypto CLI, which is required for compiling and running Scrypto blueprints. The recommended version is 1.3.0. You can install it by following the instructions at Scrypto Documentation.

3. Dependencies: Ensure that the necessary dependencies are specified in your Cargo.toml file. The dependencies include scrypto, scrypto-test, and any other crates you are using.

## Steps to Run the Code

Clone the Repository: Clone the repository code to your local machine.

`git clone <repository-url>`

## Resim

resim is a command line tool that is used to interact with local Radix Engine simulator during development.
All of the interactions with the backend are done with resim for now.

On every start or whenever you want to try something new, first command should be `resim reset`. It deletes everything from the simulator.

## Creating a new account

Running this command:
`resim new-account `
creates a new account on the simulator. When done the first time it will automatically set the account as default. Response will be something like this:

`A new account has been created!
Account component address: account_sim1qdu23xcp4jcvurxvnap5e7994xzfza8e0myjaez0s73qd2wye3
Public key: 0208beddb4a109910b5fc9ddfe8b370351bf3e6430874d2ae9e65e3a863b8b6bd6
Private key: 4edf45bf7b6da8ac4d06fec8512c9b6d37a8288c9b39e5ebbb738e60e028a297
NonFungibleGlobalId: resource_sim1qpugpy08q9mp8v9vzcs0y6yyzw5003ratjue48ag2j4s73casc:#1#
Account configuration in complete. Will use the above account as default.`

Save the Account component address, Private and Public keys and NonFungibleGlobalId as you'll need it later.
When you make the account, it will have 1000 XRD on it by default.
If you want to see the details of the account you can use:

`resim show <ACCOUNT_ADDRESS>`

At the bottom of the response there is a list of all of the resources that the account has, for example:

`Resources:
├─ { amount: 898.6816111, resource address: resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqz8qety, name: "Radix", symbol: "XRD" }
├─ { amount: 1, resource address: resource_sim1qzmm88ant9lrace0ya6x63gapqnkg3v644kcqd8y0m2qngeg6w, name: "Admin Badge" }
├─ { amount: 1, resource address: resource_sim1qp2ahm386cw0hcxmyj88r4w249wqrgnyh7ncu66v3lqq2rrts3, name: "User Badge" }
│  └─ NonFungible { id: NonFungibleLocalId("#1#"), immutable_data: Tuple(), mutable_data: Tuple() }
├─ { amount: 1, resource address: resource_sim1qpugpy08q9mp8v9vzcs0y6yyzw5003ratjue48ag2j4s73casc, name: "Owner badge" }
│  └─ NonFungible { id: NonFungibleLocalId("#1#"), immutable_data: Tuple(), mutable_data: Tuple() }`

## Publishing the package

First step is to publish pool project.

Navigate to /lending_contracts/pool, then use this command:

`resim publish .`

At the bottom of the response you'll get the package address, copy it into protocol.rs, navigate to /lending_contracts/lending_protocol

and use the same command to publish lending_protocol.

## Component Instantiation

To instantiate the LendingProtocol component run this command:

`resim call-function <PACKAGE_ADDRESS> LendingProtocol instantiate <ORACLE_COMPONENT_ADDRESS>`

You'll get the component and resource addresses in the response, something like this:

`└─ Component: component_sim1czwnyl3pfn955s45a2js64w8zjlptwz4y3w4wwwl944rk2l2ceapsc
├─ Resource: resource_sim1nfdrpva6v0chjn5h2k8gwj0wg7et8rdsxwgmvd25m0rzxv696rmnsk
├─ Resource: resource_sim1ngy8zxuvjaxnufmqp7jevml7f3v59mk4p4c2f2tqkvu4f9y7vjkw5a
└─ Resource: resource_sim1n2dsu340jv376hrd87cxsjtstw7hh8t330vk40q8z45rs2x2gsnkce`

Component address is the address of the instantiated component and it will be used for all of the transactions later on.
First Resource address is the Admin Badge that will be used for creating the Proof for using the admin specific methods.
Second Resource address is Protocol Badge that will be used for communication between lendin_protocol and pool contracts.
Third Resource address is User Badge resource address. Users will get the NFT on their first deposit with this resource address and specific ID.
You can see the info of resources with `resim show <RESOURCE_ADDRESS>`.

## Transactions

Transactions are stored in the manifests folder in the project.
Transaction consists of several parts:

##### Locking the fee payment

`CALL_METHOD ComponentAddress("<ACCOUNT_ADDRESS>") "lock_fee" Decimal("10");`

Every transaction needs to have this othervise it will not work.
User is always paying the fee, so you'll be using the same account address as for everything else in the transaction.

##### Creating the Proof

In order to use some of the methods on the app you must be authenticated. That is done through Proofs.
They are created using your account address and your badge, for now it can be Admin Badge or User Badge.

First step is to create the proof:

`CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_OR_USER_BADGE_RESOURCE_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#<NFT_ID>#")
    )
;`

Proof is created and put in auth zone, so the next step is to get it so you can use it.

`CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES
    Address("<ADMIN_OR_USER_BADGE_RESOURCE_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#<NFT_ID>1#")
    )
    Proof("NAME_OF_THE_PROOF")
;`

Name of the proof can be anything, for the sake of keeping things simple we will be using admin_badge and user_badge.

## Interacting with the app

There are several transactions you can use in order to do things on the app.

##### approve_admin_functions

This transaction will be used for all admin functions.
Three different admins must execute this function to enable use od admin functions (something like multisig).

Run it with this command:
`resim run "./manifests/approve_admin_functions.rtm"`

`CALL_METHOD
    Address("<ACCOUNT_ADDRESS>>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#<ADMIN_BADGE_ID>#")
    )
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#<ADMIN_BADGE_ID>#")
    )
;
POP_FROM_AUTH_ZONE
    Proof("<PROOF_NAME>")
;
CALL_METHOD
    Address("<PROTOCOL_COMPONENT_ADDRESS>")
    "approve_admin_functions"
    Proof("<PROOF_NAME>")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "try_deposit_batch_or_refund"
    Expression("ENTIRE_WORKTOP")
    Enum<0u8>()
;
`

##### create_pool

First transaction must be create_pool, it's creating the pool that can be used for lending.
This function will instantiate Pool component from pool contract.
It is something that only admin can do so it requires admin badge and it requires approval from at least 3 admins.

Run it with this command:

`resim run "./manifests/create_pool.rtm"`

`CALL_METHOD
    Address("<ADMIN_ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<ADMIN_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#<ADMIN_BADGE_ID>#")
    )
;
CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES
    Address("<ADMIN_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#<ADMIN_BADGE_ID>#")
    )
    Proof("<NAME_OF_PROOF>")
;
CALL_METHOD
    Address("<PROTOCOL_COMPONENT_ADDRESS>")
    "create_pool"
    Address("<RESOURCE_ADDRESS>")
    Decimal("<BASE>")
    Decimal("<BASE_MULTIPLIER>")
    Decimal("<MULTIPLIER>")
    Decimal("<KINK>")
    Decimal("<RESERVE_FACTOR>")
    Decimal("<LTV_RATIO>")
;
CALL_METHOD
    Address("<ADMIN_ACCOUNT_ADDRESS>")
    "try_deposit_batch_or_refund"
    Expression("ENTIRE_WORKTOP")
    Enum<0u8>()
;`

If XRD is the asset then the asset resource address will be resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqz8qety.
LTV ratio is a decimal number between 0 and 1.
Multiplier must be greater then 0 and greater then base multiplier.
Base must be greater then 0.
Reserve Factor must be between 0 and 1.
Kink must be between 0 and 100.

##### create_user_and_deposit

Run it with this command:

`resim run "./manifests/create_user_and_deposit.rtm"`

`CALL_METHOD ComponentAddress("<ACCOUNT_ADDRESS>") "lock_fee" Decimal("10");
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "withdraw"
    Address("<RESOURCE_ADDRESS>")
    Decimal("<DEPOSIT_AMOUNT>")
;
TAKE_FROM_WORKTOP
    Address("<RESOURCE_ADDRESS>")
    Decimal("<DEPOSIT_AMOUNT>")
    Bucket("<BUCKET_NAME>")
;
CALL_METHOD
    Address("<COMPONENT_ADDRESS>")
    "create_user_and_deposit"
    Bucket("<BUCKET_NAME>")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
`

After running this command, user is created and stored in the app and if you run the command `resim show <ACCOUNT_ADDRESS>` you should see that you have the User Badge in resources.
All information about user balances are stored in the User Badge metadata.

##### Deposit

User can now make a deposit, run it with this command:

`resim run "./manifests/deposit.rtm"`

`CALL_METHOD ComponentAddress("<ACCOUNT_ADDRESS>") "lock_fee" Decimal("10");
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<USER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#<USER_BADGE_ID>#")
    )
;
CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES
    Address("<USER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#<USER_BADGE_ID>#")
    )
    Proof("<PROOF_NAME>")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "withdraw"
    Address("<RESOURCE_ADDRESS>")
    Decimal("<DEPOSIT_AMOUNT>")
;
TAKE_FROM_WORKTOP
    Address("<RESOURCE_ADDRESS>")
    Decimal("<DEPOSIT_AMOUNT>")
    Bucket("<BUCKET_NAME>")
;
CALL_METHOD
    Address("<PROTOCOL_COMPONENT_ADDRESS>")
    "deposit"
    Bucket("<BUCKET_NAME>")
    Proof("<PROOF_NAME>")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
`

DEPOSIT_AMOUNT is decimal number of resources (XRDs) that the user wants to deposit.

##### Withdraw

User can withdraw the deposit or part of it by running this command:

`resim run "./manifests/withdraw.rtm"`

`CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<USER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#<USER_BADGE_ID>#")
    )
;
CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES
    Address("<USER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#<USER_BADGE_ID>#")
    )
    Proof("<PROOF_NAME>")
;
CALL_METHOD
    Address("<PROCOTOL_COMPONENT_ADDRESS>")
    "withdraw"
    Address("<RESOURCE_ADDRESS>")
    Decimal("<WITHDRAW_AMOUNT>")
    Proof("<PROOF_NAME>")
;
ASSERT_WORKTOP_CONTAINS
    Address("<RESOURCE_ADDRESS>")
    Decimal("<WITHDRAW_AMOUNT>")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
`

After this transaction the amount that the user requested should be put in his account.
Accrued interest is staying on the user account.

##### Borrow

User can borrow using the deposited assets as collateral running this command:

`resim run "./manifests/borrow.rtm"`

`CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<USER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#<USER_BADGE_ID>#")
    )
;
CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES
    Address("<USER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#<USER_BADGE_ID>#")
    )
    Proof("<PROOF_NAME>")
;
CALL_METHOD
    Address("<PROTOCOL_COMPONENT_ADDRESS>")
    "borrow"
    Address("<RESOURCE_ADDRESS>")
    Decimal("<BORROW_AMOUNT>")
    Proof("<PROOF_NAME>")
;
ASSERT_WORKTOP_CONTAINS
    Address("<RESOURCE_ADDRESS>")
    Decimal("<BORROW_AMOUNT>")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;`

WITHDRAW_AMOUNT is the amount that te user wants to borrow.
After running this transaction, user should get the borrowed assets on his account.

##### Repay

User can repay his debt by running this command:

`resim run "./manifests/repay.rtm"`

`CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "create_proof_of_non_fungibles"
    Address("<USER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#<USER_BADGE_ID>#")
    )
;
CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES
    Address("<USER_BADGE_ADDRESS>")
    Array<NonFungibleLocalId>(
        NonFungibleLocalId("#<USER_BADGE_ID>#")
    )
    Proof("<PROOF_NAME>")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "withdraw"
    Address("<RESOURCE_ADDRESS>")
    Decimal("<AMOUNT>")
;
TAKE_FROM_WORKTOP
    Address("<RESOURCE_ADDRESS>")
    Decimal("<AMOUNT>")
    Bucket("<BUCKET_NAME>")
;
CALL_METHOD
    Address("<PROTOCOL_COMPONENT_ADDRESS>")
    "repay"
    Bucket("<BUCKET_NAME>")
    Proof("<PROOF_NAME>")
;
CALL_METHOD
    Address("<ACCOUNT_ADDRESS>")
    "deposit_batch"
    Expression("ENTIRE_WORKTOP")
;
`

If there’s some interest, it will be calculated and added to the amount that has to be repaid. If the amount that is sent by the user is smaller then the debt, app is going to calculate how much of the debt is left to be repaid, if it’s greater, it’s going to give back the rest back to the user after it takes the amount needed.
