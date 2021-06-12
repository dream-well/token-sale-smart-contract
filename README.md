# Token sale smart contract
Very simple contract that was used to sell our native token at [btn.group](https://btn.group/secret_network/buttcoin) on the Secret network blockchain.

#### The concept is very simple:
* User sends contract accepted token
* Contract sends user offered token

## Example contract
* Holodeck testnet: [secret10rfstnqg06ngwueu8d60h46slh96c7900hsqmk](https://secretnodes.com/secret/chains/holodeck-2/contracts/secret10rfstnqg06ngwueu8d60h46slh96c7900hsqmk)
* Production: [secret1j6fpcsxp2ts9d8rsh3uj9srvdh0vvg4ewe7tsa](https://secretnodes.com/secret/chains/secret-2/contracts/secret1j6fpcsxp2ts9d8rsh3uj9srvdh0vvg4ewe7tsa)

## Current limitations/recommendations as per review by [baedrik](https://github.com/baedrik)
1. You could consider accepting the admin address as an init parameter instead of sending all received funds to the address that executed the instantiation.  I don’t know if you were planning on having a contract instantiate the token sale contract or doing it manually, but if the admin address was an init parameter, you could instantiate the token sale contract manually and still have funds sent to a timelock/multisig contract.
2. While it probably wouldn’t ever deal with amounts that could create an issue, it’s always a good idea to check for overflow.  So when you are multiplying the deposit amount by the exchange rate, check if it overflows the max value of a 128-bit unsigned int, and throw an error if it does to revert the deposit.  Otherwise a large depositor could get back a lot less of the offered token than they should.  You don’t need to check for overflow when incrementing the total_raised, since the accepted token contract does overflow checking so you could never receive more than u128::MAX, but you could add it if you want to get in the habit.
3. The exchange rate returned in the Config query could be misleading to users depending on the number of decimal points in the accepted token, versus the number of decimal points in the offered token. Everything works as expected if both tokens have the same decimal places.

    #### But for example:
    Let’s say you make the exchange rate 5, the accepted token has 4 decimal places, and the offered token has 6 decimal places. When someone sends 1.0000 accepted tokens, they will receive 0.050000 of the offered tokens.  So the exchange rate is really 0.05, not 5, but the config will say it is 5
