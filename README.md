# About
- Token mint demo
- Swap demo with fixed rate LP (not AMM) 

# Test
### Run all test
```
anchor test
```

### Run single test file
### Run test for single program
```
# rebuild and deploy program IF needed (in terminal tab 2)
# This actually rebuild and redeploy the program
yarn redeploy-program move-token

# run test (in terminal tab 2)
anchor run test-program -- move-token
anchor run test-program -- sol-swap-0

# Should not directly run via yarn because anchor run <sth> will do some more setting up
yarn test-program move-token
```

If /bin/sh does not support inline, you need to change yarn script shell to /bin/bash
> Change yarn script shell to /bin/bash to support inline function
> It's required to use inline function in yarn script.
> Yarn by default uses bin\sh, in which this particular inline Bash function cannot be run.
> 
> ```
> yarn config set script-shell /bin/bash
> ```

# Common name in the code
- ATA: Associated Token Address (Token Account)
- TMA: Token Mint Address (Token Address, Mint, ...)
